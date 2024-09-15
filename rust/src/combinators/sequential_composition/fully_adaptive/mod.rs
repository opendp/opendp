use opendp_derive::proven;

use crate::{
    combinators::{
        Adaptivity, Composability, CompositionMeasure, PendingLoss, assert_components_match,
    },
    core::{
        Domain, Function, Measurement, Metric, MetricSpace, Odometer, OdometerAnswer,
        OdometerQuery, OdometerQueryable, PrivacyMap,
    },
    error::Fallible,
    interactive::{Answer, Query, Queryable, Wrapper},
};

#[cfg(test)]
mod test;

/// Construct an odometer that can spawn a compositor queryable.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
pub fn make_fully_adaptive_composition<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + CompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Fallible<Odometer<DI, MI, MO, Measurement<DI, TO, MI, MO>, TO>>
where
    DI::Carrier: Clone,
    (DI, MI): MetricSpace,
{
    // check if fully adaptive composition is supported
    output_measure.composability(Adaptivity::FullyAdaptive)?;

    Odometer::new(
        input_domain.clone(),
        input_metric.clone(),
        output_measure.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            new_fully_adaptive_composition_queryable(
                input_domain.clone(),
                input_metric.clone(),
                output_measure.clone(),
                arg.clone(),
            )
        }),
    )
}

#[proven(
    proof_path = "combinators/sequential_composition/fully_adaptive/new_fully_adaptive_composition_queryable.tex"
)]
fn new_fully_adaptive_composition_queryable<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + CompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    data: DI::Carrier,
) -> Fallible<OdometerQueryable<Measurement<DI, TO, MI, MO>, TO, MI::Distance, MO::Distance>>
where
    (DI, MI): MetricSpace,
{
    let is_sequential = matches!(
        output_measure.composability(Adaptivity::FullyAdaptive)?,
        Composability::Sequential
    );

    let mut privacy_maps: Vec<PrivacyMap<MI, MO>> = vec![];
    Queryable::new(
        move |self_: &OdometerQueryable<
            Measurement<DI, TO, MI, MO>,
            TO,
            MI::Distance,
            MO::Distance,
        >,
              query: Query<OdometerQuery<Measurement<DI, TO, MI, MO>, MI::Distance>>| {
            // this queryable and wrapped children communicate via an AskPermission query
            // defined here, where no-one else can access the type
            struct AskPermission(pub usize);

            Ok(match query {
                // evaluate external invoke query
                Query::External(OdometerQuery::Invoke(measurement)) => {
                    assert_components_match!(
                        DomainMismatch,
                        &input_domain,
                        &measurement.input_domain
                    );

                    assert_components_match!(
                        MetricMismatch,
                        &input_metric,
                        &measurement.input_metric
                    );

                    assert_components_match!(
                        MeasureMismatch,
                        &output_measure,
                        &measurement.output_measure
                    );

                    let seq_wrapper = is_sequential.then(|| {
                        // when the output measure doesn't allow concurrent composition,
                        // wrap any interactive queryables spawned.
                        // This way, when the child gets a query it sends an AskPermission query to this parent queryable,
                        // giving this sequential odometer queryable
                        // a chance to deny the child permission to execute
                        let child_id = privacy_maps.len();
                        let mut self_ = self_.clone();

                        Wrapper::new_recursive_pre_hook(move || {
                            self_.eval_internal(&AskPermission(child_id))
                        })
                    });

                    // evaluate the query and wrap the answer
                    let answer = measurement.invoke_wrap(&data, seq_wrapper)?;

                    // we've now increased our privacy spend. This is our only state modification
                    privacy_maps.push(measurement.privacy_map.clone());

                    // done!
                    Answer::External(OdometerAnswer::Invoke(answer))
                }
                // evaluate external map query
                Query::External(OdometerQuery::PrivacyLoss(d_in)) => {
                    let d_mids = (privacy_maps.iter())
                        .map(|m| m.eval(d_in))
                        .collect::<Fallible<_>>()?;
                    let d_out = output_measure.compose(d_mids)?;
                    Answer::External(OdometerAnswer::PrivacyLoss(d_out))
                }
                Query::Internal(query) => {
                    // check if the query is from a child queryable who is asking for permission to execute
                    if let Some(AskPermission(id)) = query.downcast_ref() {
                        // deny permission if the sequential odometer has moved on
                        if *id + 1 != privacy_maps.len() {
                            return fallible!(
                                FailedFunction,
                                "sequential odometer has received a new query"
                            );
                        }
                        // otherwise, return Ok to approve the change
                        return Ok(Answer::internal(()));
                    }

                    // handler to see privacy usage after running a query.
                    // Someone is passing in an OdometerQuery internally,
                    // so return the potential privacy loss of this odometer after running this query
                    if let Some(query) = query
                        .downcast_ref::<OdometerQuery<Measurement<DI, TO, MI, MO>, MI::Distance>>()
                    {
                        return Ok(Answer::internal(match query {
                            OdometerQuery::Invoke(meas) => {
                                let mut pending_maps = privacy_maps.clone();
                                pending_maps.push(meas.privacy_map.clone());
                                let output_measure = output_measure.clone();
                                PendingLoss::New(PrivacyMap::<MI, MO>::new_fallible(
                                    move |d_in: &MI::Distance| {
                                        // check if the query is from a child queryable who is asking for permission to execute
                                        let d_mids = (pending_maps.iter())
                                            .map(|m| m.eval(d_in))
                                            .collect::<Fallible<_>>()?;
                                        output_measure.compose(d_mids)
                                    },
                                ))
                            }
                            OdometerQuery::PrivacyLoss(_) => PendingLoss::Same,
                        }));
                    }

                    return fallible!(FailedFunction, "query not recognized");
                }
            })
        },
    )
}

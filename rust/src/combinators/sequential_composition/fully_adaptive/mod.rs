use opendp_derive::proven;

use crate::{
    combinators::{Adaptivity, CompositionMeasure, Sequentiality, assert_components_match},
    core::{
        Domain, Function, Measurement, Metric, MetricSpace, Odometer, OdometerAnswer,
        OdometerQuery, OdometerQueryable,
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
    d_in: MI::Distance,
) -> Fallible<Odometer<DI, MI, MO, Measurement<DI, TO, MI, MO>, TO>>
where
    DI::Carrier: Clone + Send + Sync,
    MI::Distance: Clone + Send + Sync,
    MO::Distance: Clone + Send + Sync,
    (DI, MI): MetricSpace,
{
    let sequential = matches!(
        output_measure.theorem(Adaptivity::FullyAdaptive)?,
        Sequentiality::Sequential
    );

    Odometer::new(
        input_domain.clone(),
        Function::new_fallible(enclose!(
            (input_domain, input_metric, output_measure),
            move |arg: &DI::Carrier| {
                new_fully_adaptive_composition_queryable(
                    input_domain.clone(),
                    input_metric.clone(),
                    output_measure.clone(),
                    d_in.clone(),
                    arg.clone(),
                    sequential,
                )
            }
        )),
        input_metric,
        output_measure,
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
    d_in: MI::Distance,
    data: DI::Carrier,
    sequential: bool,
) -> Fallible<OdometerQueryable<Measurement<DI, TO, MI, MO>, TO, MO::Distance>>
where
    MI::Distance: Clone + Send + Sync,
    MO::Distance: Clone + Send + Sync,
    DI::Carrier: Clone + Send + Sync,
    (DI, MI): MetricSpace,
{
    let mut d_mids: Vec<MO::Distance> = vec![];
    Queryable::new(
        move |self_: &OdometerQueryable<Measurement<DI, TO, MI, MO>, TO, MO::Distance>,
              query: Query<OdometerQuery<Measurement<DI, TO, MI, MO>>>| {
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

                    let d_mid = measurement.map(&d_in)?;

                    let seq_wrapper = sequential.then(|| {
                        // when the output measure doesn't allow concurrent composition,
                        // wrap any interactive queryables spawned.
                        // This way, when the child gets a query it sends an AskPermission query to this parent queryable,
                        // giving this sequential odometer queryable
                        // a chance to deny the child permission to execute
                        let child_id = d_mids.len();
                        let mut self_ = self_.clone();

                        Wrapper::new_recursive_pre_hook(move || {
                            self_.eval_internal(&AskPermission(child_id))
                        })
                    });

                    // evaluate the query and wrap the answer
                    let answer = measurement.invoke_wrap(&data, seq_wrapper)?;

                    // we've now increased our privacy spend. This is our only state modification
                    d_mids.push(d_mid);

                    // done!
                    Answer::External(OdometerAnswer::Invoke(answer))
                }
                // evaluate external map query
                Query::External(OdometerQuery::PrivacyLoss) => Answer::External(
                    OdometerAnswer::PrivacyLoss(output_measure.compose(d_mids.clone())?),
                ),
                Query::Internal(query) => {
                    // check if the query is from a child queryable who is asking for permission to execute
                    if let Some(AskPermission(id)) = query.downcast_ref() {
                        // deny permission if the sequential odometer has moved on
                        if *id + 1 != d_mids.len() {
                            return fallible!(
                                FailedFunction,
                                "sequential odometer has received a new query"
                            );
                        }
                        // otherwise, return Ok to approve the change
                        return Ok(Answer::internal(()));
                    }

                    return fallible!(FailedFunction, "query not recognized");
                }
            })
        },
    )
}

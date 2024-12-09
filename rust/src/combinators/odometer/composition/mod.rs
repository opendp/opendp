use opendp_derive::bootstrap;

use crate::{
    combinators::{assert_components_match, BasicCompositionMeasure},
    core::{
        Domain, Function, Measurement, Metric, MetricSpace, Odometer, OdometerAnswer,
        OdometerQuery, OdometerQueryable, PrivacyMap,
    },
    error::Fallible,
    interactive::{compose_wrappers, Answer, Query, Queryable, Wrapper},
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(output_measure(c_type = "AnyMeasure *", rust_type = b"null")),
    generics(DI(suppress), TO(suppress), MI(suppress), MO(suppress))
)]
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
    MO: 'static + BasicCompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Fallible<Odometer<DI, MI, MO, Measurement<DI, TO, MI, MO>, TO>>
where
    DI::Carrier: Clone + Send + Sync,
    (DI, MI): MetricSpace,
{
    Odometer::new(
        input_domain.clone(),
        Function::new_interactive(enclose!(
            (input_domain, input_metric, output_measure),
            move |arg: &DI::Carrier, wrapper: Option<Wrapper>| {
                new_fully_adaptive_composition_queryable(
                    input_domain.clone(),
                    input_metric.clone(),
                    output_measure.clone(),
                    arg.clone(),
                )
                .wrap(wrapper)
            }
        )),
        input_metric,
        output_measure,
    )
}

fn new_fully_adaptive_composition_queryable<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + BasicCompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    data: DI::Carrier,
) -> OdometerQueryable<MI, MO, Measurement<DI, TO, MI, MO>, TO>
where
    DI::Carrier: Clone + Send + Sync,
{
    let mut child_maps: Vec<PrivacyMap<MI, MO>> = vec![];
    Queryable::new(
        move |self_: &OdometerQueryable<MI, MO, Measurement<DI, TO, MI, MO>, TO>,
              query: Query<OdometerQuery<Measurement<DI, TO, MI, MO>, MI::Distance>>| {
            // this queryable and wrapped children communicate via an AskPermission query
            // defined here, where no-one else can access the type
            struct AskPermission(pub usize);

            Ok(match query {
                // evaluate external invoke query
                Query::External(OdometerQuery::Invoke(measurement), query_wrapper) => {
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

                    let seq_wrapper = (!output_measure.concurrent()?).then(|| {
                        // when the output measure doesn't allow concurrent composition,
                        // wrap any interactive queryables spawned.
                        // This way, when the child gets a query it sends an AskPermission query to this parent queryable,
                        // giving this sequential odometer queryable
                        // a chance to deny the child permission to execute
                        let child_id = child_maps.len();
                        let mut self_ = self_.clone();

                        Wrapper::new_recursive_pre_hook(move || {
                            self_.eval_internal(&AskPermission(child_id))
                        })
                    });

                    let wrapper = compose_wrappers(query_wrapper, seq_wrapper);

                    let answer = measurement.invoke_wrap(&data, wrapper)?;

                    // we've now increased our privacy spend. This is our only state modification
                    child_maps.push(measurement.privacy_map.clone());

                    Answer::External(OdometerAnswer::Invoke(answer))
                }
                // evaluate external map query
                Query::External(OdometerQuery::Map(d_in), _) => {
                    let d_out = output_measure.compose(
                        child_maps
                            .iter()
                            .map(|pmap| pmap.eval(&d_in))
                            .collect::<Fallible<_>>()?,
                    )?;
                    Answer::External(OdometerAnswer::Map(d_out))
                }
                Query::Internal(query) => {
                    // handler to see privacy usage after running a query.
                    // Someone is passing in an OdometerQuery internally,
                    // so return the potential privacy map of this odometer after running this query
                    if let Some(OdometerQuery::Invoke(meas)) = query
                        .downcast_ref::<OdometerQuery<Measurement<DI, TO, MI, MO>, MI::Distance>>()
                    {
                        let mut pending_child_maps = child_maps.clone();
                        pending_child_maps.push(meas.privacy_map.clone());

                        let pending_map: PrivacyMap<MI, MO> = PrivacyMap::new_fallible(enclose!(
                            (output_measure, pending_child_maps),
                            move |d_in| {
                                output_measure.compose(
                                    pending_child_maps
                                        .iter()
                                        .map(|pmap| pmap.eval(d_in))
                                        .collect::<Fallible<_>>()?,
                                )
                            }
                        ));

                        return Ok(Answer::internal(pending_map));
                    }

                    // check if the query is from a child queryable who is asking for permission to execute
                    if let Some(AskPermission(id)) = query.downcast_ref() {
                        // deny permission if the sequential odometer has moved on
                        if *id + 1 != child_maps.len() {
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
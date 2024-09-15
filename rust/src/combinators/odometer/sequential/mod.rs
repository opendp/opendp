use crate::{
    combinators::{assert_components_match, BasicCompositionMeasure, OdometerAnswer},
    core::{Domain, Function, Measurement, Metric, MetricSpace, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{compose_wrappers, Answer, Query, Queryable, Wrapper},
};

use super::{CompositionOdometerQueryable, OdometerQuery};

#[cfg(test)]
mod test;

/// Construct an odometer that can spawn a compositor queryable.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
pub fn make_composition_odometer<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + BasicCompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Fallible<Odometer<DI, CompositionOdometerQueryable<DI, TO, MI, MO>, MI, MO>>
where
    DI::Carrier: Clone + Send + Sync,
    (DI, MI): MetricSpace,
{
    Odometer::new(
        input_domain.clone(),
        Function::new_interactive(enclose!(
            (input_domain, input_metric, output_measure),
            move |arg: &DI::Carrier, wrapper: Option<Wrapper>| {
                new_composition_odometer_queryable(
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

fn new_composition_odometer_queryable<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + BasicCompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    data: DI::Carrier,
) -> CompositionOdometerQueryable<DI, TO, MI, MO>
where
    DI::Carrier: Clone + Send + Sync,
{
    let mut child_maps: Vec<PrivacyMap<MI, MO>> = vec![];
    Queryable::new(
        move |sc_qbl: &Queryable<_, _>,
              query: Query<OdometerQuery<Measurement<DI, TO, MI, MO>, MI::Distance>>| {
            // this queryable and wrapped children communicate via an AskPermission query
            // defined here, where no-one else can access the type
            struct AskPermission(pub usize);

            Ok(match query {
                // evaluate external invoke query
                Query::External(OdometerQuery::Invoke(measurement), query_wrapper) => {
                    assert_components_match!(
                        DomainMismatch,
                        input_domain,
                        measurement.input_domain.clone()
                    );

                    assert_components_match!(
                        MetricMismatch,
                        input_metric,
                        measurement.input_metric.clone()
                    );

                    assert_components_match!(
                        MeasureMismatch,
                        output_measure,
                        measurement.output_measure.clone()
                    );

                    let seq_wrapper = (!output_measure.concurrent()?).then(|| {
                        // when the output measure doesn't allow concurrent composition,
                        // wrap any interactive queryables spawned.
                        // This way, when the child gets a query it sends an AskPermission query to this parent queryable,
                        // giving this sequential odometer queryable
                        // a chance to deny the child permission to execute
                        let child_id = child_maps.len();
                        let mut sc_qbl = sc_qbl.clone();

                        Wrapper::new_recursive_pre_hook(move || {
                            sc_qbl.eval_internal(&AskPermission(child_id))
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

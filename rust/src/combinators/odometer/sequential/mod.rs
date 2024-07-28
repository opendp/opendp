use crate::{
    combinators::{
        assert_components_match, odometer::GetId, BasicCompositionMeasure, OdometerAnswer,
    },
    core::{Domain, Function, Metric, MetricSpace, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{Answer, Query, Queryable, WrapFn},
};

use super::{ChildChange, IsInvokable, OdometerQuery, OdometerQueryable};

#[cfg(test)]
mod test;

/// Construct a sequential odometer queryable that interactively composes odometers or interactive measurements.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
pub fn make_sequential_odometer<
    DI: 'static + Domain,
    Q: 'static + IsInvokable<DI, MI, MO>,
    MI: 'static + Metric,
    MO: 'static + BasicCompositionMeasure,
>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Fallible<Odometer<DI, OdometerQueryable<Q, Q::Output, MI::Distance, MO::Distance>, MI, MO>>
where
    MI::Distance: 'static + Clone,
    DI::Carrier: Clone + Send + Sync,
    MO::Distance: Clone,
    (DI, MI): MetricSpace,
{
    Odometer::new(
        input_domain.clone(),
        Function::new_wrappable(enclose!(
            (input_domain, input_metric, output_measure),
            move |arg: &DI::Carrier, wrapper: Option<WrapFn>| {
                // IMMUTABLE STATE VARIABLES
                let input_domain = input_domain.clone();
                let input_metric = input_metric.clone();
                let output_measure = output_measure.clone();
                let arg = arg.clone();

                // MUTABLE STATE VARIABLES
                let mut child_maps: Vec<PrivacyMap<MI, MO>> = vec![];

                Queryable::new(
                    move |sc_qbl: &Queryable<_, _>,
                          query: Query<OdometerQuery<Q, MI::Distance>>| {
                        // this queryable and wrapped children communicate via an AskPermission query
                        // defined here, where no-one else can access the type
                        struct AskPermission(pub usize);

                        Ok(match query {
                            // evaluate external invoke query
                            Query::External(OdometerQuery::Invoke(invokable), wrapper) => {
                                assert_components_match!(
                                    DomainMismatch,
                                    input_domain,
                                    invokable.input_domain()
                                );

                                assert_components_match!(
                                    MetricMismatch,
                                    input_metric,
                                    invokable.input_metric()
                                );

                                assert_components_match!(
                                    MeasureMismatch,
                                    output_measure,
                                    invokable.output_measure()
                                );

                                let child_id = child_maps.len();
                                let mut sc_qbl = sc_qbl.clone();
                                let sequentiality_constraint = WrapFn::new_pre_hook(move || {
                                    sc_qbl.eval_internal(&AskPermission(child_id))
                                });

                                let (answer, privacy_map) = invokable.invoke_wrap_and_map(
                                    &arg,
                                    Some(WrapFn::new(move |_, mut inner_qbl| {
                                        sequentiality_constraint.call(Queryable::new(
                                            move |_, query| {
                                                if let Query::Internal(int) = query {
                                                    if int.downcast_ref::<GetId>().is_some() {
                                                        return Ok(Answer::internal(child_id));
                                                    }
                                                };
                                                inner_qbl.eval_query(query)
                                            },
                                            wrapper.clone(),
                                        )?)
                                    })),
                                )?;

                                child_maps.push(privacy_map);

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
                                if let Some(ChildChange {
                                    id,
                                    new_privacy_map,
                                }) = query.downcast_ref()
                                {
                                    let mut pending_child_maps = child_maps.clone();
                                    pending_child_maps[*id] = new_privacy_map.clone();

                                    let pending_map: PrivacyMap<MI, MO> =
                                        PrivacyMap::new_fallible(enclose!(
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

                                // handler to see privacy usage after running a query.
                                // Someone is passing in an OdometerQuery internally,
                                // so return the potential privacy map of this odometer after running this query
                                if let Some(OdometerQuery::Invoke(query)) =
                                    query.downcast_ref::<OdometerQuery<Q, MI::Distance>>()
                                {
                                    let mut pending_child_maps = child_maps.clone();
                                    if let Some(privacy_map) = query.one_time_privacy_map() {
                                        pending_child_maps.push(privacy_map);
                                    }

                                    let pending_map: PrivacyMap<MI, MO> =
                                        PrivacyMap::new_fallible(enclose!(
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
                                    // deny permission if the sequential compositor has moved on
                                    if *id != child_maps.len() {
                                        return fallible!(
                                            FailedFunction,
                                            "sequential compositor has received a new query"
                                        );
                                    }
                                    // otherwise, return Ok to approve the change
                                    return Ok(Answer::internal(()));
                                }

                                return fallible!(FailedFunction, "query not recognized");
                            }
                        })
                    },
                    wrapper,
                )
            }
        )),
        input_metric,
        output_measure,
    )
}

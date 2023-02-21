use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{Query, Queryable},
    traits::TotalOrd,
};

use super::{ChildChange, Invokable, OdometerQuery, OdometerQueryable};

pub fn make_odometer_to_filter<
    DI: 'static + Domain,
    Q: 'static + Invokable<DI, MI, MO> + Clone,
    MI: 'static + Metric + Default,
    MO: 'static + Measure,
>(
    odometer: Odometer<DI, OdometerQueryable<Q, Q::Output, MI::Distance, MO::Distance>, MI, MO>,
    d_in: MI::Distance,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, OdometerQueryable<Q, Q::Output, MI::Distance, MO::Distance>, MI, MO>>
where
    MI::Distance: 'static + Clone + TotalOrd,
    DI::Carrier: Clone,
    MO::Distance: Clone + TotalOrd,
{
    let Odometer {
        input_domain,
        function,
        input_metric,
        output_measure,
    } = odometer;

    Ok(Measurement::new(
        input_domain,
        Function::new_fallible(enclose!((d_in, d_out), move |arg: &DI::Carrier| {
            let d_in = d_in.clone();
            let d_out = d_out.clone();
            let mut odom_queryable = function.eval(arg)?;

            Ok(Queryable::new(
                move |_self: &Queryable<_, _>, query: Query<OdometerQuery<Q, MI::Distance>>| {
                    // 1. catch all possible queries that may change the budget
                    let pending_child_change: ChildChange<MI, MO> = match query {
                        Query::External(OdometerQuery::Invoke(invokable)) => ChildChange {
                            id: None,
                            new_privacy_map: invokable.privacy_map(),
                            commit: false,
                        },
                        Query::Internal(query) => {
                            if let Some(change) = query.downcast_ref::<ChildChange<MI, MO>>() {
                                change.clone()
                            } else {
                                return fallible!(FailedFunction, "unrecognized internal query");
                            }
                        }
                        // otherwise, if the query can't change the budget, then evaluate it
                        query => return odom_queryable.eval_query(query),
                    };

                    // 2. retrieve what the privacy map would be after the child change
                    let pending_privacy_map: PrivacyMap<MI, MO> =
                        odom_queryable.eval_internal(&pending_child_change)?;

                    // 3. check if the filter is satisfied
                    if pending_privacy_map.eval(&d_in)?.total_gt(&d_out)? {
                        return fallible!(FailedFunction, "insufficient budget");
                    }

                    // 4. filter is satisfied with the changes, so execute the query
                    odom_queryable.eval_query(query)
                },
            ))
        })),
        input_metric,
        output_measure,
        PrivacyMap::new_fallible(move |d_in_p: &MI::Distance| {
            if d_in_p.total_gt(&d_in)? {
                fallible!(
                    RelationDebug,
                    "input distance must not be greater than d_in"
                )
            } else {
                Ok(d_out.clone())
            }
        }),
    ))
}

// pub fn make_filter<
//     DI: Domain + 'static,
//     DQ: Domain + 'static,
//     DAQ: Domain + 'static,
//     DAA: Domain + 'static,
//     MI: Metric + 'static,
//     MO: BasicCompositionMeasure + 'static,
// >(
//     input_domain: DI,
//     query_domain: DQ,
//     query2_domain: DAQ,
//     answer2_domain: DAA,
//     output_measure: MO,
//     d_in: MI::Distance,
//     d_out: MO::Distance,
// ) -> Fallible<Measurement<DI, DQ, QueryableDomain<DAQ, DAA>, MI, MO>>
// where
//     MI::Distance: 'static + TotalOrd + Clone,
//     DI::Carrier: 'static + Clone,
//     DQ::Carrier: 'static + Clone,
//     MO::Distance: 'static + TotalOrd + Clone + InfAdd + Zero,
//     DQ::Carrier: Invokable<DI, DAQ, DAA, MI, MO>,
// {
//     make_odometer_to_filter(
//         make_odometer(
//             input_domain,
//             query_domain,
//             query2_domain,
//             answer2_domain,
//             output_measure,
//             d_in,
//         )?,
//         d_out,
//     )
// }

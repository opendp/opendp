use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{wrap, Answer, IntoPolyQueryable, Query, Queryable, WrapFn},
    traits::TotalOrd,
};

use super::{ChildChange, Invokable, OdometerQueryable};

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
    (DI, MI): MetricSpace,
{
    let Odometer {
        input_domain,
        function,
        input_metric,
        output_measure,
    } = odometer;

    Measurement::new(
        input_domain,
        Function::new_fallible(enclose!((d_in, d_out), move |arg: &DI::Carrier| {
            let d_in = d_in.clone();
            let d_out = d_out.clone();

            let filter_queryable =
                Queryable::<(), ()>::new(move |_self: &Queryable<_, _>, query: Query<()>| {
                    if let Query::Internal(query) = query {
                        if let Some(change) = query.downcast_ref::<ChildChange<MI, MO>>() {
                            if change.new_privacy_map.eval(&d_in)?.total_gt(&d_out)? {
                                return fallible!(FailedFunction, "insufficient budget");
                            }
                            return Ok(Answer::internal(change.new_privacy_map.clone()));
                        }
                    }
                    fallible!(FailedFunction, "query not understood")
                })?;
            let wrap_logic = WrapFn::new_odo::<MI, MO>(filter_queryable.into_poly());
            wrap(wrap_logic.as_map(), || function.eval(arg))
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
    )
}

use std::any::Any;

use num::Zero;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, Odometer, PrivacyMap},
    domains::QueryableDomain,
    error::Fallible,
    interactive::{Queryable, QueryableBase, Context},
    traits::{InfAdd, TotalOrd},
};

use super::{make_odometer, BasicCompositionMeasure};

pub fn make_odometer_to_filter<
    DI: Domain + 'static,
    DO: Domain + 'static,
    MI: Metric + 'static,
    MO: Measure + Default + 'static,
>(
    odometer: Odometer<DI, QueryableDomain<Measurement<DI, DO, MI, MO>, DO>, MI, MO>,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, QueryableDomain<Measurement<DI, DO, MI, MO>, DO>, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd + PartialOrd,
{
    let Odometer {
        input_domain,
        output_domain,
        function,
        input_metric,
        output_measure,
        d_in,
    } = odometer;

    Ok(Measurement::new(
        input_domain,
        output_domain,
        Function::new_fallible(enclose!(d_out, move |arg: &DI::Carrier| {
            let d_out = d_out.clone();
            let mut odom_queryable = function.eval(arg)?;

            Ok(Queryable::new(
                move |self_: &QueryableBase, query: &dyn Any| {
                    if let Some(measurement) = query.downcast_ref::<Measurement<DI, DO, MI, MO>>() {
                        let d_out_pending: MO::Distance = odom_queryable.eval_privacy_after(measurement)?;

                        if d_out_pending > d_out {
                            return fallible!(FailedFunction, "insufficient budget");
                        }
                        let mut answer = odom_queryable.eval(measurement)?;

                        DO::inject_context::<MO::Distance>(
                            &mut answer,
                            Context::new(self_.clone(), 0),
                        );

                        return Ok(Box::new(answer));
                    }

                    odom_queryable.base.eval_any(query)
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

pub fn make_filter<
    DI: Domain + 'static,
    DO: Domain + 'static,
    MI: Metric + 'static,
    MO: BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    output_measure: MO,
    d_in: MI::Distance,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, QueryableDomain<Measurement<DI, DO, MI, MO>, DO>, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd + Zero,
{
    make_odometer_to_filter(make_odometer(input_domain, output_measure, d_in)?, d_out)
}

use std::any::Any;

use num::Zero;

use crate::{
    core::{Domain, Function, Measure, Metric, Odometer, PrivacyMap, Measurement},
    error::Fallible,
    interactive::{Queryable, QueryableBase},
    traits::{InfAdd, TotalOrd},
};

use super::{make_odometer, BasicCompositionMeasure, Invokable};

pub fn make_odometer_to_filter<
    DI: Domain + 'static,
    DQ: Domain + 'static,
    DA: Domain + 'static,
    MI: Metric + 'static,
    MO: Measure + Default + 'static,
>(
    odometer: Odometer<DI, DQ, DA, MI, MO>,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, DQ, DA, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DQ::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd + PartialOrd,
{
    let Odometer {
        input_domain,
        query_domain,
        answer_domain,
        function,
        input_metric,
        output_measure,
        d_in,
    } = odometer;

    Ok(Measurement::new(
        input_domain,
        query_domain,
        answer_domain,
        Function::new_fallible(enclose!(d_out, move |arg: &DI::Carrier| {
            let d_out = d_out.clone();
            let mut odom_queryable = function.eval(arg)?;

            Ok(Queryable::new(
                move |_self: &QueryableBase, query: &dyn Any| {
                    if let Some(query) = query.downcast_ref::<DQ::Carrier>() {
                        let d_out_pending: MO::Distance = odom_queryable.eval_privacy_after(query)?;

                        if d_out_pending > d_out {
                            return fallible!(FailedFunction, "insufficient budget");
                        }
                        let answer = odom_queryable.eval(query)?;
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
    DQ: Domain + 'static,
    DA: Domain + 'static,
    MI: Metric + 'static,
    MO: BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    query_domain: DQ,
    answer_domain: DA,
    output_measure: MO,
    d_in: MI::Distance,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, DQ, DA, MI, MO>>
where
    MI::Distance: 'static + TotalOrd + Clone,
    DI::Carrier: 'static + Clone,
    DQ::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Clone + InfAdd + Zero,
    DQ::Carrier: Invokable<DI, DQ, DA, MI, MO>
{
    make_odometer_to_filter(make_odometer(input_domain, query_domain, answer_domain, output_measure, d_in)?, d_out)
}

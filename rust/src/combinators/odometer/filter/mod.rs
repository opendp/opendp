use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{compose_wrappers, Query, Queryable, Wrapper},
    traits::ProductOrd,
};
use std::fmt::Debug;

use super::OdometerCompositor;

pub fn make_odometer_to_filter<
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
>(
    odometer: Odometer<DI, OdometerCompositor<DI, TO, MI, MO>, MI, MO>,
    d_in: MI::Distance,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, OdometerCompositor<DI, TO, MI, MO>, MI, MO>>
where
    DI::Carrier: Clone + Send + Sync,
    MI::Distance: Clone + Debug + ProductOrd + Send + Sync + 'static,
    MO::Distance: Clone + Debug + ProductOrd,
    (DI, MI): MetricSpace,
{
    let Odometer {
        input_domain,
        function,
        input_metric,
        output_measure,
    } = odometer;

    let (d_in_, d_out_) = (d_in.clone(), d_out.clone());

    Measurement::new(
        input_domain,
        Function::new_interactive(move |arg: &DI::Carrier, query_wrapper: Option<Wrapper>| {
            let continuation_rule = Wrapper::continuation_rule::<MI, MO>(&d_in, &d_out);

            let wrapper = compose_wrappers(Some(continuation_rule), query_wrapper);
            function.eval_wrap(arg, wrapper)
        }),
        input_metric,
        output_measure,
        PrivacyMap::new_fallible(move |d_in_p: &MI::Distance| {
            if d_in_p.total_gt(&d_in_)? {
                fallible!(
                    RelationDebug,
                    "input distance must not be greater than d_in"
                )
            } else {
                Ok(d_out_.clone())
            }
        }),
    )
}

impl Wrapper {
    fn continuation_rule<MI: 'static + Metric, MO: 'static + Measure>(
        d_in: &MI::Distance,
        d_out: &MO::Distance,
    ) -> Wrapper
    where
        MI::Distance: Clone + Debug + Send + Sync + 'static,
        MO::Distance: Clone + Debug + ProductOrd + 'static,
    {
        let (d_in, d_out) = (d_in.clone(), d_out.clone());
        Wrapper::new(move |mut odo_qbl| {
            let (d_in, d_out) = (d_in.clone(), d_out.clone());
            Ok(Queryable::new(move |_, query| {
                if let Query::External(external, _) = query {
                    let pending_privacy_map: PrivacyMap<MI, MO> =
                        odo_qbl.eval_internal(external)?;
                    let pending_d_out = pending_privacy_map.eval(&d_in)?;
                    if pending_d_out.total_gt(&d_out)? {
                        return fallible!(
                            FailedFunction,
                            "insufficient budget: {:?} > {:?}",
                            pending_d_out,
                            d_out
                        );
                    }
                }
                odo_qbl.eval_query(query)
            }))
        })
    }
}

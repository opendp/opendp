use crate::{
    core::{
        Domain, Function, Measure, Measurement, Metric, MetricSpace, Odometer, OdometerQueryable,
        PrivacyMap,
    },
    error::Fallible,
    interactive::{compose_wrappers, Query, Queryable, Wrapper},
    traits::ProductOrd,
};
use std::fmt::Debug;

/// Combinator that limits the privacy loss of an odometer.
///
/// Adjusts the queryable returned by the odometer
/// to reject any query that would increase the total privacy loss
/// above the privacy guarantee of the mechanism.
///
/// # Arguments
/// * `odometer` - A privacy odometer
/// * `d_in` - Upper bound on the distance between adjacent input datasets.
/// * `d_out` - Upper bound on the distance between distributions of releases on adjacent input datasets.
pub fn make_odometer_to_filter<
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    Q: 'static,
    A: 'static,
>(
    odometer: Odometer<DI, MI, MO, Q, A>,
    d_in: MI::Distance,
    d_out: MO::Distance,
) -> Fallible<Measurement<DI, OdometerQueryable<MI, MO, Q, A>, MI, MO>>
where
    DI::Carrier: Clone + Send + Sync,
    MI::Distance: Clone + Debug + ProductOrd + Send + Sync + 'static,
    MO::Distance: Clone + Debug + ProductOrd,
    (DI, MI): MetricSpace,
{
    let function = odometer.function.clone();

    let (d_in_, d_out_) = (d_in.clone(), d_out.clone());

    Measurement::new(
        odometer.input_domain.clone(),
        Function::new_interactive(move |arg: &DI::Carrier, query_wrapper: Option<Wrapper>| {
            let continuation_rule = Wrapper::continuation_rule::<MI, MO>(&d_in, &d_out);

            let wrapper = compose_wrappers(Some(continuation_rule), query_wrapper);
            function.eval_wrap(arg, wrapper)
        }),
        odometer.input_metric.clone(),
        odometer.output_measure.clone(),
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

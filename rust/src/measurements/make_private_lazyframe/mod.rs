use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, Measure, Measurement, Metric, MetricSpace},
    domains::{LazyFrameDomain, LogicalPlanDomain},
    error::Fallible,
    metrics::SymmetricDistance,
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(output_measure(c_type = "AnyMeasure *", rust_type = b"null")),
    generics(MI(suppress), MO(suppress))
)]
/// Create a differentially private measurement from a [`LazyFrame`].
///
/// # Arguments
/// * `input_domain` - The domain of the input data.
/// * `input_metric` - How to measure distances between neighboring input data sets.
/// * `output_measure` - How to measure privacy loss.
/// * `lazyframe` - The [`LazyFrame`] to be privatized.
/// * `param` - A tune-able parameter that affects the privacy-utility tradeoff.
pub fn make_private_lazyframe<MI: Metric, MO: 'static + Measure>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    output_measure: MO,
    lazyframe: LazyFrame,
    param: f64,
) -> Fallible<Measurement<LazyFrameDomain, LazyFrame, MI, MO>>
where
    LogicalPlan: PrivateLogicalPlan<MI, MO>,
    (LogicalPlanDomain, MI): MetricSpace,
    (LazyFrameDomain, MI): MetricSpace,
{
    let m_lp = lazyframe.logical_plan.make_private(
        input_domain.cast_carrier(),
        input_metric,
        output_measure,
        param,
    )?;
    let f_lp = m_lp.function.clone();

    Measurement::new(
        m_lp.input_domain.cast_carrier(),
        Function::new_fallible(move |arg: &LazyFrame| {
            Ok(LazyFrame::from(f_lp.eval(&arg.logical_plan)?)
                .with_optimizations(arg.get_current_optimizations()))
        }),
        m_lp.input_metric.clone(),
        m_lp.output_measure.clone(),
        m_lp.privacy_map.clone(),
    )
}

pub trait PrivateLogicalPlan<MI: Metric, MO: Measure> {
    fn make_private(
        self,
        input_domain: LogicalPlanDomain,
        input_metric: MI,
        output_measure: MO,
        param: f64,
    ) -> Fallible<Measurement<LogicalPlanDomain, LogicalPlan, MI, MO>>;
}

impl<MO: Measure> PrivateLogicalPlan<SymmetricDistance, MO> for LogicalPlan {
    fn make_private(
        self,
        _input_domain: LogicalPlanDomain,
        _input_metric: SymmetricDistance,
        _output_measure: MO,
        _param: f64,
    ) -> Fallible<Measurement<LogicalPlanDomain, LogicalPlan, SymmetricDistance, MO>> {
        match &self {
            lp => fallible!(
                MakeMeasurement,
                "A step in your logical plan is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                lp
            )
        }
    }
}

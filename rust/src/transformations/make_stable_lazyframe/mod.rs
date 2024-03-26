use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;
use polars_plan::logical_plan::LogicalPlan;

use crate::{
    core::{Function, Metric, MetricSpace, Transformation},
    domains::{LazyFrameDomain, LogicalPlanDomain},
    error::Fallible,
    metrics::SymmetricDistance,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod source;

#[bootstrap(
    features("contrib"),
    arguments(output_metric(c_type = "AnyMetric *", rust_type = b"null")),
    generics(MI(suppress), MO(suppress))
)]
/// Create a stable transformation from a [`LazyFrame`].
///
/// # Arguments
/// * `input_domain` - The domain of the input data.
/// * `input_metric` - How to measure distances between neighboring input data sets.
/// * `lazyframe` - The [`LazyFrame`] to be analyzed.
pub fn make_stable_lazyframe<MI: 'static + Metric, MO: 'static + Metric>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    lazyframe: LazyFrame,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, MI, MO>>
where
    LogicalPlan: StableLogicalPlan<MI, MO>,
    (LazyFrameDomain, MI): MetricSpace,
    (LazyFrameDomain, MO): MetricSpace,
    (LogicalPlanDomain, MI): MetricSpace,
    (LogicalPlanDomain, MO): MetricSpace,
{
    let t_lp = lazyframe
        .logical_plan
        .make_stable(input_domain.cast_carrier(), input_metric)?;
    let f_lp = t_lp.function.clone();

    Transformation::new(
        t_lp.input_domain.cast_carrier(),
        t_lp.output_domain.cast_carrier(),
        Function::new_fallible(move |arg: &LazyFrame| {
            Ok(LazyFrame::from(f_lp.eval(&arg.logical_plan)?)
                .with_optimizations(arg.get_current_optimizations()))
        }),
        t_lp.input_metric.clone(),
        t_lp.output_metric.clone(),
        t_lp.stability_map.clone(),
    )
}

pub trait StableLogicalPlan<MI: Metric, MO: Metric> {
    fn make_stable(
        self,
        input_domain: LogicalPlanDomain,
        input_metric: MI,
    ) -> Fallible<Transformation<LogicalPlanDomain, LogicalPlanDomain, MI, MO>>;
}

impl StableLogicalPlan<SymmetricDistance, SymmetricDistance> for LogicalPlan {
    fn make_stable(
        self,
        input_domain: LogicalPlanDomain,
        input_metric: SymmetricDistance,
    ) -> Fallible<
        Transformation<LogicalPlanDomain, LogicalPlanDomain, SymmetricDistance, SymmetricDistance>,
    > {
        match &self {
            LogicalPlan::DataFrameScan { .. } => {
                source::make_stable_source(input_domain, input_metric, self)
            }
            lp => fallible!(
                MakeTransformation,
                "A step in your logical plan is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                lp
            )
        }
    }
}

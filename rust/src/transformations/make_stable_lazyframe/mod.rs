use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;
use polars_plan::plans::DslPlan;

use crate::{
    core::{Function, Metric, MetricSpace, Transformation},
    domains::{DslPlanDomain, LazyFrameDomain},
    error::Fallible,
    metrics::SymmetricDistance,
    polars::get_disabled_features_message,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod source;

#[cfg(feature = "contrib")]
mod filter;

#[cfg(feature = "contrib")]
mod h_stack;

#[cfg(feature = "contrib")]
pub(crate) mod select;

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
    DslPlan: StableDslPlan<MI, MO>,
    (LazyFrameDomain, MI): MetricSpace,
    (LazyFrameDomain, MO): MetricSpace,
    (DslPlanDomain, MI): MetricSpace,
    (DslPlanDomain, MO): MetricSpace,
{
    let t_input = lazyframe
        .logical_plan
        .make_stable(input_domain.cast_carrier(), input_metric)?;
    let f_input = t_input.function.clone();

    Transformation::new(
        t_input.input_domain.cast_carrier(),
        t_input.output_domain.cast_carrier(),
        Function::new_fallible(move |arg: &LazyFrame| {
            Ok(LazyFrame::from(f_input.eval(&arg.logical_plan)?)
                .with_optimizations(arg.get_current_optimizations()))
        }),
        t_input.input_metric.clone(),
        t_input.output_metric.clone(),
        t_input.stability_map.clone(),
    )
}

pub trait StableDslPlan<MI: Metric, MO: Metric> {
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: MI,
    ) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, MI, MO>>;
}

impl StableDslPlan<SymmetricDistance, SymmetricDistance> for DslPlan {
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: SymmetricDistance,
    ) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, SymmetricDistance, SymmetricDistance>>
    {
        match &self {
            DslPlan::DataFrameScan { .. } => {
                source::make_stable_source(input_domain, input_metric, self)
            }
            DslPlan::Filter { .. } => {
                filter::make_stable_filter(input_domain, input_metric, self)
            }
            DslPlan::HStack { .. } => {
                h_stack::make_h_stack(input_domain, input_metric, self)
            }
            DslPlan::Select { .. } => {
                select::make_select(input_domain, input_metric, self)
            }
            dsl => {
                match dsl.describe() {
                    Ok(describe) => fallible!(
                        MakeTransformation,
                        "A step in your query is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                        describe,
                        get_disabled_features_message()
                    ),
                    Err(e) => fallible!(
                        MakeTransformation,
                        "A step in your query is not recognized at this time, and the step cannot be identified due to the following error: {}. {:?}",
                        e,
                        get_disabled_features_message()
                    )
                }
            }
        }
    }
}

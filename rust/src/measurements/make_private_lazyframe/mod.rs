use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    combinators::BasicCompositionMeasure,
    core::{Function, Measure, Measurement, Metric, MetricSpace},
    domains::{DslPlanDomain, ExprDomain, LazyFrameDomain},
    error::Fallible,
    metrics::PartitionDistance,
    polars::{get_disabled_features_message, OnceFrame},
    transformations::{traits::UnboundedMetric, DatasetMetric, StableDslPlan},
};

use super::PrivateExpr;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod group_by;

#[cfg(feature = "contrib")]
mod postprocess;

#[cfg(feature = "contrib")]
mod select;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        global_scale(rust_type = "Option<f64>", c_type = "AnyObject *", default = b"null")
    ),
    generics(MI(suppress), MO(suppress))
)]
/// Create a differentially private measurement from a [`LazyFrame`].
///
/// Any data inside the [`LazyFrame`] is ignored,
/// but it is still recommended to start with an empty [`DataFrame`] and build up the computation using the [`LazyFrame`] API.
///
/// # Arguments
/// * `input_domain` - The domain of the input data.
/// * `input_metric` - How to measure distances between neighboring input data sets.
/// * `output_measure` - How to measure privacy loss.
/// * `lazyframe` - A description of the computations to be run, in the form of a [`LazyFrame`].
/// * `global_scale` - A tune-able parameter that affects the privacy-utility tradeoff.
pub fn make_private_lazyframe<MI: Metric, MO: 'static + Measure>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    output_measure: MO,
    lazyframe: LazyFrame,
    global_scale: Option<f64>,
) -> Fallible<Measurement<LazyFrameDomain, OnceFrame, MI, MO>>
where
    DslPlan: PrivateDslPlan<MI, MO>,
    (DslPlanDomain, MI): MetricSpace,
    (LazyFrameDomain, MI): MetricSpace,
{
    let m_lp = lazyframe.logical_plan.make_private(
        input_domain.cast_carrier(),
        input_metric,
        output_measure,
        global_scale,
    )?;
    let f_lp = m_lp.function.clone();

    Measurement::new(
        m_lp.input_domain.cast_carrier(),
        Function::new_fallible(move |arg: &LazyFrame| {
            let lf = LazyFrame::from(f_lp.eval(&arg.logical_plan)?)
                .with_optimizations(arg.get_current_optimizations());
            Ok(OnceFrame::from(lf))
        }),
        m_lp.input_metric.clone(),
        m_lp.output_measure.clone(),
        m_lp.privacy_map.clone(),
    )
}

pub trait PrivateDslPlan<MI: Metric, MO: Measure> {
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: MI,
        output_measure: MO,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<DslPlanDomain, DslPlan, MI, MO>>;
}

impl<MS, MO> PrivateDslPlan<MS, MO> for DslPlan
where
    MS: 'static + UnboundedMetric + DatasetMetric,
    MO: 'static + BasicCompositionMeasure,
    Expr: PrivateExpr<PartitionDistance<MS>, MO>,
    DslPlan: StableDslPlan<MS, MS>,
    (DslPlanDomain, MS): MetricSpace,
    (ExprDomain, MS): MetricSpace,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: MS,
        output_measure: MO,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<DslPlanDomain, DslPlan, MS, MO>> {
        if let Some(meas) = postprocess::match_postprocess(
            input_domain.clone(),
            input_metric.clone(),
            output_measure.clone(),
            self.clone(),
            global_scale,
        )? {
            return Ok(meas);
        }

        match &self {
            #[cfg(feature = "contrib")]
            dsl if matches!(dsl, DslPlan::GroupBy { .. }) => {
                group_by::make_private_group_by(
                    input_domain,
                    input_metric,
                    output_measure,
                    self,
                    global_scale,
                )
            }

            #[cfg(feature = "contrib")]
            dsl if matches!(dsl, DslPlan::Select { .. }) => select::make_private_select(
                input_domain,
                input_metric,
                output_measure,
                self,
                global_scale,
            ),

            dsl => fallible!(
                MakeMeasurement,
                "A step in your query is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                dsl.describe()?,
                get_disabled_features_message()
            )
        }
    }
}

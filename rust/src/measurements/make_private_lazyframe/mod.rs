use polars::prelude::LazyFrame;
use polars_plan::{dsl::Expr, plans::DslPlan};

use group_by::ApproximateMeasure;
use opendp_derive::bootstrap;
use std::fmt::Debug;

use crate::{
    combinators::{make_approximate, BasicCompositionMeasure},
    core::{Function, Measure, Measurement, Metric, MetricSpace},
    domains::{DslPlanDomain, LazyFrameDomain},
    error::Fallible,
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
    metrics::PartitionDistance,
    polars::{get_disabled_features_message, OnceFrame},
    transformations::{traits::UnboundedMetric, DatasetMetric, StableDslPlan},
};

use super::PrivateExpr;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod group_by;
pub(crate) use group_by::{is_threshold_predicate, match_group_by, KeySanitizer, MatchGroupBy};

#[cfg(feature = "contrib")]
mod select;

fn make_private_aggregation<MS, MI, MO>(
    input_domain: DslPlanDomain,
    input_metric: MS,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<DslPlanDomain, DslPlan, MS, MO>>
where
    MS: 'static + UnboundedMetric + DatasetMetric,
    MI: Metric,
    MO: 'static + ApproximateMeasure,
    MO::Distance: Debug,
    Expr: PrivateExpr<PartitionDistance<MS>, MO>,
    DslPlan: StableDslPlan<MS, MS>,
{
    #[cfg(feature = "contrib")]
    if group_by::match_group_by(plan.clone())?.is_some() {
        return group_by::make_private_group_by(
            input_domain,
            input_metric,
            output_measure,
            plan,
            global_scale,
            threshold,
        );
    }
    match plan {
        #[cfg(feature = "contrib")]
        plan if matches!(plan, DslPlan::Select { .. }) => select::make_private_select(
            input_domain,
            input_metric,
            output_measure,
            plan,
            global_scale,
        ),

        plan => fallible!(
            MakeMeasurement,
            "A step in your query is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
            plan.describe()?,
            get_disabled_features_message()
        )
    }
}

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        global_scale(rust_type = "Option<f64>", c_type = "AnyObject *", default = b"null"),
        threshold(rust_type = "Option<u32>", c_type = "AnyObject *", default = b"null")
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
/// * `global_scale` - Optional. A tune-able parameter that affects the privacy-utility tradeoff.
/// * `threshold` - Optional. Minimum number of rows in each released partition.
pub fn make_private_lazyframe<MI: Metric, MO: 'static + Measure>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    output_measure: MO,
    lazyframe: LazyFrame,
    global_scale: Option<f64>,
    threshold: Option<u32>,
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
        threshold,
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
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, DslPlan, MI, MO>>;
}

const SORT_ERR_MSG: &'static str = "Found sort in query plan. To conceal row ordering in the original dataset, the output dataset is shuffled. Therefore, sorting the data before release (that shuffles) is wasted computation.";

impl<MS> PrivateDslPlan<MS, MaxDivergence> for DslPlan
where
    MS: 'static + UnboundedMetric,
    DslPlan: StableDslPlan<MS, MS>,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: MS,
        output_measure: MaxDivergence,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, DslPlan, MS, MaxDivergence>> {
        if matches!(self, DslPlan::Sort { .. }) {
            return fallible!(MakeMeasurement, "{}", SORT_ERR_MSG);
        }

        make_private_aggregation::<MS, MS, _>(
            input_domain,
            input_metric,
            output_measure,
            self,
            global_scale,
            threshold,
        )
    }
}

impl<MS> PrivateDslPlan<MS, ZeroConcentratedDivergence> for DslPlan
where
    MS: 'static + UnboundedMetric,
    DslPlan: StableDslPlan<MS, MS>,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: MS,
        output_measure: ZeroConcentratedDivergence,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, DslPlan, MS, ZeroConcentratedDivergence>> {
        if matches!(self, DslPlan::Sort { .. }) {
            return fallible!(MakeMeasurement, "{}", SORT_ERR_MSG);
        }

        make_private_aggregation::<MS, MS, _>(
            input_domain,
            input_metric,
            output_measure,
            self,
            global_scale,
            threshold,
        )
    }
}

impl<MS, MO> PrivateDslPlan<MS, Approximate<MO>> for DslPlan
where
    MS: 'static + UnboundedMetric,
    MO: 'static + BasicCompositionMeasure,
    Approximate<MO>: 'static + ApproximateMeasure,
    <Approximate<MO> as Measure>::Distance: Debug,
    Expr: PrivateExpr<PartitionDistance<MS>, Approximate<MO>>,
    DslPlan: StableDslPlan<MS, MS> + PrivateDslPlan<MS, MO>,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: MS,
        output_measure: Approximate<MO>,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, DslPlan, MS, Approximate<MO>>> {
        if matches!(self, DslPlan::Sort { .. }) {
            return fallible!(MakeMeasurement, "{}", SORT_ERR_MSG);
        }

        if let Ok(meas) = make_private_aggregation::<MS, MS, _>(
            input_domain.clone(),
            input_metric.clone(),
            output_measure.clone(),
            self.clone(),
            global_scale,
            threshold,
        ) {
            return Ok(meas);
        }

        make_approximate(self.make_private(
            input_domain,
            input_metric,
            output_measure.0,
            global_scale,
            threshold,
        )?)
    }
}

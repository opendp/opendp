use polars::prelude::DslPlan;
use polars::prelude::LazyFrame;
use polars_plan::dsl::Expr;

use group_by::ApproximateMeasure;
use opendp_derive::bootstrap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

use crate::{
    combinators::{CompositionMeasure, make_approximate},
    core::{Function, Measure, Measurement, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{Database, DatabaseDomain, DslPlanDomain, Invariant, LazyFrameDomain, Margin},
    error::Fallible,
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, DatabaseIdDistance, FrameDistance,
        HammingDistance, L01InfDistance, MicrodataMetric, PolarsMetric, SymmetricDistance,
        SymmetricIdDistance,
    },
    polars::{OnceFrame, get_disabled_features_message},
    transformations::{
        StableDatabaseDslPlan, StableDslPlan,
        traits::{BoundedMetric, UnboundedMetric},
        StableExpr,
    },
};

use super::PrivateExpr;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod group_by;
pub(crate) use group_by::{KeySanitizer, MatchGroupBy, is_threshold_predicate, match_group_by};

#[cfg(feature = "contrib")]
mod postprocess;

#[cfg(feature = "contrib")]
mod select;

fn make_private_aggregation<MI, MO>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<MI>,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<DslPlanDomain, FrameDistance<MI>, MO, DslPlan>>
where
    MI: 'static + UnboundedMetric + PolarsMetric,
    MI::EventMetric: UnboundedMetric + PolarsMetric,
    MO: 'static + ApproximateMeasure,
    MO::Distance: Debug,
    Expr: PrivateExpr<L01InfDistance<MI::EventMetric>, MO>,
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MI::EventMetric>>,
{
    #[cfg(feature = "contrib")]
    if group_by::match_group_by(plan.clone())?.is_some() {
        return group_by::make_private_group_by::<MI, MO>(
            input_domain,
            input_metric,
            output_measure,
            plan,
            global_scale,
            threshold,
        );
    }
    match plan {
        DslPlan::IR { dsl, .. } => {
            make_private_aggregation::<MI, _>(
                input_domain,
                input_metric,
                output_measure,
                dsl.as_ref().clone(),
                global_scale,
                threshold,
            )
        }
        #[cfg(feature = "contrib")]
        plan if matches!(plan, DslPlan::Select { .. }) => select::make_private_select::<MI, MO>(
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
        ),
    }
}

fn make_private_database_dslplan<MO>(
    input_domain: DatabaseDomain,
    input_metric: DatabaseIdDistance,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<DatabaseDomain, DatabaseIdDistance, MO, DslPlan>>
where
    MO: 'static + CompositionMeasure + group_by::ApproximateMeasure,
    Expr: PrivateExpr<L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>, MO>
        + StableExpr<
            L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>,
            L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>,
        >,
    DslPlan: StableDatabaseDslPlan<DatabaseIdDistance, FrameDistance<SymmetricDistance>>,
    (DatabaseDomain, DatabaseIdDistance): MetricSpace,
{
    match &plan {
        DslPlan::IR { dsl, .. } => {
            return make_private_database_dslplan(
                input_domain,
                input_metric,
                output_measure,
                dsl.as_ref().clone(),
                global_scale,
                threshold,
            );
        }
        DslPlan::Sort { .. } => {
            return fallible!(MakeMeasurement, "{}", SORT_ERR_MSG);
        }
        #[cfg(feature = "contrib")]
        DslPlan::Select {
            expr: exprs, input, ..
        }
        | DslPlan::HStack { input, exprs, .. }
            if exprs
                .iter()
                .all(|e| e.clone().meta().is_column_selection(true)) =>
        {
            let m_in = make_private_database_dslplan(
                input_domain,
                input_metric,
                output_measure.clone(),
                input.as_ref().clone(),
                global_scale,
                threshold,
            )?;

            let post = match plan {
                DslPlan::Select { expr, options, .. } => Function::new_fallible(move |arg: &DslPlan| {
                    Ok(DslPlan::Select {
                        input: Arc::new(arg.clone()),
                        expr: expr.clone(),
                        options,
                    })
                }),
                DslPlan::HStack { exprs, options, .. } => Function::new_fallible(move |arg: &DslPlan| {
                    Ok(DslPlan::HStack {
                        input: Arc::new(arg.clone()),
                        exprs: exprs.clone(),
                        options,
                    })
                }),
                _ => unreachable!(),
            };
            return m_in >> post;
        }
        #[cfg(feature = "contrib")]
        _ if match_group_by(plan.clone())?.is_some() => {}
        DslPlan::Select { .. } => {}
        _ => {
            return fallible!(
                MakeMeasurement,
                "A step in your query is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                plan.describe()?,
                get_disabled_features_message()
            );
        }
    }

    let input = {
        #[cfg(feature = "contrib")]
        if let Some(matched) = match_group_by(plan.clone())? {
            matched.input
        } else {
            match &plan {
                DslPlan::Select { input, .. } => input.as_ref().clone(),
                _ => unreachable!(),
            }
        }
        #[cfg(not(feature = "contrib"))]
        match &plan {
            DslPlan::Select { input, .. } => input.as_ref().clone(),
            _ => unreachable!(),
        }
    };
    let t_prior = input.make_stable_database(input_domain, input_metric)?;

    #[cfg(feature = "contrib")]
    if match_group_by(plan.clone())?.is_some() {
        return group_by::make_private_group_by_with_prior::<
            DatabaseDomain,
            DatabaseIdDistance,
            SymmetricIdDistance,
            MO,
        >(
            t_prior,
            output_measure,
            plan,
            global_scale,
            threshold,
        );
    }

    select::make_private_select_with_prior::<
        DatabaseDomain,
        DatabaseIdDistance,
        SymmetricIdDistance,
        MO,
    >(t_prior, output_measure, plan, global_scale)
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
pub fn make_private_lazyframe<MI: Metric, MO: 'static + Measure>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    output_measure: MO,
    lazyframe: LazyFrame,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<LazyFrameDomain, MI, MO, OnceFrame>>
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
        m_lp.input_metric.clone(),
        m_lp.output_measure.clone(),
        Function::new_fallible(move |arg: &LazyFrame| {
            let lf = LazyFrame::from(f_lp.eval(&arg.logical_plan)?)
                .with_optimizations(arg.get_current_optimizations());
            Ok(OnceFrame::from(lf))
        }),
        m_lp.privacy_map.clone(),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        global_scale(rust_type = "Option<f64>", c_type = "AnyObject *", default = b"null"),
        threshold(rust_type = "Option<u32>", c_type = "AnyObject *", default = b"null")
    ),
    generics(MO(suppress))
)]
pub fn make_private_database_lazyframe<MO: 'static + Measure>(
    input_domain: DatabaseDomain,
    input_metric: DatabaseIdDistance,
    output_measure: MO,
    lazyframe: LazyFrame,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<DatabaseDomain, DatabaseIdDistance, MO, OnceFrame>>
where
    MO: CompositionMeasure + group_by::ApproximateMeasure,
    Expr: PrivateExpr<L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>, MO>
        + StableExpr<
            L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>,
            L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>,
        >,
    DslPlan: StableDatabaseDslPlan<DatabaseIdDistance, FrameDistance<SymmetricDistance>>,
    (DatabaseDomain, DatabaseIdDistance): MetricSpace,
{
    let m_lp = make_private_database_dslplan(
        input_domain,
        input_metric,
        output_measure,
        lazyframe.logical_plan,
        global_scale,
        threshold,
    )?;
    let f_lp = m_lp.function.clone();

    Measurement::new(
        m_lp.input_domain.clone(),
        m_lp.input_metric.clone(),
        m_lp.output_measure.clone(),
        Function::new_fallible(move |arg: &Database| Ok(LazyFrame::from(f_lp.eval(arg)?).into())),
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
    ) -> Fallible<Measurement<DslPlanDomain, MI, MO, DslPlan>>;
}

const SORT_ERR_MSG: &'static str = "Found sort in query plan. To conceal row ordering in the original dataset, the output dataset is shuffled. Therefore, sorting the data before release (that shuffles) is wasted computation.";

impl<MI> PrivateDslPlan<FrameDistance<MI>, MaxDivergence> for DslPlan
where
    MI: UnboundedMetric + PolarsMetric,
    MI::EventMetric: UnboundedMetric + PolarsMetric,
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MI::EventMetric>>,
    (DslPlanDomain, FrameDistance<MI>): MetricSpace,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: FrameDistance<MI>,
        output_measure: MaxDivergence,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, FrameDistance<MI>, MaxDivergence, DslPlan>> {
        if matches!(self, DslPlan::Sort { .. }) {
            return fallible!(MakeMeasurement, "{}", SORT_ERR_MSG);
        }

        if let Some(meas) = postprocess::match_postprocess(
            input_domain.clone(),
            input_metric.clone(),
            output_measure.clone(),
            self.clone(),
            global_scale,
            threshold,
        )? {
            return Ok(meas);
        }

        make_private_aggregation::<MI, _>(
            input_domain,
            input_metric,
            output_measure,
            self,
            global_scale,
            threshold,
        )
    }
}

impl<MI> PrivateDslPlan<FrameDistance<MI>, ZeroConcentratedDivergence> for DslPlan
where
    MI: UnboundedMetric + PolarsMetric,
    MI::EventMetric: UnboundedMetric + PolarsMetric,
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MI::EventMetric>>,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: FrameDistance<MI>,
        output_measure: ZeroConcentratedDivergence,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, FrameDistance<MI>, ZeroConcentratedDivergence, DslPlan>>
    {
        if matches!(self, DslPlan::Sort { .. }) {
            return fallible!(MakeMeasurement, "{}", SORT_ERR_MSG);
        }

        if let Some(meas) = postprocess::match_postprocess(
            input_domain.clone(),
            input_metric.clone(),
            output_measure.clone(),
            self.clone(),
            global_scale,
            threshold,
        )? {
            return Ok(meas);
        }

        make_private_aggregation::<MI, _>(
            input_domain,
            input_metric,
            output_measure,
            self,
            global_scale,
            threshold,
        )
    }
}

impl<MI, MO> PrivateDslPlan<FrameDistance<MI>, Approximate<MO>> for DslPlan
where
    MI: UnboundedMetric + PolarsMetric,
    MI::EventMetric: UnboundedMetric + PolarsMetric,
    MO: 'static + CompositionMeasure,
    Approximate<MO>: 'static + ApproximateMeasure,
    <Approximate<MO> as Measure>::Distance: Debug,
    Expr: PrivateExpr<L01InfDistance<MI::EventMetric>, Approximate<MO>>,
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MI::EventMetric>>
        + PrivateDslPlan<FrameDistance<MI::EventMetric>, MO>
        + PrivateDslPlan<FrameDistance<MI>, MO>,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: FrameDistance<MI>,
        output_measure: Approximate<MO>,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, FrameDistance<MI>, Approximate<MO>, DslPlan>> {
        if matches!(self, DslPlan::Sort { .. }) {
            return fallible!(MakeMeasurement, "{}", SORT_ERR_MSG);
        }

        if let Some(meas) = postprocess::match_postprocess::<FrameDistance<MI>, Approximate<MO>>(
            input_domain.clone(),
            input_metric.clone(),
            output_measure.clone(),
            self.clone(),
            global_scale,
            threshold,
        )? {
            return Ok(meas);
        }

        if let Ok(meas) = make_private_aggregation::<MI, _>(
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

impl<MI, MO> PrivateDslPlan<MI, MO> for DslPlan
where
    MI: 'static + UnboundedMetric,
    MO: 'static + Measure,
    DslPlan: PrivateDslPlan<FrameDistance<MI>, MO>,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: MI,
        output_measure: MO,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, MI, MO, DslPlan>> {
        Transformation::new(
            input_domain.clone(),
            input_metric.clone(),
            input_domain.clone(),
            FrameDistance(input_metric.clone()),
            Function::new(Clone::clone),
            StabilityMap::new(|&d_in: &u32| d_in.into()),
        ) >> self.make_private(
            input_domain,
            FrameDistance(input_metric),
            output_measure,
            global_scale,
            threshold,
        )?
    }
}

macro_rules! impl_plan_bounded_dp {
    ($ty:ty) => {
        impl<MO: 'static + Measure> PrivateDslPlan<$ty, MO> for DslPlan
        where
            DslPlan: PrivateDslPlan<<$ty as BoundedMetric>::UnboundedMetric, MO>,
        {
            fn make_private(
                self,
                input_domain: DslPlanDomain,
                input_metric: $ty,
                output_measure: MO,
                global_scale: Option<f64>,
                threshold: Option<u32>,
            ) -> Fallible<Measurement<DslPlanDomain, $ty, MO, DslPlan>> {
                let mut middle_domain = input_domain.clone();
                if let Some(prev_margin) = middle_domain
                    .margins
                    .iter_mut()
                    .find(|m| m.by == HashSet::new())
                {
                    prev_margin.invariant = Some(Invariant::Lengths);
                } else {
                    middle_domain
                        .margins
                        .push(Margin::select().with_invariant_lengths());
                }
                let middle_metric = input_metric.to_unbounded();

                Transformation::new(
                    input_domain.clone(),
                    input_metric.clone(),
                    middle_domain.clone(),
                    middle_metric.clone(),
                    Function::new(Clone::clone),
                    StabilityMap::new_from_constant(2),
                )? >> self.make_private(
                    middle_domain,
                    middle_metric,
                    output_measure,
                    global_scale,
                    threshold,
                )?
            }
        }
    };
}

impl_plan_bounded_dp!(HammingDistance);
impl_plan_bounded_dp!(ChangeOneDistance);

impl<MO: 'static + Measure> PrivateDslPlan<ChangeOneIdDistance, MO> for DslPlan
where
    DslPlan: PrivateDslPlan<<ChangeOneIdDistance as BoundedMetric>::UnboundedMetric, MO>,
{
    fn make_private(
        self,
        input_domain: DslPlanDomain,
        input_metric: ChangeOneIdDistance,
        output_measure: MO,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<Measurement<DslPlanDomain, ChangeOneIdDistance, MO, DslPlan>> {
        let middle_metric = input_metric.to_unbounded();
        Transformation::new(
            input_domain.clone(),
            input_metric.clone(),
            input_domain.clone(),
            middle_metric.clone(),
            Function::new(Clone::clone),
            StabilityMap::new_from_constant(2),
        )? >> self.make_private(
            input_domain,
            middle_metric,
            output_measure,
            global_scale,
            threshold,
        )?
    }
}

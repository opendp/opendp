use opendp_derive::bootstrap;
use polars::{lazy::frame::LazyFrame, prelude::DslPlan};
use std::collections::HashSet;

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{Database, DatabaseDomain, DslPlanDomain, Invariant, LazyFrameDomain, Margin},
    error::Fallible,
    metrics::{
        filter_id_sites, unique_id_expr, Bounds, ChangeOneDistance, ChangeOneIdDistance,
        DatabaseIdDistance, FrameDistance, HammingDistance, PolarsMetric, SymmetricDistance,
        SymmetricIdDistance,
    },
    polars::get_disabled_features_message,
};

use super::traits::{BoundedMetric, UnboundedMetric};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod filter;

#[cfg(feature = "contrib")]
mod group_by;

#[cfg(feature = "contrib")]
mod h_stack;

#[cfg(feature = "contrib")]
mod join;

#[cfg(feature = "contrib")]
pub(crate) mod select;

#[cfg(feature = "contrib")]
pub(crate) mod source;

#[cfg(feature = "contrib")]
pub(crate) mod truncate;

#[bootstrap(
    features("contrib"),
    arguments(output_metric(c_type = "AnyMetric *", rust_type = b"null")),
    generics(MI(suppress), MO(default = "MI"))
)]
pub fn make_stable_lazyframe<MI: 'static + Metric, MO: 'static + Metric>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    lazyframe: LazyFrame,
) -> Fallible<Transformation<LazyFrameDomain, MI, LazyFrameDomain, MO>>
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
        t_input.input_metric.clone(),
        t_input.output_domain.cast_carrier(),
        t_input.output_metric.clone(),
        Function::new_fallible(move |arg: &LazyFrame| {
            Ok(LazyFrame::from(f_input.eval(&arg.logical_plan)?)
                .with_optimizations(arg.get_current_optimizations()))
        }),
        t_input.stability_map.clone(),
    )
}

pub fn make_stable_database_lazyframe(
    input_domain: DatabaseDomain,
    input_metric: DatabaseIdDistance,
    lazyframe: LazyFrame,
) -> Fallible<
    Transformation<
        DatabaseDomain,
        DatabaseIdDistance,
        LazyFrameDomain,
        FrameDistance<SymmetricIdDistance>,
    >,
>
where
    DslPlan: StableDatabaseDslPlan<DatabaseIdDistance, FrameDistance<SymmetricIdDistance>>,
    (DatabaseDomain, DatabaseIdDistance): MetricSpace,
{
    let t_input: Transformation<
        DatabaseDomain,
        DatabaseIdDistance,
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
    > = lazyframe
        .logical_plan
        .make_stable_database(input_domain, input_metric)?;
    let f_input = t_input.function_clone();
    let m_input = t_input.stability_map_clone();

    Transformation::new(
        t_input.input_domain.clone(),
        t_input.input_metric.clone(),
        t_input.output_domain.cast_carrier(),
        t_input.output_metric.clone(),
        Function::new_fallible(move |arg: &Database| Ok(LazyFrame::from(f_input.eval(arg)?))),
        m_input,
    )
}

pub trait StableDslPlan<MI: Metric, MO: Metric> {
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: MI,
    ) -> Fallible<Transformation<DslPlanDomain, MI, DslPlanDomain, MO>>;
}

pub trait StableDatabaseDslPlan<MI: Metric, MO: Metric> {
    fn make_stable_database(
        self,
        input_domain: DatabaseDomain,
        input_metric: MI,
    ) -> Fallible<Transformation<DatabaseDomain, MI, DslPlanDomain, MO>>;
}

pub(crate) fn table_metric(
    input_metric: &DatabaseIdDistance,
    table_name: &str,
) -> SymmetricIdDistance {
    SymmetricIdDistance {
        protected_label: input_metric.protected_label.clone(),
        id_sites: input_metric
            .id_sites
            .get(table_name)
            .cloned()
            .unwrap_or_default(),
    }
}

pub(crate) fn database_metric(input_metric: &DatabaseIdDistance) -> SymmetricIdDistance {
    SymmetricIdDistance {
        protected_label: input_metric.protected_label.clone(),
        id_sites: input_metric
            .id_sites
            .values()
            .flat_map(|sites| filter_id_sites(sites, &input_metric.protected_label))
            .collect(),
    }
}

impl StableDslPlan<FrameDistance<SymmetricIdDistance>, FrameDistance<SymmetricDistance>>
    for DslPlan
{
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: FrameDistance<SymmetricIdDistance>,
    ) -> Fallible<
        Transformation<
            DslPlanDomain,
            FrameDistance<SymmetricIdDistance>,
            DslPlanDomain,
            FrameDistance<SymmetricDistance>,
        >,
    > {
        if let DslPlan::IR { dsl, .. } = self {
            return dsl.as_ref().clone().make_stable(input_domain, input_metric);
        }

        let identifier = unique_id_expr(&input_metric.0.active_id_sites())?
            .ok_or_else(|| err!(MakeTransformation, "truncation requires at least one identifier"))?;
        let truncations = truncate::match_truncations(self.clone(), &identifier)?.1;
        if !truncations.is_empty() {
            return truncate::make_stable_truncate(input_domain, input_metric, self);
        }

        fallible!(
            MakeTransformation,
            "queries with identifier metrics require an explicit truncation before converting to event-level stability"
        )
    }
}

impl<M: UnboundedMetric + PolarsMetric> StableDslPlan<FrameDistance<M>, FrameDistance<M>> for DslPlan {
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: FrameDistance<M>,
    ) -> Fallible<Transformation<DslPlanDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>>
    {
        match &self {
            DslPlan::IR { dsl, .. } => (**dsl).clone().make_stable(input_domain, input_metric),
            DslPlan::DataFrameScan { .. } => {
                source::make_stable_source(input_domain, input_metric, self)
            }
            DslPlan::GroupBy { .. } => {
                group_by::make_stable_group_by(input_domain, input_metric, self)
            }
            DslPlan::Filter { .. } => filter::make_stable_filter(input_domain, input_metric, self),
            DslPlan::HStack { .. } => h_stack::make_h_stack(input_domain, input_metric, self),
            DslPlan::Select { .. } => select::make_select(input_domain, input_metric, self),
            dsl => match dsl.describe() {
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
                ),
            },
        }
    }
}

impl StableDatabaseDslPlan<DatabaseIdDistance, FrameDistance<SymmetricDistance>> for DslPlan {
    fn make_stable_database(
        self,
        input_domain: DatabaseDomain,
        input_metric: DatabaseIdDistance,
    ) -> Fallible<
        Transformation<
            DatabaseDomain,
            DatabaseIdDistance,
            DslPlanDomain,
            FrameDistance<SymmetricDistance>,
        >,
    > {
        let plan = match self {
            DslPlan::IR { dsl, .. } => return dsl.as_ref().clone().make_stable_database(input_domain, input_metric),
            plan => plan,
        };

        let identifier = unique_id_expr(&database_metric(&input_metric).active_id_sites())?
            .ok_or_else(|| err!(MakeTransformation, "truncation requires at least one identifier"))?;
        let (input, truncations, _) = truncate::match_truncations(plan.clone(), &identifier)?;

        if truncations.is_empty() {
            return fallible!(
                MakeTransformation,
                "queries with identifier metrics require an explicit truncation before converting to event-level stability"
            );
        }

        let t_prior = input.make_stable_database(input_domain, input_metric)?;
        truncate::make_chain_truncate(t_prior, plan)
    }
}

impl<MI: UnboundedMetric, MO: UnboundedMetric> StableDslPlan<MI, FrameDistance<MO>> for DslPlan
where
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MO>>,
{
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: MI,
    ) -> Fallible<Transformation<DslPlanDomain, MI, DslPlanDomain, FrameDistance<MO>>> {
        Transformation::new(
            input_domain.clone(),
            input_metric.clone(),
            input_domain.clone(),
            FrameDistance(input_metric.clone()),
            Function::new(Clone::clone),
            StabilityMap::new(|&d_in| Bounds::from(d_in)),
        )? >> self.make_stable(input_domain, FrameDistance(input_metric))?
    }
}

macro_rules! impl_plan_bounded_dp {
    ($ty:ty) => {
        impl<MO: UnboundedMetric> StableDslPlan<$ty, FrameDistance<MO>> for DslPlan
        where
            DslPlan: StableDslPlan<<$ty as BoundedMetric>::UnboundedMetric, FrameDistance<MO>>,
        {
            fn make_stable(
                self,
                input_domain: DslPlanDomain,
                input_metric: $ty,
            ) -> Fallible<Transformation<DslPlanDomain, $ty, DslPlanDomain, FrameDistance<MO>>>
            {
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
                )? >> self.make_stable(middle_domain, middle_metric)?
            }
        }
    };
}

impl_plan_bounded_dp!(HammingDistance);
impl_plan_bounded_dp!(ChangeOneDistance);

impl<MO: UnboundedMetric> StableDslPlan<ChangeOneIdDistance, FrameDistance<MO>> for DslPlan
where
    DslPlan:
        StableDslPlan<<ChangeOneIdDistance as BoundedMetric>::UnboundedMetric, FrameDistance<MO>>,
{
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: ChangeOneIdDistance,
    ) -> Fallible<
        Transformation<DslPlanDomain, ChangeOneIdDistance, DslPlanDomain, FrameDistance<MO>>,
    > {
        Transformation::new(
            input_domain.clone(),
            input_metric.clone(),
            input_domain.clone(),
            input_metric.to_unbounded(),
            Function::new(Clone::clone),
            StabilityMap::new_from_constant(2),
        )? >> self.make_stable(input_domain, input_metric.to_unbounded())?
    }
}

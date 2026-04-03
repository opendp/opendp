use opendp_derive::bootstrap;
use polars::{lazy::frame::LazyFrame, prelude::DslPlan};
use std::collections::HashSet;

use crate::{
    core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{Database, DatabaseDomain, DslPlanDomain, Invariant, LazyFrameDomain, Margin},
    error::Fallible,
    metrics::{
        Bounds, ChangeOneDistance, ChangeOneIdDistance, DatabaseIdDistance, FrameDistance,
        HammingDistance, PolarsMetric, SymmetricDistance, SymmetricIdDistance, filter_bindings,
        normalize_claims,
    },
    polars::get_disabled_features_message,
    transformations::{
        make_stable_lazyframe::{
            filter::make_chain_filter, group_by::make_chain_group_by, h_stack::make_chain_h_stack,
            join::make_stable_database_join,
        },
        select::make_chain_select,
        source::make_stable_database_source,
    },
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
    DslPlan: StableDslPlan<DslPlanDomain, MI, MO>,
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
    DslPlan: StableDslPlan<DatabaseDomain, DatabaseIdDistance, FrameDistance<SymmetricIdDistance>>,
    (DatabaseDomain, DatabaseIdDistance): MetricSpace,
{
    let t_input: Transformation<
        DatabaseDomain,
        DatabaseIdDistance,
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
    > = lazyframe
        .logical_plan
        .make_stable(input_domain, input_metric)?;
    let f_input = t_input.function.clone();
    let m_input = t_input.stability_map.clone();

    Transformation::new(
        t_input.input_domain.clone(),
        t_input.input_metric.clone(),
        t_input.output_domain.cast_carrier(),
        t_input.output_metric.clone(),
        Function::new_fallible(move |arg: &Database| Ok(LazyFrame::from(f_input.eval(arg)?))),
        m_input,
    )
}

pub trait StableDslPlan<DI: Domain, MI: Metric, MO: Metric> {
    fn make_stable(
        self,
        input_domain: DI,
        input_metric: MI,
    ) -> Fallible<Transformation<DI, MI, DslPlanDomain, MO>>;
}

trait StableDslPlanInput<MI: Metric, MO: Metric>: Domain {
    fn make_stable_scan(
        input_domain: Self,
        input_metric: MI,
        plan: DslPlan,
    ) -> Fallible<Transformation<Self, MI, DslPlanDomain, MO>>;
    fn make_stable_join(
        input_domain: Self,
        input_metric: MI,
        plan: DslPlan,
    ) -> Fallible<Transformation<Self, MI, DslPlanDomain, MO>>;
}

pub(crate) fn database_metric(input_metric: &DatabaseIdDistance) -> SymmetricIdDistance {
    SymmetricIdDistance {
        protect: input_metric.protect.clone(),
        bindings: input_metric
            .bindings
            .values()
            .flat_map(|sites| filter_bindings(sites, &input_metric.protect))
            .collect(),
        owner_claims: normalize_claims(
            &input_metric
                .base_owner_claims
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<_>>(),
        ),
    }
}

fn make_stable_dslplan<DI, MI, M>(
    plan: DslPlan,
    input_domain: DI,
    input_metric: MI,
) -> Fallible<Transformation<DI, MI, DslPlanDomain, FrameDistance<M>>>
where
    DI: Domain + Clone + 'static,
    MI: Metric + Clone + 'static,
    M: UnboundedMetric + PolarsMetric,
    DslPlan: StableDslPlan<DI, MI, FrameDistance<M>>,
    (DI, MI): MetricSpace,
    (DslPlanDomain, FrameDistance<M>): MetricSpace,
    DI: StableDslPlanInput<MI, FrameDistance<M>>,
{
    match plan {
        DslPlan::IR { dsl, .. } => {
            <DslPlan as StableDslPlan<DI, MI, FrameDistance<M>>>::make_stable(
                dsl.as_ref().clone(),
                input_domain,
                input_metric,
            )
        }
        plan @ DslPlan::DataFrameScan { .. } => {
            DI::make_stable_scan(input_domain, input_metric, plan)
        }
        #[cfg(patch_polars)]
        DslPlan::GroupBy {
            input,
            keys,
            predicates,
            aggs,
            apply,
            maintain_order,
            options,
        } => {
            if !predicates.is_empty() {
                return fallible!(
                    MakeTransformation,
                    "Having is not currently supported in logical plan. Please open an issue if this would be useful to you."
                );
            }
            let t_prior = input
                .as_ref()
                .clone()
                .make_stable(input_domain, input_metric)?;
            make_chain_group_by(
                t_prior,
                keys,
                aggs,
                apply,
                maintain_order,
                options,
            )
        }
        #[cfg(not(patch_polars))]
        DslPlan::GroupBy {
            input,
            keys,
            aggs,
            apply,
            maintain_order,
            options,
        } => {
            let t_prior = input
                .as_ref()
                .clone()
                .make_stable(input_domain, input_metric)?;
            make_chain_group_by(
                t_prior,
                keys,
                predicates,
                aggs,
                apply,
                maintain_order,
                options,
            )
        }
        DslPlan::Filter { input, predicate } => {
            let t_prior = input
                .as_ref()
                .clone()
                .make_stable(input_domain, input_metric)?;
            make_chain_filter(t_prior, predicate)
        }
        DslPlan::HStack {
            input,
            exprs,
            options,
        } => {
            let t_prior = input
                .as_ref()
                .clone()
                .make_stable(input_domain, input_metric)?;
            make_chain_h_stack(t_prior, exprs, options)
        }
        DslPlan::Select {
            input,
            expr,
            options,
        } => {
            let t_prior = input
                .as_ref()
                .clone()
                .make_stable(input_domain, input_metric)?;
            make_chain_select(t_prior, expr, options)
        }
        plan @ DslPlan::Join { .. } => DI::make_stable_join(input_domain, input_metric, plan),
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

impl<M: UnboundedMetric + PolarsMetric> StableDslPlanInput<FrameDistance<M>, FrameDistance<M>>
    for DslPlanDomain
{
    fn make_stable_scan(
        input_domain: DslPlanDomain,
        input_metric: FrameDistance<M>,
        plan: DslPlan,
    ) -> Fallible<Transformation<DslPlanDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>>
    {
        source::make_stable_source(input_domain, input_metric, plan)
    }
    fn make_stable_join(
        _input_domain: DslPlanDomain,
        _input_metric: FrameDistance<M>,
        _plan: DslPlan,
    ) -> Fallible<Transformation<DslPlanDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>>
    {
        fallible!(
            MakeTransformation,
            "joins are only supported on database contexts"
        )
    }
}

impl StableDslPlanInput<DatabaseIdDistance, FrameDistance<SymmetricIdDistance>> for DatabaseDomain {
    fn make_stable_scan(
        input_domain: DatabaseDomain,
        input_metric: DatabaseIdDistance,
        plan: DslPlan,
    ) -> Fallible<
        Transformation<
            DatabaseDomain,
            DatabaseIdDistance,
            DslPlanDomain,
            FrameDistance<SymmetricIdDistance>,
        >,
    > {
        make_stable_database_source(input_domain, input_metric, plan)
    }
    fn make_stable_join(
        input_domain: DatabaseDomain,
        input_metric: DatabaseIdDistance,
        plan: DslPlan,
    ) -> Fallible<
        Transformation<
            DatabaseDomain,
            DatabaseIdDistance,
            DslPlanDomain,
            FrameDistance<SymmetricIdDistance>,
        >,
    > {
        make_stable_database_join(input_domain, input_metric, plan)
    }
}

impl
    StableDslPlan<
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
        FrameDistance<SymmetricDistance>,
    > for DslPlan
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

        if truncate::find_truncation_claim(&input_metric.0, &self)?.is_some() {
            return truncate::make_stable_truncate(input_domain, input_metric, self);
        }
        if truncate::has_binding_matched_truncation(&input_metric.0, &self)? {
            return fallible!(
                MakeTransformation,
                "truncation currently requires a compatible single-factor owner claim"
            );
        }

        fallible!(
            MakeTransformation,
            "queries with identifier metrics require an explicit truncation before converting to event-level stability"
        )
    }
}

impl<M: UnboundedMetric + PolarsMetric>
    StableDslPlan<DslPlanDomain, FrameDistance<M>, FrameDistance<M>> for DslPlan
{
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: FrameDistance<M>,
    ) -> Fallible<Transformation<DslPlanDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>>
    {
        make_stable_dslplan(self, input_domain, input_metric)
    }
}

impl StableDslPlan<DatabaseDomain, DatabaseIdDistance, FrameDistance<SymmetricIdDistance>>
    for DslPlan
{
    fn make_stable(
        self,
        input_domain: DatabaseDomain,
        input_metric: DatabaseIdDistance,
    ) -> Fallible<
        Transformation<
            DatabaseDomain,
            DatabaseIdDistance,
            DslPlanDomain,
            FrameDistance<SymmetricIdDistance>,
        >,
    > {
        let plan = match self {
            DslPlan::IR { dsl, .. } => {
                return (*dsl).clone().make_stable(input_domain, input_metric);
            }
            plan => plan,
        };

        make_stable_dslplan(plan, input_domain, input_metric)
    }
}

impl StableDslPlan<DatabaseDomain, DatabaseIdDistance, FrameDistance<SymmetricDistance>>
    for DslPlan
{
    fn make_stable(
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
            DslPlan::IR { dsl, .. } => {
                return dsl.as_ref().clone().make_stable(input_domain, input_metric);
            }
            plan => plan,
        };

        let frame_metric = database_metric(&input_metric);
        let Some(claim) = truncate::find_truncation_claim(&frame_metric, &plan)? else {
            if truncate::has_binding_matched_truncation(&frame_metric, &plan)? {
                return fallible!(
                    MakeTransformation,
                    "truncation currently requires a compatible single-factor owner claim"
                );
            }
            return fallible!(
                MakeTransformation,
                "queries with identifier metrics require an explicit truncation before converting to event-level stability"
            );
        };
        let (input, _truncations, _) = truncate::match_truncations(plan.clone(), &claim[0])?;

        let t_prior = input.make_stable(input_domain, input_metric)?;
        truncate::make_chain_truncate(t_prior, plan)
    }
}

impl<MI: UnboundedMetric, MO: UnboundedMetric> StableDslPlan<DslPlanDomain, MI, FrameDistance<MO>>
    for DslPlan
where
    DslPlan: StableDslPlan<DslPlanDomain, FrameDistance<MI>, FrameDistance<MO>>,
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
        impl<MO: UnboundedMetric> StableDslPlan<DslPlanDomain, $ty, FrameDistance<MO>> for DslPlan
        where
            DslPlan: StableDslPlan<
                    DslPlanDomain,
                    <$ty as BoundedMetric>::UnboundedMetric,
                    FrameDistance<MO>,
                >,
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

impl<MO: UnboundedMetric> StableDslPlan<DslPlanDomain, ChangeOneIdDistance, FrameDistance<MO>>
    for DslPlan
where
    DslPlan: StableDslPlan<
            DslPlanDomain,
            <ChangeOneIdDistance as BoundedMetric>::UnboundedMetric,
            FrameDistance<MO>,
        >,
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

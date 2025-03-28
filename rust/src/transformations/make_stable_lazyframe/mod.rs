use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;
use polars_plan::plans::DslPlan;
use std::collections::HashSet;

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{DslPlanDomain, LazyFrameDomain, Margin, MarginPub},
    error::Fallible,
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, GroupBounds, HammingDistance, Multi,
        SymmetricDistance, SymmetricIdDistance,
    },
    polars::get_disabled_features_message,
};

use super::traits::{BoundedMetric, UnboundedMetric};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod filter;

#[cfg(feature = "contrib")]
mod h_stack;

#[cfg(feature = "contrib")]
pub(crate) mod select;

#[cfg(feature = "contrib")]
mod source;

#[cfg(feature = "contrib")]
mod truncate;

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

impl StableDslPlan<Multi<SymmetricIdDistance>, Multi<SymmetricDistance>> for DslPlan {
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: Multi<SymmetricIdDistance>,
    ) -> Fallible<
        Transformation<
            DslPlanDomain,
            DslPlanDomain,
            Multi<SymmetricIdDistance>,
            Multi<SymmetricDistance>,
        >,
    > {
        if let DslPlan::Filter { predicate, .. } = &self {
            if truncate::match_truncate(&predicate, &input_metric.0.identifier).is_some() {
                return truncate::make_stable_truncate(input_domain, input_metric, self);
            }
        }

        match &self {
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

impl<M: UnboundedMetric> StableDslPlan<Multi<M>, Multi<M>> for DslPlan {
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: Multi<M>,
    ) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, Multi<M>, Multi<M>>> {
        match &self {
            DslPlan::DataFrameScan { .. } => {
                source::make_stable_source(input_domain, input_metric, self)
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

impl<M: UnboundedMetric> StableDslPlan<M, Multi<M>> for DslPlan {
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: M,
    ) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, M, Multi<M>>> {
        Transformation::new(
            input_domain.clone(),
            input_domain.clone(),
            Function::new(Clone::clone),
            input_metric.clone(),
            Multi(input_metric.clone()),
            StabilityMap::new(|&d_in| GroupBounds::from(d_in)),
        )? >> self.make_stable(input_domain, Multi(input_metric))?
    }
}

macro_rules! impl_plan_bounded_dp {
    ($ty:ty) => {
        impl<MO: UnboundedMetric> StableDslPlan<$ty, Multi<MO>> for DslPlan
        where
            DslPlan: StableDslPlan<<$ty as BoundedMetric>::UnboundedMetric, Multi<MO>>,
        {
            fn make_stable(
                self,
                input_domain: DslPlanDomain,
                input_metric: $ty,
            ) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, $ty, Multi<MO>>> {
                let mut middle_domain = input_domain.clone();
                if let Some(prev_margin) = middle_domain
                    .margins
                    .iter_mut()
                    .find(|m| m.by == HashSet::new())
                {
                    prev_margin.public_info = Some(MarginPub::Lengths);
                } else {
                    middle_domain
                        .margins
                        .push(Margin::select().with_public_lengths());
                }
                let middle_metric = input_metric.to_unbounded();

                Transformation::new(
                    input_domain.clone(),
                    middle_domain.clone(),
                    Function::new(Clone::clone),
                    input_metric.clone(),
                    middle_metric.clone(),
                    StabilityMap::new_from_constant(2),
                )? >> self.make_stable(middle_domain, middle_metric)?
            }
        }
    };
}

impl_plan_bounded_dp!(HammingDistance);
impl_plan_bounded_dp!(ChangeOneDistance);

impl<MO: UnboundedMetric> StableDslPlan<ChangeOneIdDistance, Multi<MO>> for DslPlan
where
    DslPlan: StableDslPlan<<ChangeOneIdDistance as BoundedMetric>::UnboundedMetric, Multi<MO>>,
{
    fn make_stable(
        self,
        input_domain: DslPlanDomain,
        input_metric: ChangeOneIdDistance,
    ) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, ChangeOneIdDistance, Multi<MO>>>
    {
        Transformation::new(
            input_domain.clone(),
            input_domain.clone(),
            Function::new(Clone::clone),
            input_metric.clone(),
            input_metric.to_unbounded(),
            StabilityMap::new_from_constant(2),
        )? >> self.make_stable(input_domain, input_metric.to_unbounded())?
    }
}

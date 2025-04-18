use polars::{lazy::frame::LazyFrame, prelude::DslPlan};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace},
    domains::{DslPlanDomain, LazyFrameDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
        util::TypeContents,
    },
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, FrameDistance, HammingDistance,
        InsertDeleteDistance, SymmetricDistance, SymmetricIdDistance,
    },
    transformations::{
        StableDslPlan,
        traits::{BoundedMetric, UnboundedMetric},
    },
};

use super::make_stable_lazyframe;

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_stable_lazyframe(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    lazyframe: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let MI = input_metric.type_.clone();

    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();

    fn is_in<T>() -> Option<()> {
        Some(())
    }

    if let TypeContents::GENERIC { .. } = MI.contents {
        fn monomorphize<MI: 'static + Metric>(
            input_domain: LazyFrameDomain,
            input_metric: &AnyMetric,
            lazyframe: LazyFrame,
        ) -> Fallible<AnyTransformation>
        where
            DslPlan: StableDslPlan<MI, MI>,
            (LazyFrameDomain, MI): MetricSpace,
            (DslPlanDomain, MI): MetricSpace,
        {
            let input_metric = input_metric.downcast_ref::<MI>()?.clone();
            make_stable_lazyframe(input_domain, input_metric, lazyframe).into_any()
        }
        dispatch!(
            monomorphize,
            [(MI, [FrameDistance<SymmetricDistance>, FrameDistance<SymmetricIdDistance>, FrameDistance<InsertDeleteDistance>])],
            (
                input_domain,
                input_metric,
                lazyframe
            )
        )
    } else if dispatch!(is_in, [(MI, [SymmetricDistance, SymmetricIdDistance, InsertDeleteDistance])], ()).is_some() {
        fn monomorphize<MI: UnboundedMetric>(
            input_domain: LazyFrameDomain,
            input_metric: &AnyMetric,
            lazyframe: LazyFrame,
        ) -> Fallible<AnyTransformation>
        where
            DslPlan: StableDslPlan<MI, FrameDistance<MI>>,
        {
            let input_metric = input_metric.downcast_ref::<MI>()?.clone();
            make_stable_lazyframe(input_domain, input_metric, lazyframe).into_any()
        }
        dispatch!(
            monomorphize,
            [(MI, [SymmetricDistance, SymmetricIdDistance, InsertDeleteDistance])],
            (
                input_domain,
                input_metric,
                lazyframe
            )
        )
    } else {
        fn monomorphize<MI: BoundedMetric>(
            input_domain: LazyFrameDomain,
            input_metric: &AnyMetric,
            lazyframe: LazyFrame,
        ) -> Fallible<AnyTransformation>
        where
            DslPlan: StableDslPlan<MI, FrameDistance<MI::UnboundedMetric>>,
        {
            let input_metric = input_metric.downcast_ref::<MI>()?.clone();
            make_stable_lazyframe(input_domain, input_metric, lazyframe).into_any()
        }
        dispatch!(
            monomorphize,
            [(MI, [ChangeOneDistance, HammingDistance, ChangeOneIdDistance])],
            (
                input_domain,
                input_metric,
                lazyframe
            )
        )
    }.into()
}

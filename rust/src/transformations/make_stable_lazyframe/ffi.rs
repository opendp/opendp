use std::ffi::c_char;

use polars::{lazy::frame::LazyFrame, prelude::DslPlan};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace},
    domains::{DslPlanDomain, LazyFrameDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
        util::{Type, TypeContents, to_str},
    },
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, FrameDistance, SymmetricDistance,
        SymmetricIdDistance,
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
    MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let MI = input_metric.type_.clone();
    let requested_mo = try_!(to_str(MO));
    let MO = match requested_mo {
        "MI" => MI.clone(),
        v => try_!(Type::try_from(v)),
    };

    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();

    fn is_in<T>() -> Option<()> {
        Some(())
    }

    if let TypeContents::GENERIC { .. } = MI.contents {
        // FrameDistance<MI> -> FrameDistance<MI>
        if MI == MO {
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
                [(MI, [FrameDistance<SymmetricDistance>, FrameDistance<SymmetricIdDistance>])],
                (
                    input_domain,
                    input_metric,
                    lazyframe
                )
            )
        // FrameDistance<SymmetricIdDistance> -> FrameDistance<SymmetricDistance>
        } else if MI == Type::of::<FrameDistance<SymmetricIdDistance>>() && MO == Type::of::<FrameDistance<SymmetricDistance>>() {
            let input_metric = try_!(input_metric.downcast_ref::<FrameDistance<SymmetricIdDistance>>()).clone();
            make_stable_lazyframe::<_, FrameDistance<SymmetricDistance>>(input_domain, input_metric, lazyframe).into_any()
        } else {
            err!(FFI, "MI ({:?}) must match MO ({:?}), or FrameDistance<SymmetricIdDistance> -> FrameDistance<SymmetricDistance>", MI.descriptor, MO.descriptor).into()
        }
    } else if dispatch!(is_in, [(MI, [SymmetricDistance, SymmetricIdDistance])], ()).is_some() {
        // MI -> FrameDistance<MI>
        if MI == try_!(MO.get_atom()) {
            fn monomorphize<MI: UnboundedMetric>(
                input_domain: LazyFrameDomain,
                input_metric: &AnyMetric,
                lazyframe: LazyFrame,
            ) -> Fallible<AnyTransformation>
            where
                DslPlan: StableDslPlan<MI, FrameDistance<MI>> + StableDslPlan<FrameDistance<MI>, FrameDistance<MI>>,
            {
                let input_metric = input_metric.downcast_ref::<MI>()?.clone();
                make_stable_lazyframe(input_domain, input_metric, lazyframe).into_any()
            }
            dispatch!(
                monomorphize,
                [(MI, [SymmetricDistance, SymmetricIdDistance])],
                (
                    input_domain,
                    input_metric,
                    lazyframe
                )
            )
        // SymmetricIdDistance -> FrameDistance<SymmetricDistance>
        } else if MI == Type::of::<SymmetricIdDistance>() && MO == Type::of::<FrameDistance<SymmetricDistance>>() {
            let input_metric = try_!(input_metric.downcast_ref::<SymmetricIdDistance>()).clone();
            make_stable_lazyframe::<_, FrameDistance<SymmetricDistance>>(input_domain, input_metric, lazyframe).into_any()
        } else {
            err!(FFI, "MI ({:?}) must match MO ({:?}), or SymmetricIdDistance -> FrameDistance<SymmetricDistance>", MI.descriptor, MO.descriptor).into()
        }
    } else {
        // Bounded metrics always convert to their unbounded FrameDistance form.
        // An explicit MO must match the actual result rather than being ignored.
        let expected_mo = if MI == Type::of::<ChangeOneDistance>() {
            Type::of::<FrameDistance<SymmetricDistance>>()
        } else {
            Type::of::<FrameDistance<SymmetricIdDistance>>()
        };
        if requested_mo != "MI" && MO != expected_mo {
            return err!(
                FFI,
                "MI ({:?}) requires MO ({:?}), got ({:?})",
                MI.descriptor, expected_mo.descriptor, MO.descriptor
            ).into();
        }

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
            [(MI, [ChangeOneDistance, ChangeOneIdDistance])],
            (
                input_domain,
                input_metric,
                lazyframe
            )
        )
    }.into()
}

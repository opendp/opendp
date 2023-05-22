use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, Metric, MetricSpace};
use crate::err;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyTransformation, IntoAnyStabilityMapExt};
use crate::ffi::util::Type;
use crate::metrics::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
};
use crate::transformations::cast_metric::traits::{
    BoundedMetric, OrderedMetric, UnboundedMetric, UnorderedMetric,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_ordered_random(
    domain: *const AnyDomain,
    D: *const c_char,
    MI: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let domain = try_as_ref!(domain).clone();
    let D = try_!(Type::try_from(D));
    let MI = try_!(Type::try_from(MI));

    if D != domain.type_ {
        return err!(FFI, "D must match domain's type").into();
    }

    fn monomorphize<MI: 'static + UnorderedMetric<Distance = IntDistance>>(
        domain: AnyDomain,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::OrderedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::OrderedMetric): MetricSpace,
    {
        let trans = try_!(super::make_ordered_random::<AnyDomain, MI>(domain));

        trans
            .with_map(
                AnyMetric::new(trans.input_metric.clone()),
                AnyMetric::new(trans.output_metric.clone()),
                trans.stability_map.clone().into_any(),
            )
            .into()
    }

    dispatch!(
        monomorphize,
        [(MI, [SymmetricDistance, ChangeOneDistance])],
        (domain)
    )
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_unordered(
    domain: *const AnyDomain,
    D: *const c_char,
    MI: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let domain = try_as_ref!(domain).clone();
    let D = try_!(Type::try_from(D));
    let MI = try_!(Type::try_from(MI));

    if D != domain.type_ {
        return err!(FFI, "D must match domain's type").into();
    }

    fn monomorphize<MI: 'static + OrderedMetric<Distance = IntDistance>>(
        domain: AnyDomain,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::UnorderedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::UnorderedMetric): MetricSpace,
    {
        let trans = try_!(super::make_unordered::<AnyDomain, MI>(domain));

        trans
            .with_map(
                AnyMetric::new(trans.input_metric.clone()),
                AnyMetric::new(trans.output_metric.clone()),
                trans.stability_map.clone().into_any(),
            )
            .into()
    }

    dispatch!(
        monomorphize,
        [(MI, [InsertDeleteDistance, HammingDistance])],
        (domain)
    )
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_metric_bounded(
    domain: *const AnyDomain,
    D: *const c_char,
    MI: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let domain = try_as_ref!(domain).clone();
    let D = try_!(Type::try_from(D));
    let MI = try_!(Type::try_from(MI));

    if D != domain.type_ {
        return err!(FFI, "D must match domain's type").into();
    }

    fn monomorphize<MI: 'static + UnboundedMetric<Distance = IntDistance>>(
        domain: AnyDomain,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::BoundedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::BoundedMetric): MetricSpace,
    {
        let trans = try_!(super::make_metric_bounded::<AnyDomain, MI>(domain));

        trans
            .with_map(
                AnyMetric::new(trans.input_metric.clone()),
                AnyMetric::new(trans.output_metric.clone()),
                trans.stability_map.clone().into_any(),
            )
            .into()
    }
    dispatch!(
        monomorphize,
        [(MI, [SymmetricDistance, InsertDeleteDistance])],
        (domain)
    )
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_metric_unbounded(
    domain: *const AnyDomain,
    D: *const c_char,
    MI: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let domain = try_as_ref!(domain).clone();
    let D = try_!(Type::try_from(D));
    let MI = try_!(Type::try_from(MI));

    if D != domain.type_ {
        return err!(FFI, "D must match domain's type").into();
    }

    fn monomorphize<MI: 'static + BoundedMetric<Distance = IntDistance>>(
        domain: AnyDomain,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::UnboundedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::UnboundedMetric): MetricSpace,
    {
        let trans = try_!(super::make_metric_unbounded::<AnyDomain, MI>(domain));

        trans
            .with_map(
                AnyMetric::new(trans.input_metric.clone()),
                AnyMetric::new(trans.output_metric.clone()),
                trans.stability_map.clone().into_any(),
            )
            .into()
    }
    dispatch!(
        monomorphize,
        [(MI, [ChangeOneDistance, HammingDistance])],
        (domain)
    )
}

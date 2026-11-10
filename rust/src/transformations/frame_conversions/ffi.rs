use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace},
    domains::{DataFrameDomain, LazyFrameDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, FrameDistance, HammingDistance,
        InsertDeleteDistance, SymmetricDistance, SymmetricIdDistance,
    },
};

use super::{make_collect, make_lazy};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_collect(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain =
        try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let M = input_metric.type_.clone();

    fn monomorphize<M: 'static + Metric>(
        domain: LazyFrameDomain,
        metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        (LazyFrameDomain, M): MetricSpace,
        (DataFrameDomain, M): MetricSpace,
        M::Distance: Clone + 'static,
    {
        make_collect(domain, metric.downcast_ref::<M>()?.clone()).into_any()
    }

    dispatch!(
        monomorphize,
        [(M, [
            SymmetricDistance, InsertDeleteDistance, ChangeOneDistance,
            HammingDistance, SymmetricIdDistance, ChangeOneIdDistance,
            FrameDistance<SymmetricDistance>,
            FrameDistance<InsertDeleteDistance>,
            FrameDistance<SymmetricIdDistance>
        ])],
        (input_domain, input_metric)
    ).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_lazy(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain =
        try_!(try_as_ref!(input_domain).downcast_ref::<DataFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let M = input_metric.type_.clone();

    fn monomorphize<M: 'static + Metric>(
        domain: DataFrameDomain,
        metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        (DataFrameDomain, M): MetricSpace,
        (LazyFrameDomain, M): MetricSpace,
        M::Distance: Clone + 'static,
    {
        make_lazy(domain, metric.downcast_ref::<M>()?.clone()).into_any()
    }

    dispatch!(
        monomorphize,
        [(M, [
            SymmetricDistance, InsertDeleteDistance, ChangeOneDistance,
            HammingDistance, SymmetricIdDistance, ChangeOneIdDistance,
            FrameDistance<SymmetricDistance>,
            FrameDistance<InsertDeleteDistance>,
            FrameDistance<SymmetricIdDistance>
        ])],
        (input_domain, input_metric)
    ).into()
}

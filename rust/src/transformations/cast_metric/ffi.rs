use crate::core::{FfiResult, Metric, MetricSpace};
use crate::err;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast, IntoAnyStabilityMapExt};
use crate::metrics::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
};
use crate::transformations::cast_metric::traits::{
    BoundedMetric, OrderedMetric, UnboundedMetric, UnorderedMetric,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_ordered_random(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MI_ = input_metric.type_.clone();

    fn monomorphize<MI: 'static + UnorderedMetric<Distance = IntDistance>>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::OrderedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::OrderedMetric): MetricSpace,
    {
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        let trans = try_!(super::make_ordered_random::<AnyDomain, MI>(
            input_domain.clone(),
            input_metric
        ));

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
        [(MI_, [SymmetricDistance, ChangeOneDistance])],
        (input_domain, input_metric)
    )
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_unordered(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MI_ = input_metric.type_.clone();

    fn monomorphize<MI: 'static + OrderedMetric<Distance = IntDistance>>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::UnorderedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::UnorderedMetric): MetricSpace,
    {
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        let trans = try_!(super::make_unordered::<AnyDomain, MI>(
            input_domain.clone(),
            input_metric
        ));

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
        [(MI_, [InsertDeleteDistance, HammingDistance])],
        (input_domain, input_metric)
    )
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_metric_bounded(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MI_ = input_metric.type_.clone();

    fn monomorphize<MI: 'static + UnboundedMetric<Distance = IntDistance>>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::BoundedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::BoundedMetric): MetricSpace,
    {
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        let trans = try_!(super::make_metric_bounded::<AnyDomain, MI>(
            input_domain.clone(),
            input_metric
        ));

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
        [(MI_, [SymmetricDistance, InsertDeleteDistance])],
        (input_domain, input_metric)
    )
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_metric_unbounded(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MI_ = input_metric.type_.clone();

    fn monomorphize<MI: 'static + BoundedMetric<Distance = IntDistance>>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI::Distance: 'static,
        <MI::UnboundedMetric as Metric>::Distance: 'static,
        (AnyDomain, MI): MetricSpace,
        (AnyDomain, MI::UnboundedMetric): MetricSpace,
    {
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        let trans = try_!(super::make_metric_unbounded::<AnyDomain, MI>(
            input_domain.clone(),
            input_metric,
        ));

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
        [(MI_, [ChangeOneDistance, HammingDistance])],
        (input_domain, input_metric)
    )
}

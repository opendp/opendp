use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, Downcast};
use crate::ffi::any::{AnyObject, AnyTransformation};
use crate::ffi::util::Type;
use crate::traits::{Hashable, Number, Primitive};
use crate::transformations::{make_find, make_find_bin, make_index, DatasetMetric};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_find(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    categories: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        categories: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        M: 'static + DatasetMetric,
        TIA: 'static + Hashable,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<OptionDomain<AtomDomain<usize>>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let categories = categories.downcast_ref::<Vec<TIA>>()?.clone();
        make_find(input_domain, input_metric, categories).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let categories = try_as_ref!(categories);
    let M = input_metric.type_.clone();
    let TIA = try_!(input_domain.type_.get_atom());
    dispatch!(monomorphize, [
        (M, @dataset_metrics),
        (TIA, @hashable)
    ], (input_domain, input_metric, categories))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_find_bin(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    edges: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        edges: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TIA: 'static + Number,
        M: 'static + DatasetMetric,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<usize>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let edges = try_as_ref!(edges).downcast_ref::<Vec<TIA>>()?.clone();
        make_find_bin(input_domain, input_metric, edges).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let edges = try_as_ref!(edges);
    let M = input_metric.type_.clone();
    let TIA = try_!(input_domain.type_.get_atom());
    dispatch!(monomorphize, [
        (M, @dataset_metrics),
        (TIA, @numbers)
    ], (input_domain, input_metric, edges))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_index(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    categories: *const AnyObject,
    null: *const AnyObject,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, TOA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        edges: &AnyObject,
        null: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TOA: Primitive,
        M: 'static + DatasetMetric,
        (VectorDomain<AtomDomain<usize>>, M): MetricSpace,
        (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<usize>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let edges = try_as_ref!(edges).downcast_ref::<Vec<TOA>>()?.clone();
        let null = try_as_ref!(null).downcast_ref::<TOA>()?.clone();
        make_index(input_domain, input_metric, edges, null).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let categories = try_as_ref!(categories);
    let null = try_as_ref!(null);
    let M = input_metric.type_.clone();
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (M, @dataset_metrics),
        (TOA, @primitives)
    ], (input_domain, input_metric, categories, null))
    .into()
}

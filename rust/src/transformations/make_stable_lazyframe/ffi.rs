use polars::lazy::frame::LazyFrame;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    domains::LazyFrameDomain,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    metrics::SymmetricDistance,
};

use super::make_stable_lazyframe;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_stable_lazyframe(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    lazyframe: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<SymmetricDistance>()).clone();

    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();
    make_stable_lazyframe(input_domain, input_metric, lazyframe)
        .into_any()
        .into()
}

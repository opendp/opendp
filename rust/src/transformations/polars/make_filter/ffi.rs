use polars::prelude::*;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    domains::LazyFrameDomain,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    metrics::SymmetricDistance,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_filter(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    expr: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    // dereference all the pointers
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<SymmetricDistance>()).clone();
    let expr = try_!(try_as_ref!(expr).downcast_ref::<Expr>()).clone();

    // call the original function
    super::make_filter(input_domain, input_metric, expr)
        .into_any()
        .into()
}

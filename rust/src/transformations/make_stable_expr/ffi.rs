use polars_plan::dsl::Expr;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    domains::WildExprDomain,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    metrics::{L1Distance, PartitionDistance, SymmetricDistance},
};

use super::make_stable_expr;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_stable_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    expr: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<WildExprDomain>()).clone();
    let input_metric =
        try_!(try_as_ref!(input_metric).downcast_ref::<PartitionDistance<SymmetricDistance>>())
            .clone();

    let expr = try_!(try_as_ref!(expr).downcast_ref::<Expr>()).clone();
    // TODO: dispatch to different output types
    make_stable_expr::<_, L1Distance<f64>>(input_domain, input_metric, expr)
        .into_any()
        .into()
}

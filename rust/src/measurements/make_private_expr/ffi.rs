use polars_plan::dsl::Expr;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::ExprDomain,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util,
    },
    measures::MaxDivergence,
    metrics::{PartitionDistance, SymmetricDistance},
};

use super::make_private_expr;

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    expr: *const AnyObject,
    global_scale: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<ExprDomain>()).clone();
    let input_metric =
        try_!(try_as_ref!(input_metric).downcast_ref::<PartitionDistance<SymmetricDistance>>())
            .clone();
    let output_measure =
        try_!(try_as_ref!(output_measure).downcast_ref::<MaxDivergence<f64>>()).clone();

    let expr = try_!(try_as_ref!(expr).downcast_ref::<Expr>()).clone();

    let global_scale = if let Some(param) = util::as_ref(global_scale) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };

    make_private_expr(
        input_domain,
        input_metric,
        output_measure,
        expr,
        global_scale,
    )
    .into_any()
    .into()
}

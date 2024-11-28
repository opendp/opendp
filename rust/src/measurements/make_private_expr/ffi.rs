use polars_plan::dsl::Expr;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt, Measure},
    domains::WildExprDomain,
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util,
    },
    measurements::PrivateExpr,
    measures::{MaxDivergence, ZeroConcentratedDivergence},
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
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<WildExprDomain>()).clone();
    let input_metric =
        try_!(try_as_ref!(input_metric).downcast_ref::<PartitionDistance<SymmetricDistance>>())
            .clone();

    let output_measure = try_as_ref!(output_measure);
    let MO_ = output_measure.type_.clone();

    let expr = try_!(try_as_ref!(expr).downcast_ref::<Expr>()).clone();

    let global_scale = if let Some(param) = util::as_ref(global_scale) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };

    fn monomorphize<MO: 'static + Measure>(
        input_domain: WildExprDomain,
        input_metric: PartitionDistance<SymmetricDistance>,
        output_measure: &AnyMeasure,
        expr: Expr,
        global_scale: Option<f64>,
    ) -> Fallible<AnyMeasurement>
    where
        Expr: PrivateExpr<PartitionDistance<SymmetricDistance>, MO>,
    {
        let output_measure = output_measure.downcast_ref::<MO>()?.clone();
        make_private_expr(
            input_domain,
            input_metric,
            output_measure,
            expr,
            global_scale,
        )
        .into_any()
    }

    dispatch!(
        monomorphize,
        [(MO_, [MaxDivergence, ZeroConcentratedDivergence])],
        (
            input_domain,
            input_metric,
            output_measure,
            expr,
            global_scale
        )
    )
    .into()
}

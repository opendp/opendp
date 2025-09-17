use polars_plan::dsl::Expr;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace},
    domains::{ExprDomain, WildExprDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    metrics::{FrameDistance, InsertDeleteDistance, SymmetricDistance, SymmetricIdDistance},
    transformations::StableExpr,
};

use super::make_stable_expr;

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_stable_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    expr: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<WildExprDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let expr = try_!(try_as_ref!(expr).downcast_ref::<Expr>()).clone();

    let M = input_metric.type_.clone();

    fn monomorphize<M: 'static + Metric>(
        input_domain: WildExprDomain,
        input_metric: &AnyMetric,
        expr: Expr,
    ) -> Fallible<AnyTransformation>
    where
        Expr: StableExpr<M, M>,
        (WildExprDomain, M): MetricSpace,
        (ExprDomain, M): MetricSpace,
    {
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        make_stable_expr::<M, M>(input_domain, input_metric, expr).into_any()
    }

    dispatch!(
        monomorphize,
        [(M, [FrameDistance<SymmetricDistance>, FrameDistance<SymmetricIdDistance>, FrameDistance<InsertDeleteDistance>])],
        (input_domain, input_metric, expr)
    )
    .into()
}

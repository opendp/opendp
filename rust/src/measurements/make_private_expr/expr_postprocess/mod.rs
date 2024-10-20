use std::sync::Arc;

use crate::combinators::{make_basic_composition, BasicCompositionMeasure};
use crate::core::{Metric, MetricSpace};
use crate::domains::{ExprPlan, WildExprDomain};
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};

use polars::lazy::dsl::Expr;

use super::PrivateExpr;

#[cfg(test)]
mod test;

/// Make a measurement that applies post-processing to an expression under bounded-DP
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - valid selections shown in table above
/// * `output_measure` - how to measure privacy loss
/// * `input_exprs` - expressions to be post-processed
/// * `postprocessor` - function that applies post-processing to the expressions
/// * `param` - global noise (re)scale parameter
pub fn make_expr_postprocess<MI: 'static + Metric, MO: 'static + BasicCompositionMeasure>(
    input_domain: WildExprDomain,
    input_metric: MI,
    output_measure: MO,
    input_exprs: Vec<Expr>,
    postprocessor: impl Fn(Vec<Expr>) -> Fallible<Expr> + 'static + Send + Sync,
    param: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, ExprPlan, MI, MO>>
where
    Expr: PrivateExpr<MI, MO>,
    (WildExprDomain, MI): MetricSpace,
{
    let m_exprs = input_exprs
        .into_iter()
        .map(|expr| {
            expr.make_private(
                input_domain.clone(),
                input_metric.clone(),
                output_measure.clone(),
                param,
            )
        })
        .collect::<Fallible<Vec<_>>>()?;

    let m_comp = make_basic_composition(m_exprs)?;
    let f_comp = m_comp.function.clone();

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg| {
            let plans = f_comp.eval(&arg)?;
            let plan = plans[0].plan.clone();
            let (exprs, fills): (_, Vec<Option<Expr>>) =
                plans.into_iter().map(|p| (p.expr, p.fill)).unzip();

            Ok(ExprPlan {
                plan,
                expr: postprocessor(exprs)?,
                fill: fills
                    .into_iter()
                    .collect::<Option<_>>()
                    .map(|exprs| postprocessor(exprs))
                    .transpose()?,
            })
        }),
        input_metric,
        output_measure,
        m_comp.privacy_map.clone(),
    )
}

pub fn match_postprocess<MI: 'static + Metric, MO: 'static + BasicCompositionMeasure>(
    input_domain: WildExprDomain,
    input_metric: MI,
    output_measure: MO,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Option<Measurement<WildExprDomain, ExprPlan, MI, MO>>>
where
    Expr: PrivateExpr<MI, MO>,
    (WildExprDomain, MI): MetricSpace,
{
    match expr {
        #[cfg(feature = "contrib")]
        Expr::Alias(expr, name) => make_expr_postprocess(
            input_domain,
            input_metric,
            output_measure,
            vec![expr.as_ref().clone()],
            move |exprs| {
                let [expr] = <[Expr; 1]>::try_from(exprs)
                    .expect("Alias will always be applied to exactly one expression.");
                Ok(expr.alias(name.clone()))
            },
            global_scale,
        ),

        #[cfg(feature = "contrib")]
        Expr::BinaryExpr { left, op, right } => {
            make_expr_postprocess(
                input_domain,
                input_metric,
                output_measure,
                vec![left.as_ref().clone(), right.as_ref().clone()],
                move |exprs| {
                    let [left, right] = <[Expr; 2]>::try_from(exprs)
                        .expect("Binary operations will always be applied over exactly two expressions.")
                        .map(Arc::new);
                    Ok(Expr::BinaryExpr { left, op, right })
                },
                global_scale,
            )
        }

        _ => return Ok(None)
    }.map(Some)
}

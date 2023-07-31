use crate::combinators::{make_basic_composition, BasicCompositionMeasure};
use crate::core::{Metric, MetricSpace};
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
};

use polars::lazy::dsl::Expr;

use super::PrivateExpr;

/// Make a measurement that applies post-processing to an expression under bounded-DP
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - valid selections shown in table above
/// * `output_measure` - how to measure privacy loss
/// * `input_exprs` - expressions to be post-processed
/// * `postprocessor` - function that applies post-processing to the expressions
/// * `param` - global noise (re)scale parameter
pub fn make_expr_postprocess<MS: 'static + Metric, MO: 'static + BasicCompositionMeasure>(
    input_domain: ExprDomain,
    input_metric: MS,
    output_measure: MO,
    input_exprs: Vec<Expr>,
    postprocessor: impl Fn(Vec<Expr>) -> Fallible<Expr> + 'static + Send + Sync,
    param: f64,
) -> Fallible<Measurement<ExprDomain, Expr, MS, MO>>
where
    Expr: PrivateExpr<MS, MO>,
    (ExprDomain, MS): MetricSpace,
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
        Function::new_fallible(move |arg| postprocessor(f_comp.eval(&arg)?)),
        input_metric,
        output_measure,
        m_comp.privacy_map.clone(),
    )
}

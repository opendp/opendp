use crate::core::{ExprFunction, Measure, Metric, MetricSpace, PrivacyMap};
use crate::transformations::StableExpr;
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
};

use num::Zero;
use polars::lazy::dsl::Expr;

#[cfg(test)]
mod test;

/// Make a measurement that returns a literal.
///
/// Commonly known as the "constant" mechanism.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - valid selections shown in table above
/// * `expr` - literal expression
pub fn make_expr_private_lit<MI: 'static + Metric, MO: 'static + Measure>(
    input_domain: ExprDomain,
    input_metric: MI,
    expr: Expr,
) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>
where
    MO::Distance: Zero,
    Expr: StableExpr<MI, MI>,
    (ExprDomain, MI): MetricSpace,
{
    let Expr::Literal(_) = &expr else {
        return fallible!(MakeMeasurement, "Expected Literal expression");
    };

    Measurement::new(
        input_domain,
        Function::from_expr(expr),
        input_metric,
        MO::default(),
        PrivacyMap::new(move |_| MO::Distance::zero()),
    )
}

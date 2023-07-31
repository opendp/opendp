use crate::core::{Measure, Metric, MetricSpace, PrivacyMap};
use crate::transformations::StableExpr;
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
};

use num::Zero;
use polars::lazy::dsl::Expr;
use polars_plan::logical_plan::{LiteralValue, LogicalPlan};

/// Make a measurement that returns a literal.
///
/// Commonly known as the "constant" mechanism.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - valid selections shown in table above
/// * `expr` - literal expression
pub fn make_expr_private_lit<MS: 'static + Metric, MO: 'static + Measure>(
    input_domain: ExprDomain,
    input_metric: MS,
    expr: Expr,
) -> Fallible<Measurement<ExprDomain, Expr, MS, MO>>
where
    MO::Distance: Zero,
    Expr: StableExpr<MS, MS>,
    (ExprDomain, MS): MetricSpace,
{
    let Expr::Literal(lit) = &expr else {
        return fallible!(MakeMeasurement, "Expected Literal expression");
    };

    let LiteralValue::Series(series) = lit else {
        return fallible!(MakeMeasurement, "Expected Literal series");
    };
    println!("lit: {:?}", &**series);

    Measurement::new(
        input_domain,
        Function::new_fallible(
            // in most other situations, we would use `Function::new_expr`, but we need to return a Fallible here
            move |(_, expr_wild): &(LogicalPlan, Expr)| -> Fallible<Expr> {
                if expr_wild != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "Expected all() as input (denoting that all columns are selected). This is because literal is a leaf node in the expression tree."
                    );
                }
                Ok(expr.clone())
            },
        ),
        input_metric,
        MO::default(),
        PrivacyMap::new(move |_| MO::Distance::zero()),
    )
}

use crate::core::{Measure, Metric, MetricSpace, PrivacyMap};
use crate::domains::{ExprPlan, WildExprDomain};
use crate::metrics::L0PInfDistance;
use crate::{
    core::{Function, Measurement},
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
pub fn make_expr_private_lit<MI: 'static + Metric, MO: 'static + Measure, const P: usize>(
    input_domain: WildExprDomain,
    input_metric: L0PInfDistance<P, MI>,
    expr: Expr,
) -> Fallible<Measurement<WildExprDomain, L0PInfDistance<P, MI>, MO, ExprPlan>>
where
    MO::Distance: Zero,
    (WildExprDomain, L0PInfDistance<P, MI>): MetricSpace,
{
    let Expr::Literal(_) = &expr else {
        return fallible!(MakeMeasurement, "Expected Literal expression");
    };

    Measurement::new(
        input_domain,
        input_metric,
        MO::default(),
        Function::from_expr(expr),
        PrivacyMap::new(|_| MO::Distance::zero()),
    )
}

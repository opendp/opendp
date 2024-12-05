use crate::core::{Measure, Metric, MetricSpace, PrivacyMap};
use crate::metrics::PartitionDistance;
use crate::polars::ExprFunction;
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
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Measurement<ExprDomain, Expr, PartitionDistance<MI>, MO>>
where
    MO::Distance: Zero,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
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

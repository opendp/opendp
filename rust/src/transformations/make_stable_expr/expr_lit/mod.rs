use polars::prelude::*;
use polars_plan::utils::expr_output_name;

use crate::core::{ExprFunction, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, LogicalPlanDomain, OuterMetric, SeriesDomain};
use crate::error::Fallible;
use crate::transformations::DatasetMetric;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a literal.
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
/// * `expr` - A literal expression.
pub fn make_expr_lit<M: OuterMetric>(
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
{
    let Expr::Literal(literal_value) = &expr else {
        return fallible!(MakeTransformation, "Expected literal expression");
    };

    let name = expr_output_name(&expr)?;
    let dtype = literal_value.get_datatype();
    let mut series_domain = SeriesDomain::new_from_field(Field::new(name.as_ref(), dtype))?;
    series_domain.nullable = false;

    let output_domain = ExprDomain::new(
        LogicalPlanDomain::new(vec![series_domain])?,
        input_domain.context.clone(),
    );

    Transformation::new(
        input_domain,
        output_domain,
        Function::from_expr(expr),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}

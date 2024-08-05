use polars::prelude::*;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric};
use crate::error::{ErrorVariant::MakeTransformation, *};
use crate::polars::ExprFunction;
use crate::transformations::DatasetMetric;

/// Make a Transformation that returns a `col(column_name)` expression for a Lazy Frame.
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
/// * `expr` - A column expression.
pub fn make_expr_col<M: OuterMetric>(
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
{
    let Expr::Column(col_name) = expr else {
        return fallible!(MakeTransformation, "Expected col() expression");
    };
    let col_name = col_name.to_string();

    let mut output_domain = input_domain.clone();
    output_domain
        .frame_domain
        .series_domains
        .retain(|v| v.field.name == col_name);

    output_domain
        .check_one_column()
        .with_variant(MakeTransformation)?;

    Transformation::new(
        input_domain,
        output_domain,
        Function::from_expr(col(&*col_name)),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}

#[cfg(test)]
mod test;

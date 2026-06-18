use super::expr_count::counting_query_stability_map;
use crate::core::{Function, MetricSpace, Transformation};
use crate::domains::{AtomDomain, Context, ExprDomain, Margin, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{L01InfDistance, LpDistance};
use crate::transformations::traits::UnboundedMetric;
use polars::datatypes::DataType;
use polars::prelude::len;
use polars_plan::dsl::Expr;
use polars_plan::plans::typed_lit;

#[cfg(test)]
mod test;

/// Polars operator to get the length of a LazyFrame
///
/// | input_metric                              |
/// | ----------------------------------------- |
/// | `PartitionDistance<SymmetricDistance>`    |
/// | `PartitionDistance<InsertDeleteDistance>` |
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - valid selections shown in table above
/// * `expr` - a length expression
pub fn make_expr_len<MI, const P: usize>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
    allow_negative: bool,
) -> Fallible<Transformation<WildExprDomain, L01InfDistance<MI>, ExprDomain, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (WildExprDomain, L01InfDistance<MI>): MetricSpace,
    (ExprDomain, LpDistance<P, f64>): MetricSpace,
{
    let output_expr = match &expr {
        Expr::Len => expr.clone(),
        Expr::Cast {
            expr: inner,
            dtype: _,
            options: _,
        } if matches!(**inner, Expr::Len) => expr.clone(),
        _ => return fallible!(MakeTransformation, "expected len expression"),
    };

    let old_margin = input_domain.context.aggregation("len")?;
    let margin = Margin {
        by: old_margin.by,
        max_length: Some(1),
        max_groups: Some(1),
        invariant: old_margin.invariant,
    };

    // build output domain
    let output_domain = ExprDomain {
        column: SeriesDomain::new("len", AtomDomain::<u32>::default()),
        context: Context::Aggregation {
            margin: margin.clone(),
        },
    };

    // Need to cast here?
    Transformation::new(
        input_domain,
        input_metric,
        output_domain,
        LpDistance::default(),
        Function::from_expr(output_expr).fill_with(typed_lit(0u32)),
        counting_query_stability_map(margin.invariant),
    )
}

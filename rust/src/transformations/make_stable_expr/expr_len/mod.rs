use crate::core::{Function, MetricSpace, Transformation};
use crate::domains::{AtomDomain, Context, ExprDomain, Margin, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{LpDistance, PartitionDistance};
use crate::transformations::traits::UnboundedMetric;
use polars_plan::dsl::{len, Expr};
use polars_plan::plans::typed_lit;

use super::expr_count::counting_query_stability_map;

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
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (WildExprDomain, PartitionDistance<MI>): MetricSpace,
    (ExprDomain, LpDistance<P, f64>): MetricSpace,
{
    let Expr::Len = expr else {
        return fallible!(MakeTransformation, "expected len expression");
    };

    let old_margin = input_domain.context.aggregation("len")?;
    let margin = Margin {
        by: old_margin.by,
        max_partition_length: Some(1),
        max_num_partitions: Some(1),
        max_partition_contributions: old_margin.max_partition_contributions,
        max_influenced_partitions: old_margin.max_influenced_partitions,
        public_info: old_margin.public_info,
    };

    // build output domain
    let output_domain = ExprDomain {
        column: SeriesDomain::new("len", AtomDomain::<u32>::default()),
        context: Context::Aggregation {
            margin: margin.clone(),
        },
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::from_expr(len()).fill_with(typed_lit(0u32)),
        input_metric,
        LpDistance::default(),
        counting_query_stability_map(margin.public_info),
    )
}

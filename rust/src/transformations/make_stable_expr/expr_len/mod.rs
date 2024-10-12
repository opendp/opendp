use crate::core::{Function, MetricSpace, Transformation};
use crate::domains::{AtomDomain, DslPlanDomain, ExprDomain, SeriesDomain};
use crate::error::*;
use crate::metrics::{LpDistance, PartitionDistance};
use crate::polars::ExprFunction;
use crate::transformations::traits::UnboundedMetric;
use polars_plan::dsl::{len, Expr};

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
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
    (ExprDomain, LpDistance<P, f64>): MetricSpace,
{
    let Expr::Len = expr else {
        return fallible!(MakeTransformation, "expected len expression");
    };

    // check that we are in a context where it is ok to break row-alignment
    input_domain.context.check_alignment_can_be_broken()?;

    // build output domain
    let output_domain = ExprDomain::new(
        DslPlanDomain::new(vec![SeriesDomain::new("len", AtomDomain::<u32>::default())])?,
        input_domain.context.clone(),
    );

    // we only care about the margin that matches the current grouping columns
    let public_info = input_domain.active_margin()?.public_info;

    Transformation::new(
        input_domain,
        output_domain,
        Function::from_expr(len()),
        input_metric,
        LpDistance::default(),
        counting_query_stability_map(public_info),
    )
}

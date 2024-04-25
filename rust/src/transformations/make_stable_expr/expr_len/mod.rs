use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, LogicalPlanDomain, MarginPub, SeriesDomain};
use crate::error::*;
use crate::metrics::{IntDistance, LpDistance, PartitionDistance};
use crate::traits::{InfCast, InfMul, InfSqrt, ProductOrd};
use crate::transformations::traits::UnboundedMetric;
use polars_plan::dsl::{len, Expr};
use polars_plan::logical_plan::LogicalPlan;

#[cfg(test)]
mod test;

/// Polars operator to sum a column in a LazyFrame
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
    input_domain.context.break_alignment()?;

    // build output domain
    let output_domain = ExprDomain::new(
        LogicalPlanDomain::new(vec![SeriesDomain::new("len", AtomDomain::<u32>::default())])?,
        input_domain.context.clone(),
    );

    // we only care about the margin that matches the current grouping columns
    let margin_id = input_domain.context.grouping_columns()?;
    let public_info = (input_domain.frame_domain.margins.get(&margin_id))
        .map(|m| m.public_info.clone())
        .unwrap_or_default();

    let norm_map = move |d_in: f64| match P {
        1 => Ok(d_in),
        2 => d_in.inf_sqrt(),
        _ => return fallible!(MakeTransformation, "unsupported Lp norm"),
    };

    let pp_map = move |d_in: &IntDistance| match public_info {
        Some(MarginPub::Lengths) => 0,
        _ => *d_in,
    };

    let stability_map = StabilityMap::new_fallible(
        move |(l0, l1, l_inf): &(IntDistance, IntDistance, IntDistance)| {
            let l0_p = norm_map(f64::from(*l0))?;
            let l1_p = f64::inf_cast(pp_map(l1))?;
            let l_inf_p = f64::inf_cast(pp_map(l_inf))?;

            l1_p.total_min(l0_p.inf_mul(&l_inf_p)?)
        },
    );

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(
            // in most other situations, we would use `Function::new_expr`, but we need to return a Fallible here
            move |(lp, expr): &(LogicalPlan, Expr)| -> Fallible<(LogicalPlan, Expr)> {
                if expr != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "Expected all() as input (denoting that all columns are selected). This is because column selection is a leaf node in the expression tree."
                    );
                }
                Ok((lp.clone(), len()))
            },
        ),
        input_metric,
        LpDistance::default(),
        stability_map,
    )
}

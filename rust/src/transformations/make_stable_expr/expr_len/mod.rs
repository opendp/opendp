use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, DslPlanDomain, ExprDomain, MarginPub, SeriesDomain};
use crate::error::*;
use crate::metrics::{IntDistance, LpDistance, PartitionDistance};
use crate::polars::ExprFunction;
use crate::traits::{InfCast, InfMul, InfSqrt, ProductOrd};
use crate::transformations::traits::UnboundedMetric;
use polars_plan::dsl::{len, Expr};

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
    input_domain.context.break_alignment()?;

    // build output domain
    let output_domain = ExprDomain::new(
        DslPlanDomain::new(vec![SeriesDomain::new("len", AtomDomain::<u32>::default())])?,
        input_domain.context.clone(),
    );

    // we only care about the margin that matches the current grouping columns
    let margin_id = input_domain.context.grouping_columns()?;
    let public_info = (input_domain.frame_domain.margins.get(&margin_id))
        .map(|m| m.public_info.clone())
        .unwrap_or_default();

    //  norm_map(d_in) returns d_in^(1/p)
    let norm_map = move |d_in: f64| match P {
        1 => Ok(d_in),
        2 => d_in.inf_sqrt(),
        _ => {
            return fallible!(
                MakeTransformation,
                "unsupported Lp norm. Must be an L1 or L2 norm."
            )
        }
    };

    //  pp_map(d_in) returns the per-partition change in counts if d_in input records are changed
    let pp_map = move |d_in: &IntDistance| match public_info {
        Some(MarginPub::Lengths) => 0,
        _ => *d_in,
    };

    // an explanatory example of this math is provided in the tests
    let stability_map = StabilityMap::new_fallible(
        move |(l0, l1, l_inf): &(IntDistance, IntDistance, IntDistance)| {
            // if l0 partitions may change, then l0_p denotes how sensitivity scales wrt the norm
            let l0_p = norm_map(f64::from(*l0))?;
            // per-partition count sensitivity depends on whether counts are public information/may change
            let l1_p = f64::inf_cast(pp_map(l1))?;
            let l_inf_p = f64::inf_cast(pp_map(l_inf))?;

            // min(l1',    l0' *         l∞')
            l1_p.total_min(l0_p.inf_mul(&l_inf_p)?)
        },
    );

    Transformation::new(
        input_domain,
        output_domain,
        Function::from_expr(len()),
        input_metric,
        LpDistance::default(),
        stability_map,
    )
}

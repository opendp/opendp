use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    AtomDomain, Context, ExprDomain, Margin, MarginPub, SeriesDomain, WildExprDomain,
};
use crate::error::*;
use crate::metrics::{IntDistance, LpDistance, PartitionDistance};
use crate::traits::{InfMul, InfSqrt, ProductOrd};
use crate::transformations::traits::UnboundedMetric;
use polars::prelude::{AggExpr, FunctionExpr};
use polars_plan::dsl::Expr;
use polars_plan::plans::typed_lit;

use super::StableExpr;

#[cfg(test)]
mod test;

enum Strategy {
    Count,
    NullCount,
    Len,
    NUnique,
}

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
pub fn make_expr_count<MI, const P: usize>(
    input_domain: WildExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
    (ExprDomain, LpDistance<P, f64>): MetricSpace,
    Expr: StableExpr<PartitionDistance<MI>, PartitionDistance<MI>>,
{
    let (input, strategy) = match expr {
        Expr::Agg(AggExpr::Count(input, include_nulls)) => (
            input.as_ref().clone(),
            if include_nulls {
                Strategy::Len
            } else {
                Strategy::Count
            },
        ),
        Expr::Function {
            input,
            function: FunctionExpr::NullCount,
            ..
        } => (
            <[Expr; 1]>::try_from(input)
                .map_err(|_| err!(MakeTransformation, "null_count must take one argument"))?[0]
                .clone(),
            Strategy::NullCount,
        ),
        Expr::Agg(AggExpr::NUnique(input)) => (input.as_ref().clone(), Strategy::NUnique),
        _ => {
            return fallible!(
                MakeTransformation,
                "expected count, null_count, len, or n_unique expression"
            )
        }
    };

    let is_row_by_row = input
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())
        .is_ok();

    // try to construct a row-by-row expression
    let t_prior = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let margin = middle_domain.context.aggregation("count")?;

    let output_domain = ExprDomain {
        column: SeriesDomain::new(
            middle_domain.column.name.as_str(),
            AtomDomain::<u32>::default(),
        ),
        context: Context::Aggregation {
            margin: Margin {
                by: margin.by,
                max_partition_length: Some(1),
                max_num_partitions: margin.max_num_partitions,
                max_partition_contributions: None,
                max_influenced_partitions: margin.max_influenced_partitions,
                public_info: margin.public_info.clone(),
            },
        },
    };

    let will_count_all = match strategy {
        Strategy::Len => is_row_by_row,
        Strategy::Count => is_row_by_row && !middle_domain.column.nullable,
        _ => false,
    };

    let public_info = if will_count_all {
        margin.public_info
    } else {
        None
    };

    t_prior
        >> Transformation::new(
            middle_domain,
            output_domain,
            Function::then_expr(move |e| match strategy {
                Strategy::Count => e.count(),
                Strategy::NullCount => e.null_count(),
                Strategy::Len => e.len(),
                Strategy::NUnique => e.n_unique(),
            })
            .fill_with(typed_lit(0u32)),
            middle_metric,
            LpDistance::default(),
            counting_query_stability_map(public_info),
        )?
}

pub(crate) fn counting_query_stability_map<M: UnboundedMetric, const P: usize>(
    public_info: Option<MarginPub>,
) -> StabilityMap<PartitionDistance<M>, LpDistance<P, f64>> {
    if let Some(MarginPub::Lengths) = public_info {
        return StabilityMap::new(move |_| 0.);
    }

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

    // an explanatory example of this math is provided in the tests
    StabilityMap::new_fallible(
        move |(l0, l1, l_inf): &(IntDistance, IntDistance, IntDistance)| {
            // if l0 partitions may change, then l0_p denotes how sensitivity scales wrt the norm
            let l0_p = norm_map(f64::from(*l0))?;
            let l1_p = f64::from(*l1);
            let l_inf_p = f64::from(*l_inf);

            // min(l1',    l0' *         l∞')
            l1_p.total_min(l0_p.inf_mul(&l_inf_p)?)
        },
    )
}

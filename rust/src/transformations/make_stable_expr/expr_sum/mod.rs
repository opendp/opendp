use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, Margin, MarginPub::Lengths, NumericDataType, SeriesDomain};
use crate::error::*;
use crate::metrics::{IntDistance, LpDistance, PartitionDistance};
use crate::polars::ExprFunction;
use crate::traits::{
    AlertingAbs, CheckAtom, ExactIntCast, InfAdd, InfCast, InfMul, InfSqrt, InfSub, Number,
    ProductOrd,
};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{
    can_int_sum_overflow, CanFloatSumOverflow, Sequential, SumRelaxation,
};
use num::Zero;
use polars::prelude::*;
use std::collections::HashMap;

use super::StableExpr;

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
/// * `expr` - an expression ending with sum
pub fn make_expr_sum<MI, const P: usize>(
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
    Expr: StableExpr<PartitionDistance<MI>, PartitionDistance<MI>>,
{
    let Expr::Agg(AggExpr::Sum(prior_expr)) = expr else {
        return fallible!(MakeTransformation, "expected sum expression");
    };

    let t_prior = prior_expr
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let dtype = &middle_domain.active_series()?.field.dtype;

    // check that we are in a context where it is ok to break row-alignment
    middle_domain.context.break_alignment()?;

    // build output domain
    let mut output_domain = middle_domain.clone();

    // summation invalidates bounds on the active column
    output_domain.active_series_mut()?.drop_bounds()?;

    // we only care about the margin that matches the current grouping columns
    let margin_id = middle_domain.context.grouping_columns()?;
    let input_margin = (output_domain.frame_domain.margins.get(&margin_id))
        .cloned()
        .unwrap_or_default();

    // Set the margins on the output domain to consist of only one margin: 1 row per group, with at most 1 record in each group.
    let output_margin = input_margin.clone().with_max_partition_length(1);
    output_domain.frame_domain.margins = HashMap::from_iter([(margin_id, output_margin)]);

    let series_domain = middle_domain.active_series()?.clone();
    let stability_map = match dtype {
        DataType::UInt32 => sum_stability_map::<MI, P, u32>(series_domain, input_margin),
        DataType::UInt64 => sum_stability_map::<MI, P, u64>(series_domain, input_margin),
        DataType::Int8 => sum_stability_map::<MI, P, i8>(series_domain, input_margin),
        DataType::Int16 => sum_stability_map::<MI, P, i16>(series_domain, input_margin),
        DataType::Int32 => sum_stability_map::<MI, P, i32>(series_domain, input_margin),
        DataType::Int64 => sum_stability_map::<MI, P, i64>(series_domain, input_margin),
        DataType::Float32 => sum_stability_map::<MI, P, f32>(series_domain, input_margin),
        DataType::Float64 => sum_stability_map::<MI, P, f64>(series_domain, input_margin),
        _ => fallible!(MakeTransformation, "unsupported data type"),
    }?;

    t_prior
        >> Transformation::<_, _, PartitionDistance<MI>, LpDistance<P, _>>::new(
            middle_domain,
            output_domain,
            Function::then_expr(Expr::sum),
            middle_metric.clone(),
            LpDistance::default(),
            stability_map,
        )?
}

fn sum_stability_map<MI, const P: usize, TI>(
    series_domain: SeriesDomain,
    margin: Margin,
) -> Fallible<StabilityMap<PartitionDistance<MI>, LpDistance<P, f64>>>
where
    MI: UnboundedMetric,
    TI: Summand,
    f64: InfCast<TI::Sum> + InfCast<u32>,
{
    let (l, u) = series_domain.atom_domain::<TI>()?.get_closed_bounds()?;
    let (l, u) = (TI::Sum::neg_inf_cast(l)?, TI::Sum::inf_cast(u)?);

    let public_info = margin.public_info;

    let max_size = usize::exact_int_cast(margin.max_partition_length.ok_or_else(|| {
        err!(
            MakeTransformation,
            "must specify max_partition_length in margin"
        )
    })?)?;

    let pp_relaxation = f64::inf_cast(TI::Sum::relaxation(max_size, l, u)?)?;

    let norm_map = move |d_in: f64| match P {
        1 => Ok(d_in),
        2 => d_in.inf_sqrt(),
        _ => return fallible!(MakeTransformation, "unsupported Lp norm"),
    };

    let pp_map = move |d_in: &IntDistance| match public_info {
        Some(Lengths) => TI::Sum::inf_cast(*d_in / 2)?.inf_mul(&u.inf_sub(&l)?),
        _ => TI::Sum::inf_cast(*d_in)?.inf_mul(&l.alerting_abs()?.total_max(u)?),
    };

    // 'mnp_check: this invariant is used later
    if !pp_relaxation.is_zero() && !MI::ORDERED && margin.max_num_partitions.is_none() {
        return fallible!(MakeTransformation, "max_num_partitions must be known when the metric is not sensitive to ordering (SymmetricDistance)");
    }

    Ok(StabilityMap::new_fallible(
        move |(l0, l1, l_inf): &(IntDistance, IntDistance, IntDistance)| {
            // max changed partitions
            let mcp = if pp_relaxation.is_zero() {
                0u32
            } else if MI::ORDERED {
                (*l0).min(margin.max_num_partitions.unwrap_or(*l0))
            } else {
                margin
                    .max_num_partitions
                    .expect("not none due to 'mnp_check above")
            };

            let mcp_p = norm_map(f64::from(mcp))?;
            let l0_p = norm_map(f64::from(*l0))?;
            let l1_p = f64::inf_cast(pp_map(l1)?)?;
            let l_inf_p = f64::inf_cast(pp_map(l_inf)?)?;

            let relaxation = mcp_p.inf_mul(&pp_relaxation)?;

            l1_p.total_min(l0_p.inf_mul(&l_inf_p)?)?
                .inf_add(&relaxation)
        },
    ))
}

/// A data type that can be summed.
pub trait Summand: NumericDataType + CheckAtom + ProductOrd {
    /// The type of the sum emitted by Polars.
    type Sum: Accumulator + InfCast<Self>;
}

macro_rules! impl_summand {
    ($ti:ty, $to:ty) => {
        impl Summand for $ti {
            type Sum = $to;
        }
    };
}

// these associations are tested in test::test_polars_sum_types
impl_summand!(i8, i64);
impl_summand!(i16, i64);
impl_summand!(i32, i32);
impl_summand!(i64, i64);
impl_summand!(u32, u32);
impl_summand!(u64, u64);
impl_summand!(f32, f32);
impl_summand!(f64, f64);

pub trait Accumulator: Number + NumericDataType + Sized {
    fn relaxation(size_limit: usize, lower: Self, upper: Self) -> Fallible<Self>;
}

macro_rules! impl_accumulator_for_float {
    ($t:ty) => {
        impl Accumulator for $t {
            fn relaxation(size_limit: usize, lower: Self, upper: Self) -> Fallible<Self> {
                if Sequential::<$t>::can_float_sum_overflow(size_limit, (lower, upper))? {
                    return fallible!(
                        MakeTransformation,
                        "potential for overflow when computing function. You could resolve this by choosing tighter clipping bounds."
                    );
                }
                Sequential::<$t>::relaxation(size_limit, lower, upper)
            }
        }
    };
}

impl_accumulator_for_float!(f32);
impl_accumulator_for_float!(f64);

macro_rules! impl_accumulator_for_int {
    ($($t:ty)+) => {
        $(impl Accumulator for $t {
            fn relaxation(size_limit: usize, lower: Self, upper: Self) -> Fallible<Self> {
                if can_int_sum_overflow(size_limit, (lower, upper)) {
                    return fallible!(
                        MakeTransformation,
                        "potential for overflow when computing function. You could resolve this by choosing tighter clipping bounds or by using a data type with greater bit-depth."
                    );
                }
                Ok(0)
            }
        })+
    };
}
impl_accumulator_for_int!(u64 i64 u32 i32);

#[cfg(test)]
mod test;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{AtomDomain, Context, Margin, WildExprDomain};
use crate::domains::{ExprDomain, MarginPub::Lengths, NumericDataType, SeriesDomain};
use crate::error::*;
use crate::metrics::{IntDistance, LpDistance, PartitionDistance};
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
use polars_plan::plans::{typed_lit, TypedLiteral};

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
    input_domain: WildExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    Expr: StableExpr<PartitionDistance<MI>, PartitionDistance<MI>>,
{
    let Expr::Agg(AggExpr::Sum(input)) = expr else {
        return fallible!(MakeTransformation, "expected sum expression");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let (by, input_margin) = middle_domain.context.grouping("sum")?;

    if middle_domain.column.nullable {
        return fallible!(
            MakeTransformation,
            "input data ({}) might contain nulls. Preprocess your data with `.fill_null`.",
            (*input).clone().meta().output_name()?
        );
    }

    let dtype = middle_domain.column.dtype();

    use DataType::*;

    let nan = match dtype {
        Float32 => middle_domain.column.atom_domain::<f32>()?.nullable(),
        Float64 => middle_domain.column.atom_domain::<f64>()?.nullable(),
        _ => false,
    };

    if nan {
        return fallible!(
            MakeTransformation,
            "input data ({}) might contain nans. Preprocess your data with `.fill_nan`.",
            (*input).clone().meta().output_name()?
        );
    }

    let stability_map = match dtype {
        UInt32 => sum_stability_map::<MI, P, u32>(&middle_domain),
        UInt64 => sum_stability_map::<MI, P, u64>(&middle_domain),
        Int8 => sum_stability_map::<MI, P, i8>(&middle_domain),
        Int16 => sum_stability_map::<MI, P, i16>(&middle_domain),
        Int32 => sum_stability_map::<MI, P, i32>(&middle_domain),
        Int64 => sum_stability_map::<MI, P, i64>(&middle_domain),
        Float32 => sum_stability_map::<MI, P, f32>(&middle_domain),
        Float64 => sum_stability_map::<MI, P, f64>(&middle_domain),
        _ => fallible!(MakeTransformation, "unsupported data type"),
    }?;

    let name = middle_domain.column.name.clone();

    let (series_domain, fill_value) = match dtype {
        UInt32 => sum_components::<u32>(name),
        UInt64 => sum_components::<u64>(name),
        Int8 => sum_components::<i8>(name),
        Int16 => sum_components::<i16>(name),
        Int32 => sum_components::<i32>(name),
        Int64 => sum_components::<i64>(name),
        Float32 => sum_components::<f32>(name),
        Float64 => sum_components::<f64>(name),
        _ => fallible!(MakeTransformation, "unsupported data type"),
    }?;

    // build output domain
    let output_domain = ExprDomain {
        column: series_domain,
        context: Context::Grouping {
            by,
            margin: Margin {
                max_partition_length: Some(1),
                max_num_partitions: input_margin.max_num_partitions,
                max_partition_contributions: None,
                max_influenced_partitions: Some(1),
                public_info: input_margin.public_info,
            },
        },
    };

    t_prior
        >> Transformation::<_, _, PartitionDistance<MI>, LpDistance<P, _>>::new(
            middle_domain,
            output_domain,
            Function::then_expr(Expr::sum).fill_with(fill_value),
            middle_metric.clone(),
            LpDistance::default(),
            stability_map,
        )?
}

fn sum_components<TI>(name: PlSmallStr) -> Fallible<(SeriesDomain, Expr)>
where
    TI: Summand,
    TI::Sum: Zero + TypedLiteral,
{
    Ok((
        SeriesDomain::new(name, AtomDomain::<TI::Sum>::default()),
        typed_lit(TI::Sum::zero()),
    ))
}

fn sum_stability_map<MI, const P: usize, TI>(
    domain: &ExprDomain,
) -> Fallible<StabilityMap<PartitionDistance<MI>, LpDistance<P, f64>>>
where
    MI: UnboundedMetric,
    TI: Summand,
    f64: InfCast<TI::Sum> + InfCast<u32>,
{
    let margin = domain.context.grouping("sum")?.1;
    let (l, u) = domain.column.atom_domain::<TI>()?.get_closed_bounds()?;
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

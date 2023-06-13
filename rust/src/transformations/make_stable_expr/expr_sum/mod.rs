use crate::core::{ExprFunction, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, Margin, MarginPub, NumericDataType, SeriesDomain};
use crate::error::*;
use crate::metrics::{IntDistance, LpDistance, PartitionDistance};
use crate::traits::{
    AlertingAbs, CheckAtom, ExactIntCast, InfAdd, InfCast, InfMul, InfSqrt, InfSub, Number,
    ProductOrd,
};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{CanFloatSumOverflow, CanIntSumOverflow, Sequential, SumRelaxation};
use num::Zero;
use polars::prelude::*;
use std::collections::HashMap;

use super::StableExpr;

/// Polars operator to sum the active column in a LazyFrame
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
        return fallible!(MakeTransformation, "expected function expression");
    };

    let t_prior = prior_expr.make_stable(input_domain, input_metric)?;
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
    let input_margin = (output_domain.frame_domain.margins.remove(&margin_id))
        .ok_or_else(|| err!(MakeTransformation, "failed to find margin {:?}", margin_id))?;

    // Set the margins on the output domain to consist of only one margin: 1 row per group, with at most 1 record in each group.
    let output_margin = input_margin.clone().with_max_partition_length(1);
    output_domain.frame_domain.margins = HashMap::from_iter([(margin_id, output_margin)]);

    fn stability_map<MI, const P: usize, TI>(
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

        let public_info = margin
            .public_info
            .ok_or_else(|| err!(MakeTransformation, "keys must be public information"))?;

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
            MarginPub::Keys => TI::Sum::inf_cast(*d_in)?.inf_mul(&l.alerting_abs()?.total_max(u)?),
            MarginPub::Lengths => TI::Sum::inf_cast(*d_in / 2)?.inf_mul(&u.inf_sub(&l)?),
        };

        Ok(StabilityMap::new_fallible(
            move |(l0, l1, li): &(IntDistance, IntDistance, IntDistance)| {
                // max changed partitions
                let mcp = if pp_relaxation.is_zero() {
                    0u32
                } else if MI::ORDERED {
                    (*l0).min(margin.max_num_partitions.unwrap_or(*l0))
                } else {
                    margin.max_num_partitions
                        .ok_or_else(|| err!(FailedFunction, "max_num_partitions must be known when the metric is not sensitive to ordering (SymmetricDistance)"))?
                };

                let mcp_p = norm_map(f64::from(mcp))?;
                let l0_p = norm_map(f64::from(*l0))?;
                let l1_p = f64::inf_cast(pp_map(l1)?)?;
                let li_p = f64::inf_cast(pp_map(li)?)?;

                let relaxation = mcp_p.inf_mul(&pp_relaxation)?;

                l1_p.total_min(l0_p.inf_mul(&li_p)?)?.inf_add(&relaxation)
            },
        ))
    }

    let series_domain = middle_domain.active_series()?.clone();
    let stability_map = match dtype {
        DataType::UInt32 => stability_map::<MI, P, u32>(series_domain, input_margin),
        DataType::UInt64 => stability_map::<MI, P, u64>(series_domain, input_margin),
        DataType::Int8 => stability_map::<MI, P, i8>(series_domain, input_margin),
        DataType::Int16 => stability_map::<MI, P, i16>(series_domain, input_margin),
        DataType::Int32 => stability_map::<MI, P, i32>(series_domain, input_margin),
        DataType::Int64 => stability_map::<MI, P, i64>(series_domain, input_margin),
        DataType::Float32 => stability_map::<MI, P, f32>(series_domain, input_margin),
        DataType::Float64 => stability_map::<MI, P, f64>(series_domain, input_margin),
        _ => fallible!(MakeTransformation, "unsupported data type"),
    }?;

    t_prior
        >> Transformation::<_, _, PartitionDistance<MI>, LpDistance<P, _>>::new(
            middle_domain,
            output_domain,
            Function::new_expr(Expr::sum),
            middle_metric.clone(),
            LpDistance::default(),
            stability_map,
        )?
}

pub trait Summand: NumericDataType + CheckAtom + ProductOrd {
    type Sum: Accumulator + InfCast<Self>;
}

// Determine the resulting type of the sum of a column of a given type.
// ```python
// import polars as pl
// types = [pl.Int8, pl.Int16, pl.Int32, pl.Int64,
//          pl.UInt8, pl.UInt16, pl.UInt32, pl.UInt64
//          pl.Float32, pl.Float64]
//
// def sum_type(ty):
//     df = pl.LazyFrame({"": [12, 1]}, schema_overrides={"": ty})
//     return df.select(pl.col("").sum()).collect()[""].dtype
//
// print({v: sum_type(v) for v in types})
// ```

// When the input is [key], the output is [value]:
// {Int8: Int64, Int16: Int64, Int32: Int32, Int64: Int64, UInt8: Int64, UInt16: Int64, UInt32: UInt32, UInt64: UInt64, Float32: Float32, Float64: Float64}

// This relationship between types is encoded in the associated type `Sum` of the `Summand` trait:
macro_rules! impl_summand {
    ($ti:ty, $to:ty) => {
        impl Summand for $ti {
            type Sum = $to;
        }
    };
}

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
                if Sequential::<$t>::float_sum_can_overflow(size_limit, (lower, upper))? {
                    return fallible!(
                        MakeTransformation,
                        "potential for overflow when computing function"
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
                if <$t>::int_sum_can_overflow(size_limit, (lower, upper))? {
                    return fallible!(
                        MakeTransformation,
                        "potential for overflow when computing function"
                    );
                }
                Ok(0)
            }
        })+
    };
}
impl_accumulator_for_int!(u64 i64 u32 i32);

#[cfg(test)]
mod test_make_sum_expr {

    use crate::{
        metrics::{InsertDeleteDistance, L2Distance, PartitionDistance},
        transformations::polars_test::get_test_data,
    };

    use super::*;

    #[test]
    fn test_select_make_sum_expr() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.aggregate(["chunk_2_bool", "cycle_5_alpha"]);

        // Get resulting sum (expression result)
        let t_sum: Transformation<_, _, _, L2Distance<f64>> = col("const_1f64")
            .clip(lit(0), lit(1))
            .sum()
            .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;
        let expr_res = t_sum.invoke(&(lf.logical_plan.clone(), all()))?.1;
        // dtype in clip changes
        assert_eq!(expr_res, col("const_1f64").clip(lit(0.), lit(1.)).sum());

        let sens = t_sum.map(&(4, 4, 1))?;
        println!("sens: {:?}", sens);
        assert!(sens > (2.).into());
        assert!(sens < (2.00001).into());
        Ok(())
    }

    #[test]
    fn test_grouped_make_sum_expr() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.aggregate(["chunk_(..10u32)"]);

        // Get resulting sum (expression result)
        let t_sum: Transformation<_, _, _, L2Distance<f64>> = col("cycle_(..100i32)")
            .clip(lit(0), lit(1))
            .sum()
            .clone()
            .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;
        let expr_res = t_sum.invoke(&(lf.logical_plan.clone(), all()))?.1;

        let df_actual = lf
            .group_by(["chunk_(..10u32)"])
            .agg([expr_res])
            .collect()?
            .sort(["chunk_(..10u32)"], false, false)?;

        let df_expected = df!(
            "chunk_(..10u32)" => [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            "cycle_(..100i32)" => [99i32; 10]
        )?;

        assert_eq!(df_actual, df_expected);

        // assume we're in a grouping context.
        // By the following triple, we know
        // 1. an individual can influence up to 10 partitions (l0)
        // 2. an individual can contribute up to 10 records total (l1)
        // 3. an individual can contribute at most 1 record to any partition (linf)
        let sens = t_sum.map(&(10, 10, 1))?;

        // The sensitivity d_out under the l2 distance in unbounded DP is given by the following formula:
        // = min(sqrt(l0) * map(linf)         , map(l1))
        // = min(sqrt(l0) * linf * max(|L|, U), l1 * max(|L|, U))
        // = min(sqrt(10) * 1, 10)
        // = min(3.16227, 10)
        // = 3.16227

        // that is, in the worst case, we know the sum will differ by at most 1 in 10 partitions,
        // so the l2 distance between any two outputs on neighboring data sets is at most 3.16227

        // The sensitivity is slightly higher to account for potential rounding errors.
        println!("sens: {:?}", sens);
        assert!(sens > (3.16227).into());
        assert!(sens < (3.162278).into());
        Ok(())
    }
}

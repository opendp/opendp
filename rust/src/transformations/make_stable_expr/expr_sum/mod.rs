use crate::core::{
    ExprFunction, Function, Metric, MetricSpace, Scalar, StabilityMap, Transformation,
};
use crate::domains::{ExprDomain, Margin, MarginPub, NumericDataType, SeriesDomain};
use crate::error::*;
use crate::metrics::{IntDistance, L1Distance, L2Distance, LpDistance, PartitionDistance};
use crate::traits::{
    AlertingAbs, CheckAtom, ExactIntCast, Float, InfAdd, InfCast, InfMul, InfSqrt, InfSub, Number,
    ProductOrd,
};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{CanFloatSumOverflow, CanIntSumOverflow, Sequential, SumRelaxation};
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
/// * `expr` - sum expression
pub fn make_expr_sum<MI, MX, const P: usize>(
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, Scalar>>>
where
    MI: 'static + Metric,
    MX: 'static + UnboundedMetric,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
    (ExprDomain, PartitionDistance<MX>): MetricSpace,
    Expr: StableExpr<PartitionDistance<MI>, PartitionDistance<MX>>,
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

    fn stability_map<MX, const P: usize, TI>(
        series_domain: SeriesDomain,
        margin: Margin,
    ) -> Fallible<StabilityMap<PartitionDistance<MX>, LpDistance<P, Scalar>>>
    where
        MX: UnboundedMetric,
        TI: Summand,
        Scalar: From<TI::Sum>,
        f64: InfCast<TI::Sum>,
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
        let pp_relaxation = TI::Sum::relaxation(max_size, l, u)?;

        let pp_map = move |d_in: &IntDistance| match public_info {
            MarginPub::Keys => TI::Sum::inf_cast(*d_in)?.inf_mul(&l.alerting_abs()?.total_max(u)?),
            MarginPub::Lengths => TI::Sum::inf_cast(*d_in / 2)?.inf_mul(&u.inf_sub(&l)?),
        };

        Ok(StabilityMap::new_fallible(move |(l0, l1, li)| {
            // max changed partitions
            let mcp = if MX::ORDERED {
                *l0
            } else {
                margin.max_num_partitions
                    .ok_or_else(|| err!(FailedFunction, "max_partitions must be set when the metric is not sensitive to ordering (SymmetricDistance)"))?
            };

            let (l0, l1, li) = (TI::Sum::inf_cast(*l0)?, pp_map(l1)?, pp_map(li)?);

            let relaxation = TI::Sum::inf_cast(mcp)?.inf_mul(&pp_relaxation)?;

            Ok(if P == 1 {
                let ideal_sens = l1.total_min(l0.inf_mul(&li)?)?;
                Scalar::from(ideal_sens.inf_add(&relaxation)?)
            } else if P == 2 {
                let (l0, l1, li) = (f64::inf_cast(l0)?, f64::inf_cast(l1)?, f64::inf_cast(li)?);
                let relaxation = f64::inf_cast(relaxation)?;
                let ideal_sens = l1.total_min(l0.inf_sqrt()?.inf_mul(&li)?)?;
                Scalar::F64(ideal_sens.inf_add(&relaxation)?)
            } else {
                return fallible!(MakeTransformation, "unsupported Lp norm");
            })
        }))
    }

    let series_domain = middle_domain.active_series()?.clone();
    let stability_map = match dtype {
        DataType::UInt32 => stability_map::<MX, P, u32>(series_domain, input_margin),
        DataType::UInt64 => stability_map::<MX, P, u64>(series_domain, input_margin),
        DataType::Int8 => stability_map::<MX, P, i8>(series_domain, input_margin),
        DataType::Int16 => stability_map::<MX, P, i16>(series_domain, input_margin),
        DataType::Int32 => stability_map::<MX, P, i32>(series_domain, input_margin),
        DataType::Int64 => stability_map::<MX, P, i64>(series_domain, input_margin),
        DataType::Float32 => stability_map::<MX, P, f32>(series_domain, input_margin),
        DataType::Float64 => stability_map::<MX, P, f64>(series_domain, input_margin),
        _ => fallible!(MakeTransformation, "unsupported data type"),
    }?;

    t_prior
        >> Transformation::new(
            middle_domain,
            output_domain,
            Function::new_expr(Expr::sum),
            middle_metric.clone(),
            LpDistance::default(),
            stability_map,
        )?
}

trait SumStabilityMap<MI: Metric, MO: Metric> {
    fn make_stability_map(
        margin: Margin,
        input_metric: MI,
        output_metric: MO,
        max_size: usize,
    ) -> Fallible<StabilityMap<MI, MO>>
    where
        (ExprDomain, MI): MetricSpace,
        (ExprDomain, MO): MetricSpace;
}

// 1. spread into triple
// aggregate  | l0    l1    li
// select     | 1     d_in  d_in

// 2. define map(d)
// known N    | d // 2 * (U - L)
// unknown N  | d * max(|L|, U)

// 3. ideal sensitivity
// l1distance | min(map(l1), l0 * map(li))
// l2distance | min(map(l1), sqrt(l0) * map(li))

// 4. max changed partitions
//             SymmetricDistance     InsertDeleteDistance
// aggregate  | max_partitions      | max_partitions.min(l0)
// select     | 1                   | 1

// 5. relaxation
// l1distance | mcp       * relaxation(max_partition_len, L, U)
// l2distance | sqrt(mcp) * relaxation(max_partition_len, L, U)

// 6. actual sensitivity = ideal sensitivity + relaxation

trait SumOutputMetric: Metric<Distance = Scalar> {}
impl SumOutputMetric for L1Distance<Scalar> {}
impl SumOutputMetric for L2Distance<Scalar> {}

// 3. ideal sensitivity
trait NativeSumOutputMetric: Metric {
    type SumOutputMetric: SumOutputMetric;
    fn ideal_sensitivity(
        l0: Self::Distance,
        l1: Self::Distance,
        li: Self::Distance,
    ) -> Fallible<Self::Distance>;
}

impl<Q: Number> NativeSumOutputMetric for L1Distance<Q> {
    type SumOutputMetric = L1Distance<Scalar>;
    fn ideal_sensitivity(
        l0: Self::Distance,
        l1: Self::Distance,
        li: Self::Distance,
    ) -> Fallible<Self::Distance> {
        l1.total_min(l0.inf_mul(&li)?)
    }
}

impl<Q: Float> NativeSumOutputMetric for L2Distance<Q> {
    type SumOutputMetric = L2Distance<Scalar>;
    fn ideal_sensitivity(
        l0: Self::Distance,
        l1: Self::Distance,
        li: Self::Distance,
    ) -> Fallible<Self::Distance> {
        l1.total_min(l0.inf_sqrt()?.inf_mul(&li)?)
    }
}

pub trait Summand: NumericDataType + CheckAtom + ProductOrd {
    type Sum: Accumulator + InfCast<Self> + InfCast<IntDistance> + ProductOrd;
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
        metrics::{InsertDeleteDistance, PartitionDistance, SymmetricDistance},
        transformations::polars_test::get_test_data,
    };

    use super::*;

    #[test]
    fn test_select_make_sum_expr() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.aggregate(["C"]);
        let expr_exp = col("B").clip(lit(1.), lit(2.)).sum();

        // Get resulting sum (expression result)
        let t_sum: Transformation<_, _, _, L2Distance<Scalar>> = expr_exp
            .clone()
            .make_stable(expr_domain, PartitionDistance(SymmetricDistance))?;
        let expr_res = t_sum.invoke(&(lf.logical_plan.clone(), all()))?.1;
        assert_eq!(expr_exp, expr_res);

        let sens = t_sum.map(&(1, 1, 1))?;
        println!("sens: {:?}", sens);
        assert!(sens > (2.).into());
        assert!(sens < (2.00001).into());
        Ok(())
    }

    #[test]
    fn test_grouped_make_sum_expr() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.aggregate(["C"]);
        let expr_exp = col("B").clip(lit(0.), lit(1.)).sum();

        // Get resulting sum (expression result)
        let t_sum: Transformation<_, _, _, L2Distance<Scalar>> = expr_exp
            .clone()
            .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;
        let expr_res = t_sum.invoke(&(lf.logical_plan.clone(), all()))?.1;
        assert_eq!(expr_exp, expr_res);

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

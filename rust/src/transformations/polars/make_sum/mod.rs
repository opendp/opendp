use crate::core::{ExprFunction, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, LazyDomain, NumericDataType, OuterMetric};
use crate::error::*;
use crate::metrics::{AbsoluteDistance, InsertDeleteDistance, IntDistance, SymmetricDistance, L1};
use crate::traits::{CheckAtom, ExactIntCast, InfAdd, InfCast, InfMul, InfSub, Number, TotalOrd};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{
    CanFloatSumOverflow, CanIntSumOverflow, DatasetMetric, Sequential, SumRelaxation,
};
use opendp_derive::bootstrap;
use polars::prelude::*;
use std::collections::{BTreeSet, HashMap};

use super::item;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(ffi = false)]
/// Polars operator to sum the active column in a LazyFrame
///
/// | input_metric               | input_domain                     |
/// | -------------------------- | -------------------------------- |
/// | `SymmetricDistance`        | `ExprDomain<LazyFrameDomain>`   |
/// | `InsertDeleteDistance`     | `ExprDomain<LazyFrameDomain>`   |
/// | `L1<SymmetricDistance>`    | `ExprDomain<LazyGroupByDomain>` |
/// | `L1<InsertDeleteDistance>` | `ExprDomain<LazyGroupByDomain>` |
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - valid selections shown in table above
pub fn make_sum_expr<MI, TI>(
    input_domain: ExprDomain<MI::LazyDomain>,
    input_metric: MI,
) -> Fallible<
    Transformation<ExprDomain<MI::LazyDomain>, ExprDomain<MI::LazyDomain>, MI, MI::SumMetric>,
>
where
    MI: SumOuterMetric<TI>,
    MI::InnerMetric: UnboundedMetric,
    TI: Summand,
    TI::Sum: InfCast<u32> + InfCast<TI>,

    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
    (ExprDomain<MI::LazyDomain>, MI::SumMetric): MetricSpace,
{
    let input_dtype = &input_domain.active_series()?.field.dtype;
    if input_dtype != &TI::dtype() {
        return fallible!(
            MakeTransformation,
            "based on the input domain, TI should be {:?} but is {:?}",
            input_dtype,
            TI::dtype()
        );
    }

    // check that we are in a context where it is ok to break row-alignment
    input_domain.context.break_alignment()?;

    // build output domain
    let mut output_domain = input_domain.clone();

    // summation invalidates bounds on the active column
    (output_domain.lazy_frame_domain)
        .try_column_mut(input_domain.active_column()?)?
        .drop_bounds()?;

    // we only care about the margin that matches the current grouping columns
    let margin_id = BTreeSet::from_iter(input_domain.context.grouping_columns());
    let input_margin = (output_domain.lazy_frame_domain.margins.remove(&margin_id))
        .ok_or_else(|| err!(MakeTransformation, "failed to find margin {:?}", margin_id))?;

    // Set the margins on the output domain to consist of only one margin: 1 row per group, with all counts set to 1.
    let mut output_margin = input_margin.clone();
    output_margin.data = (output_margin.data.clone())
        .with_column(lit(1u32).alias(output_margin.get_count_column_name()?.as_str()));
    output_domain.lazy_frame_domain.margins = HashMap::from_iter([(margin_id, output_margin)]);

    // prepare stability map
    let (l, u) = input_domain
        .active_series()?
        .atom_domain::<TI>()?
        .get_closed_bounds()?;
    let (lower, upper) = (TI::Sum::inf_cast(l)?, TI::Sum::neg_inf_cast(u)?);
    let ideal_sensitivity = upper.inf_sub(&lower)?;

    let max_size = usize::exact_int_cast(input_margin.get_max_size()?)?;

    let per_partition_relaxation = TI::Sum::relaxation(max_size, lower, upper)?;
    let max_partitions = item::<u32>(input_margin.data.clone().select([len()]))?;

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_expr(Expr::sum),
        input_metric.clone(),
        MI::SumMetric::default(),
        StabilityMap::new_fallible(move |&d_in: &IntDistance| {
            let max_changed_partitions =
                TI::Sum::inf_cast(if !<MI::LazyDomain as LazyDomain>::Context::GROUPBY {
                    1
                } else if MI::InnerMetric::ORDERED {
                    max_partitions.min(d_in)
                } else {
                    max_partitions
                })?;

            TI::Sum::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&max_changed_partitions.inf_mul(&per_partition_relaxation)?)
        }),
    )
}

/// Requirements for a metric that can be used in a sum transformation.
pub trait SumOuterMetric<TI: Summand>: OuterMetric<Distance = IntDistance> {
    type SumMetric: OuterMetric<
        // the inner metric will always be an absolute distance
        InnerMetric = AbsoluteDistance<TI::Sum>,
        // regardless the choice of metric, the distance type will always be QO
        Distance = TI::Sum,
        // the type of the domain will never change
        LazyDomain = Self::LazyDomain,
    >;
}

impl<TI: Summand> SumOuterMetric<TI> for InsertDeleteDistance {
    type SumMetric = AbsoluteDistance<TI::Sum>;
}
impl<TI: Summand> SumOuterMetric<TI> for L1<InsertDeleteDistance> {
    type SumMetric = L1<AbsoluteDistance<TI::Sum>>;
}
impl<TI: Summand> SumOuterMetric<TI> for SymmetricDistance {
    type SumMetric = AbsoluteDistance<TI::Sum>;
}
impl<TI: Summand> SumOuterMetric<TI> for L1<SymmetricDistance> {
    type SumMetric = L1<AbsoluteDistance<TI::Sum>>;
}

pub trait Summand: NumericDataType + CheckAtom + TotalOrd {
    type Sum: Accumulator + InfCast<Self> + TotalOrd;
}

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

// {Int8: Int64, Int16: Int64, Int32: Int32, Int64: Int64, UInt8: Int64, UInt16: Int64, UInt32: UInt32, UInt64: UInt64, Float32: Float32, Float64: Float64}

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
// impl_summand!(u8, i64);
// impl_summand!(u16, i64);
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
        domains::Margin,
        metrics::Lp,
        transformations::polars_test::{get_grouped_test_data, get_select_test_data},
    };

    use super::*;

    #[test]
    fn test_select_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr::<_, f64>(expr_domain, InsertDeleteDistance)?;
        let expr_res = trans.invoke(&(lazy_frame.clone(), col("B")))?.1;
        let frame_actual = (*lazy_frame).clone().clone().select([expr_res]).collect()?;

        // Get expected sum
        let frame_expected = (*lazy_frame).clone().select([col("B").sum()]).collect()?;

        assert_eq!(frame_actual, frame_expected);

        let sens = trans.map(&2)?;
        println!("sens: {:?}", sens);
        assert!(sens > 2.);
        assert!(sens < 2.00001);
        Ok(())
    }

    #[test]
    fn test_grouped_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_grouped_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr::<_, f64>(expr_domain, Lp(InsertDeleteDistance))?;
        let expr_res = trans.invoke(&(lazy_frame.clone(), col("B")))?.1;
        let frame_actual = (*lazy_frame)
            .clone()
            .agg([expr_res])
            .sort("A", Default::default())
            .collect()?;

        // Get expected sum
        let frame_expected = (*lazy_frame)
            .clone()
            .agg([col("B").sum()])
            .sort("A", Default::default())
            .collect()?;

        assert_eq!(frame_actual, frame_expected);

        let sens = trans.map(&2)?;
        println!("sens: {:?}", sens);

        assert!(sens > 2.);
        assert!(sens < 2.00001);
        Ok(())
    }

    #[test]
    fn test_make_sum_expr_output_domain() -> Fallible<()> {
        let (mut expr_domain, _) = get_grouped_test_data()?;

        let output_domain_res =
            make_sum_expr::<_, f64>(expr_domain.clone(), Lp(InsertDeleteDistance))?
                .output_domain
                .clone();

        let margin_id = BTreeSet::from_iter(expr_domain.context.grouping_columns());
        let margin = Margin::new_from_counts(
            df!("A" => &[1, 2], "count" => &[1u32, 1])?.lazy(),
            "count".to_string(),
        )?;

        expr_domain.lazy_frame_domain.margins.clear();
        (expr_domain.lazy_frame_domain.margins).insert(margin_id, margin);

        assert_eq!(output_domain_res, expr_domain);

        Ok(())
    }
}

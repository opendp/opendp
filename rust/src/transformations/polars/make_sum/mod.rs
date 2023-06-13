use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    item, Context, DataTypeFrom, DatasetMetric, ExprDomain, ExprMetric, LazyFrameContext,
    LazyGroupByContext,
};
use crate::error::*;
use crate::metrics::{
    AbsoluteDistance, InsertDeleteDistance, IntDistance, L1Distance, SymmetricDistance,
};
use crate::traits::{ExactIntCast, Number};
use crate::transformations::{CanFloatSumOverflow, CanIntSumOverflow, Sequential, SumRelaxation};
use polars::prelude::*;
use std::collections::BTreeSet;

use num::ToPrimitive;

/// Polars operator to compute sum of a series in a LazyFrame
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
pub trait ContextInSum<QO>: Context {
    type OutputMetric: ExprMetric<Self, InnerMetric = AbsoluteDistance<QO>, Distance = QO>;
}

impl<QO> ContextInSum<QO> for LazyGroupByContext {
    type OutputMetric = L1Distance<QO>;
}

impl<QO> ContextInSum<QO> for LazyFrameContext {
    type OutputMetric = AbsoluteDistance<QO>;
}

pub trait SumPrimitive: Sized {
    fn relaxation(size_limit: usize, lower: Self, upper: Self) -> Fallible<Self>;
    fn check_overflow(size_limit: usize, lower: Self, upper: Self) -> Fallible<()>;
}

pub trait SumDatasetMetric: DatasetMetric {
    fn max_changed_partitions(
        &self,
        num_partitions: usize,
        d_in: &Self::Distance,
    ) -> Fallible<usize>;
}

impl SumDatasetMetric for InsertDeleteDistance {
    fn max_changed_partitions(
        &self,
        _num_partitions: usize,
        d_in: &Self::Distance,
    ) -> Fallible<usize> {
        usize::exact_int_cast(*d_in)
    }
}
impl SumDatasetMetric for SymmetricDistance {
    fn max_changed_partitions(
        &self,
        num_partitions: usize,
        _d_in: &Self::Distance,
    ) -> Fallible<usize> {
        Ok(num_partitions)
    }
}

macro_rules! impl_sum_primitive_for_float {
    ($t:ty) => {
        impl SumPrimitive for $t {
            fn relaxation(size_limit: usize, lower: Self, upper: Self) -> Fallible<Self> {
                Sequential::<$t>::relaxation(size_limit, lower, upper)
            }
            fn check_overflow(size_limit: usize, lower: Self, upper: Self) -> Fallible<()> {
                if Sequential::<$t>::float_sum_can_overflow(size_limit, (lower, upper))? {
                    return fallible!(
                        MakeTransformation,
                        "potential for overflow when computing function"
                    );
                }
                Ok(())
            }
        }
    };
}

impl_sum_primitive_for_float!(f32);
impl_sum_primitive_for_float!(f64);

macro_rules! impl_sum_primitive_for_int {
    ($t:ty) => {
        impl SumPrimitive for $t {
            fn relaxation(_size_limit: usize, _lower: Self, _upper: Self) -> Fallible<Self> {
                Ok(0)
            }
            fn check_overflow(size_limit: usize, lower: Self, upper: Self) -> Fallible<()> {
                if <$t>::int_sum_can_overflow(size_limit, (lower, upper))? {
                    return fallible!(
                        MakeTransformation,
                        "potential for overflow when computing function"
                    );
                }
                Ok(())
            }
        }
    };
}
impl_sum_primitive_for_int!(u64);
impl_sum_primitive_for_int!(i64);

pub fn make_sum_expr<MI, C: ContextInSum<QO>, QO>(
    input_domain: ExprDomain<C>,
    input_metric: MI,
) -> Fallible<Transformation<ExprDomain<C>, ExprDomain<C>, MI, C::OutputMetric>>
where
    MI: 'static + ExprMetric<C, Distance = IntDistance> + Send + Sync,
    MI::InnerMetric: SumDatasetMetric,
    QO: Number + SumPrimitive + DataTypeFrom + ExactIntCast<i64>,

    (ExprDomain<C>, MI): MetricSpace,
    (ExprDomain<C>, C::OutputMetric): MetricSpace,
{
    // check that input type is one of the typs that result in QO
    let active_column = input_domain.clone().active_column()?;
    let lf_domain = input_domain.lazy_frame_domain.clone();
    let input_dtype = (lf_domain.try_column(active_column.clone())?)
        .field
        .dtype
        .clone();
    let output_dtype = QO::dtype();

    let output_dtype_matches = match output_dtype {
        // TODO: check/add other variants for ints
        DataType::UInt64 => [DataType::UInt64, DataType::UInt32].contains(&input_dtype),
        DataType::Int64 => [DataType::Int8, DataType::Int32].contains(&input_dtype),
        DataType::Float32 => DataType::Float32 == input_dtype,
        DataType::Float64 => DataType::Float64 == input_dtype,
        _ => return fallible!(MakeTransformation, "unsupported output type"),
    };

    if !output_dtype_matches {
        return fallible!(
            MakeTransformation,
            "input dtype does not result in output dtype!"
        );
    }

    // output domain
    let mut output_domain = input_domain.clone();
    (output_domain.lazy_frame_domain)
        .try_column_mut(input_domain.active_column()?)?
        .drop_bounds()?;
    output_domain.aligned = false;

    // stability map

    // 1. compute relaxation (via trait helper)
    // 2. scale the relaxation based on number of affected partitions
    // 3. compute the sensitivity

    macro_rules! impl_convert_bounds_int {
        ($t:ty) => {{
            let bounds = input_domain
                .active_series()?
                .atom_domain::<$t>()?
                .get_closed_bounds()?;
            let lower = (bounds.0.to_i64())
                .ok_or_else(|| err!(MakeTransformation, "failed to convert bound to i64"))?;
            let upper = (bounds.1.to_i64())
                .ok_or_else(|| err!(MakeTransformation, "failed to convert bound to i64"))?;
            (QO::exact_int_cast(lower)?, QO::exact_int_cast(upper)?)
        }};
    }

    let (lower, upper) = match input_dtype {
        DataType::UInt8 => impl_convert_bounds_int!(u8),
        DataType::UInt16 => impl_convert_bounds_int!(u16),
        DataType::UInt32 => impl_convert_bounds_int!(u32),
        DataType::UInt64 => impl_convert_bounds_int!(u64),
        DataType::Int8 => impl_convert_bounds_int!(i8),
        DataType::Int16 => impl_convert_bounds_int!(i16),
        DataType::Int32 => impl_convert_bounds_int!(i32),
        DataType::Int64 => impl_convert_bounds_int!(i64),
        DataType::Float32 => input_domain
            .active_series()?
            .atom_domain::<QO>()?
            .get_closed_bounds()?,
        DataType::Float64 => input_domain
            .active_series()?
            .atom_domain::<QO>()?
            .get_closed_bounds()?,
        _ => return fallible!(MakeTransformation, "unsupported input type"),
    };

    let ideal_sensitivity = upper.inf_sub(&lower)?;

    let margin = input_domain
        .lazy_frame_domain
        .margins
        .get(&BTreeSet::from_iter(
            input_domain.context.grouping_columns(),
        ))
        .ok_or_else(|| err!(MakeTransformation, "failed to find margin"))?;
    
    let max_size = margin.get_max_size()? as usize;

    QO::check_overflow(max_size, lower, upper)?;

    let relaxation = QO::relaxation(max_size, lower, upper)?;

    let num_partitions = item::<u32>(margin.data.clone().select([count()]))? as usize;

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new(
            move |(frame, expr): &(C::Value, Expr)| -> (C::Value, Expr) {
                (frame.clone(), expr.clone().sum())
            },
        ),
        input_metric.clone(),
        C::OutputMetric::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            let max_changed_partitions = QO::exact_int_cast(if C::GROUPBY {
                input_metric
                    .inner_metric()
                    .max_changed_partitions(num_partitions, d_in)?
            } else {
                1
            })?;
            QO::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&max_changed_partitions.inf_mul(&relaxation)?)
        }),
    )
}

#[cfg(test)]
mod test_make_sum_expr {
    use crate::{
        domains::{AtomDomain, LazyFrameDomain, LazyGroupByContext, SeriesDomain},
        metrics::Lp,
    };

    use super::*;

    fn get_test_data() -> Fallible<(ExprDomain<LazyGroupByContext>, LazyGroupBy)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::new_closed((1, 4))?),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 5.5))?),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1u32, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2u32, 1]]?.lazy())?;

        let expr_domain = ExprDomain::new(
            frame_domain,
            LazyGroupByContext {
                columns: vec!["A".to_string()],
            },
            Some("B".to_string()),
            true,
        );

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],)?
        .lazy();

        Ok((expr_domain, lazy_frame.groupby([col("A")])))
    }

    #[test]
    fn test_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr::<_, _, f64>(expr_domain, Lp(InsertDeleteDistance))?;
        let expr_res = trans.invoke(&(lazy_frame.clone(), col("B")))?.1;
        let frame_actual = lazy_frame
            .clone()
            .agg([expr_res])
            .sort("A", Default::default())
            .collect()?;

        // Get expected sum
        let frame_expected = lazy_frame
            .agg([col("B").sum()])
            .sort("A", Default::default())
            .collect()?;

        assert_eq!(frame_actual, frame_expected);

        let sens = trans.map(&2)?;
        println!("sens: {:?}", sens);

        assert!(sens > 5.);
        assert!(sens < 5.00001);
        Ok(())
    }

    #[test]
    fn test_make_sum_expr_output_domain() -> Fallible<()> {
        let (mut expr_domain, _) = get_test_data()?;

        let output_domain_res =
            make_sum_expr::<_, _, f64>(expr_domain.clone(), Lp(InsertDeleteDistance))?
                .output_domain
                .clone();

        expr_domain.aligned = false;

        assert_eq!(output_domain_res, expr_domain);

        Ok(())
    }
}

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    item, Context, DataTypeFrom, DatasetMetric, ExprDomain, ExprMetric, LazyFrameContext,
    LazyGroupByContext, Margin,
};
use crate::error::*;
use crate::metrics::{
    AbsoluteDistance, InsertDeleteDistance, IntDistance, L1Distance, SymmetricDistance,
};
use crate::traits::{ExactIntCast, Number};
use crate::transformations::{CanFloatSumOverflow, CanIntSumOverflow, Sequential, SumRelaxation};
use opendp_derive::bootstrap;
use polars::prelude::*;
use std::collections::{BTreeSet, HashMap};

use num::ToPrimitive;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(generics(MI(suppress), QO(suppress)))] 
/// Polars operator to compute sum of a series in a LazyFrame
/// 
/// | input metric               | input domain                     |
/// | -------------------------- | -------------------------------- |
/// | `SymmetricDistance`        | `ExprDomain<LazyFrameContext>`   |
/// | `InsertDeleteDistance`     | `ExprDomain<LazyFrameContext>`   |
/// | `L1<SymmetricDistance>`    | `ExprDomain<LazyGroupByContext>` |
/// | `L1<InsertDeleteDistance>` | `ExprDomain<LazyGroupByContext>` |
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
pub fn make_sum_expr<MI, QO>(
    input_domain: ExprDomain<MI::Context>,
    input_metric: MI,
) -> Fallible<Transformation<ExprDomain<MI::Context>, ExprDomain<MI::Context>, MI, MI::SumMetric>>
where
    MI: 'static + SumExprMetric<Distance = IntDistance> + Send + Sync,
    MI::InnerMetric: SumDatasetMetric,
    QO: Number + SumPrimitive + DataTypeFrom + ExactIntCast<i64>,

    (ExprDomain<MI::Context>, MI): MetricSpace,
    (ExprDomain<MI::Context>, MI::Context::SumMetric): MetricSpace,
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

    // output margins
    // if select, then only one margin, with no grouping keys.
    // if groupby, then only one margin, with current grouping keys. the data is one row per group: old_df, but replace all counts with 1

    // we only care about margins that match the current grouping columns
    let margin_id = BTreeSet::from_iter(input_domain.context.grouping_columns());
    let margin = (input_domain.lazy_frame_domain.margins.get(&margin_id))
        .ok_or_else(|| err!(MakeTransformation, "failed to find margin {:?}", margin_id))?;

    set_sum_margins(&mut output_domain, margin)?;

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

    let max_size = margin.get_max_size()? as usize;
    let num_partitions = item::<u32>(margin.data.clone().select([count()]))? as usize;

    QO::check_overflow(max_size, lower, upper)?;
    let relaxation = QO::relaxation(max_size, lower, upper)?;

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new(
            move |(frame, expr): &(MI::Value, Expr)| -> (MI::Value, Expr) {
                (frame.clone(), expr.clone().sum())
            },
        ),
        input_metric.clone(),
        MI::SumMetric::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            let max_changed_partitions = QO::exact_int_cast(if <MI::Context as Context>::GROUPBY {
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

/// Set the margins on the output domain to consist of only one margin: 1 row per group, with all counts set to 1.
fn set_sum_margins<C: Context>(
    output_domain: &mut ExprDomain<C>,
    sum_margin: &Margin<LazyFrame>,
) -> Fallible<()> {
    let output_margin = Margin {
        data: (sum_margin.data.clone())
            .with_column(lit(1u32).alias(sum_margin.get_count_column_name()?.as_str())),
        counts_index: sum_margin.counts_index,
    };
    output_domain.lazy_frame_domain.margins =
        HashMap::from_iter([(BTreeSet::new(), output_margin)]);
    Ok(())
}

pub trait SumExprMetric<QO>: ExprMetric {
    type SumMetric: ExprMetric<InnerMetric = AbsoluteDistance<QO>, Distance = QO>;
}

impl<QO> SumExprMetric<QO> for LazyGroupByContext {
    type SumMetric = L1Distance<QO>;
}

impl<QO> SumExprMetric<QO> for LazyFrameContext {
    type SumMetric = AbsoluteDistance<QO>;
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

#[cfg(test)]
mod test_make_sum_expr {
    use crate::{
        domains::{AtomDomain, LazyFrameDomain, LazyGroupByContext, SeriesDomain},
        metrics::Lp,
    };

    use super::*;

    pub fn get_select_test_data() -> Fallible<(ExprDomain<LazyFrameContext>, LazyFrame)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 2.5))?),
        ])?
        .with_counts(df!["count" => [3u32]]?.lazy())?
        .with_counts(df!["A" => [1, 2], "count" => [1u32, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2u32, 1]]?.lazy())?;

        let expr_domain = ExprDomain::new(
            frame_domain.clone(),
            LazyFrameContext::Select,
            Some("B".to_string()),
            true,
        );

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],)?
        .lazy();

        Ok((expr_domain, lazy_frame))
    }

    #[test]
    fn test_select_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr::<_, _, f64>(expr_domain, InsertDeleteDistance)?;
        let expr_res = trans.invoke(&(lazy_frame.clone(), col("B")))?.1;
        let frame_actual = lazy_frame.clone().select([expr_res]).collect()?;
        println!("frame_actual: {:?}", frame_actual);

        // Get expected sum
        let frame_expected = lazy_frame.select([col("B").sum()]).collect()?;

        assert_eq!(frame_actual, frame_expected);

        let sens = trans.map(&2)?;
        println!("sens: {:?}", sens);
        assert!(sens > 2.);
        assert!(sens < 2.00001);
        Ok(())
    }

    fn get_grouped_test_data() -> Fallible<(ExprDomain<LazyGroupByContext>, LazyGroupBy)> {
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
    fn test_grouped_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_grouped_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr::<_, _, f64>(expr_domain, Lp(InsertDeleteDistance))?;
        let expr_res = trans.invoke(&(lazy_frame.clone(), col("B")))?.1;
        let frame_actual = lazy_frame
            .clone()
            .agg([expr_res])
            .sort("A", Default::default())
            .collect()?;
        println!("frame_actual: {:?}", frame_actual);

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
        let (mut expr_domain, _) = get_grouped_test_data()?;

        let output_domain_res =
            make_sum_expr::<_, _, f64>(expr_domain.clone(), Lp(InsertDeleteDistance))?
                .output_domain
                .clone();

        expr_domain.aligned = false;

        let margin_id = BTreeSet::from_iter(expr_domain.context.grouping_columns());
        let margin = (expr_domain.lazy_frame_domain.margins.get(&margin_id))
            .ok_or_else(|| err!(MakeTransformation, "failed to find margin {:?}", margin_id))?
            .clone();

        set_sum_margins(&mut expr_domain, &margin)?;

        assert_eq!(output_domain_res, expr_domain);

        Ok(())
    }
}

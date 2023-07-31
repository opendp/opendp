use std::collections::BTreeSet;

use polars::lazy::dsl::Expr;

use crate::{
    core::{Function, Measurement, MetricSpace},
    domains::{item, DataTypeFrom, ExprDomain, ExprMetric},
    error::Fallible,
    measures::MaxDivergence,
    metrics::IntDistance,
    traits::{
        samplers::{CastInternalRational, SampleDiscreteLaplaceZ2k},
        ExactIntCast, Float,
    },
    transformations::{make_sum_expr, ContextInSum, SumDatasetMetric, SumPrimitive},
};

use super::then_laplace_expr;
use polars::prelude::*;

/// Polars operator to compute mean of a series in a LazyFrame
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
/// * `k` - Granularity of the noise in term of 2^k
///
/// # Generics
/// * `MI` - Input Metric.
/// * `C` - Context of the LazyFrame.
/// * `TA` - Data type of the output distance and scale.
pub fn make_private_mean_expr<MI, C: 'static + ContextInSum<TA>, TA>(
    input_domain: ExprDomain<C>,
    input_metric: MI,
    scale: TA,
    k: Option<i32>,
) -> Fallible<Measurement<ExprDomain<C>, Expr, MI, MaxDivergence<TA>>>
where
    MI: 'static + ExprMetric<C, Distance = IntDistance> + Send + Sync,
    MI::InnerMetric: SumDatasetMetric,
    (ExprDomain<C>, MI): MetricSpace,
    (ExprDomain<C>, C::SumMetric): MetricSpace,
    i32: ExactIntCast<TA::Bits>,
    TA: Float
        + CastInternalRational
        + DataTypeFrom
        + SampleDiscreteLaplaceZ2k
        + SumPrimitive
        + ExactIntCast<i64>,
    TA::Polars: PolarsNumericType<Native = TA>,
    Series: NamedFrom<Vec<TA>, [TA]>,
{
    let margins = input_domain.lazy_frame_domain.margins.clone();
    let scale = if C::GROUPBY {
        let margin = margins
            .get(&BTreeSet::from_iter(
                input_domain.context.grouping_columns(),
            ))
            .ok_or_else(|| err!(MakeTransformation, "failed to find margin"))?;
        let min_size = margin.get_min_size()?;
        scale/TA::inf_cast(min_size)?
    } else {
        let margin = (margins.iter())
            .find(|(_, m)| m.counts_index.is_some())
            .ok_or_else(|| err!(MakeTransformation, "failed to find margin"))?
            .1;
        let count_column = margin.get_count_column_name()?;
        let size = item::<u32>(
            margin
                .data
                .clone()
                .select([col(count_column.as_str()).sum()]),
        )? as usize;
        scale/TA::inf_cast(size)?
    };

    make_sum_expr::<_, _, TA>(input_domain, input_metric)?
        >> then_laplace_expr(scale, k)
        >> Function::new(move |sum_expr: &Expr| {
            map_binary(
                sum_expr.clone(),
                count(),
                move |s: Series, count: Series| Ok(Some(s / count)),
                GetOutput::from_type(TA::dtype()),
            )
        })
}

#[cfg(test)]
mod test_make_mean_expr {
    use super::*;
    use crate::{
        measurements::polars::make_laplace::test_make_laplace_expr::{
            get_grouped_test_data, get_test_data,
        },
        metrics::{InsertDeleteDistance, Lp},
    };

    #[test]
    fn test_mean_expr_select() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;
        let scale: f64 = 0.1;

        let meas = make_private_mean_expr::<_, _, f64>(expr_domain, InsertDeleteDistance, scale, None)?;
        let expr_meas = meas.invoke(&(lazy_frame.clone(), col("B")))?;

        let release = lazy_frame.select([expr_meas]).collect()?;
        println!("{:?}", release);

        let sens = meas.map(&2)?;
        println!("sens: {:?}", sens);

        Ok(())
    }

    #[test]
    fn test_mean_expr_groupby() -> Fallible<()> {
        let (expr_domain, group_by) = get_grouped_test_data()?;
        let scale: f64 = 0.1;

        let meas = make_private_mean_expr(expr_domain, Lp(InsertDeleteDistance), scale, None)?;
        let expr_meas = meas.invoke(&(group_by.clone(), col("B")))?;

        let release = group_by.agg([expr_meas]).sort("A", Default::default()).collect()?;
        println!("{:?}", release);

        let sens = meas.map(&2)?;
        println!("sens: {:?}", sens); // why ?
        assert!(sens > 25.);
        assert!(sens < 25.00001);

        Ok(())
    }
}

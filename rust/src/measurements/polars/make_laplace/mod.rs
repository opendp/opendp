use crate::{
    core::{Function, Measurement, MetricSpace},
    domains::{AtomDomain, Context, ExprDomain, VectorDomain},
    error::Fallible,
    measurements::{get_discretization_consts, make_base_laplace},
    measures::MaxDivergence,
    metrics::L1Distance,
    traits::samplers::SampleDiscreteLaplaceZ2k,
};
use polars::prelude::*;

/// Polars operator to make the Laplace noise measurement
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
pub fn make_laplace_expr<C: Context>(
    input_domain: ExprDomain<C>,
    input_metric: L1Distance<f64>,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<ExprDomain<C>, Expr, L1Distance<f64>, MaxDivergence<f64>>>
where
    (ExprDomain<C>, L1Distance<f64>): MetricSpace,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    if input_domain.active_series()?.field.dtype != DataType::Float64 {
        return fallible!(MakeMeasurement, "input must be f64");
    }

    // Create Laplace measurement (needs explicit type annotation)
    let lap_meas = make_base_laplace::<VectorDomain<AtomDomain<f64>>>(
        Default::default(),
        Default::default(),
        scale.clone(),
        k,
    )?;

    // TODO: delete when threading supported
    let k = get_discretization_consts::<f64>(k)?.0;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |(_frame, expr): &(C::Value, Expr)| -> Fallible<Expr> {
            Ok(expr.clone().map(
                move |s: Series| {
                    let noisy_vec: Vec<f64> = s
                        .unpack::<Float64Type>()?
                        .into_no_null_iter()
                        .map(|value| f64::sample_discrete_laplace_Z2k(scale.clone(), value, k))
                        .collect::<Fallible<Vec<_>>>()
                        .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

                    // TODO: use this when threading supported
                    // let noisy_vec = lap_meas.invoke(&vec).unwrap();
                    Ok(Some(Series::new(&s.name(), noisy_vec)))
                },
                GetOutput::same_type(),
            ))
        }),
        input_metric,
        MaxDivergence::default(),
        lap_meas.privacy_map.clone(),
    )
}

#[cfg(test)]
mod test_make_laplace_expr {
    use super::*;
    use crate::{
        domains::{
            AtomDomain, LazyFrameContext, LazyFrameDomain, LazyGroupByContext, SeriesDomain,
            VectorDomain,
        },
        measurements::make_base_laplace,
    };

    fn get_test_data() -> Fallible<(ExprDomain<LazyFrameContext>, LazyFrame)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::new_closed((1, 4))?),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 5.5))?),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]]?.lazy())?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyFrameContext::Select,
            active_column: Some("B".to_string()),
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],)?
        .lazy();

        Ok((expr_domain, lazy_frame))
    }

    // How to get it as in other branches
    fn get_grouped_test_data() -> Fallible<(ExprDomain<LazyGroupByContext>, LazyGroupBy)> {
        let (expr_domain, lazy_frame) = get_test_data()?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: expr_domain.lazy_frame_domain,
            context: LazyGroupByContext {
                columns: vec!["A".to_string()],
            },
            active_column: expr_domain.active_column,
        };

        Ok((expr_domain, lazy_frame.groupby([col("A")])))
    }

    #[test]
    fn test_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;
        let scale: f64 = 1.0;

        // Get the measurement
        let meas = make_laplace_expr(expr_domain, L1Distance::default(), scale, None)?;
        // Get actual measurement result
        let meas_res = meas.invoke(&(lazy_frame.clone(), col("B")))?;
        let serie_actual = lazy_frame
            .select([meas_res])
            .collect()?
            .column("B")?
            .clone();

        // Get expected measurement result
        let chain = make_base_laplace(VectorDomain::default(), L1Distance::default(), scale, None)?;
        let result = chain.invoke(&vec![1.0, 1.0, 2.0])?;
        let result_serie = Series::new("B", result);

        assert_ne!(serie_actual, result_serie);
        Ok(())
    }

    #[test]
    fn test_make_sum_expr_grouped() -> Fallible<()> {
        let (expr_domain, lazy_groupby) = get_grouped_test_data()?;
        let scale: f64 = 1.0;

        // Get the measurement
        let meas = make_laplace_expr(expr_domain, L1Distance::default(), scale, None)?;
        // Get actual measurement result
        let meas_res = meas.invoke(&(lazy_groupby.clone(), col("B")))?;
        let serie_actual = lazy_groupby.agg([meas_res]).collect()?.column("B")?.clone();

        // Get expected measurement result
        let chain = make_base_laplace(VectorDomain::default(), L1Distance::default(), scale, None)?;
        let result = chain.invoke(&vec![1.0, 1.0, 2.0])?;
        let result_serie = Series::new("B", result);

        assert_ne!(serie_actual, result_serie);
        Ok(())
    }
}

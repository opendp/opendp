use crate::{
    core::{Function, Measurement, MetricSpace, PrivacyMap},
    domains::{Context, ExprDomain},
    error::Fallible,
    measurements::get_discretization_consts,
    measures::MaxDivergence,
    metrics::L1Distance,
    traits::{samplers::SampleDiscreteLaplaceZ2k, InfAdd, InfDiv},
};
use num::{Float, Zero};
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
    // Create Laplace measurement
    // let _scalar_laplace_measurement = make_base_laplace(
    //     VectorDomain::new(AtomDomain::<f64>::default()),
    //     L1Distance::<f64>::default(),
    //     scale.clone(),
    //     k,
    // )?;

    // TODO: delete when threading supported
    let (k, relaxation) = get_discretization_consts::<f64>(k)?;

    // let laplace_measurement_privacy_map = &scalar_laplace_measurement.privacy_map;

    Measurement::new(
        input_domain.clone(),
        Function::new_fallible(move |(_frame, expr): &(C::Value, Expr)| -> Fallible<Expr> {
            let active_column = input_domain
                .active_column
                .clone()
                .ok_or_else(|| err!(MakeTransformation, "No active column"))?;

            let expr = expr.clone().map(
                move |s: Series| {
                    if s.name() == active_column {
                        let noisy_vec: Vec<f64> = s
                            .unpack::<Float64Type>()?
                            .into_iter()
                            .filter_map(|opt_value| opt_value)
                            .map(|value| f64::sample_discrete_laplace_Z2k(scale.clone(), value, k))
                            .collect::<Fallible<Vec<_>>>()
                            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

                        // TODO: use this when threading supported
                        // let noisy_vec = scalar_laplace_measurement.invoke(&vec).unwrap();
                        let noisy_serie = Series::new(&active_column, noisy_vec);
                        Ok(Some(noisy_serie))
                    } else {
                        Ok(Some(s))
                    }
                },
                GetOutput::from_type(DataType::Float64),
            );

            Ok(expr)
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &f64| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }

            if d_in.is_zero() {
                return Ok(f64::zero());
            }

            if scale.is_zero() {
                return Ok(f64::infinity());
            }

            // increase d_in by the worst-case rounding of the discretization
            let d_in = d_in.inf_add(&relaxation)?;

            // d_in / scale
            d_in.inf_div(&scale)
        }), // laplace_measurement_privacy_map.clone(),
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

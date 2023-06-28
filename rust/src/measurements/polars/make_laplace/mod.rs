use crate::{
    core::{Function, Measurement},
    domains::{ExprDomain, VectorDomain},
    error::Fallible,
    measurements::make_base_laplace,
    measures::MaxDivergence,
    metrics::{L1Distance, LpDistance},
};
use polars::prelude::*;
use polars::{lazy::dsl::Expr, prelude::LazyFrame};


/// Polars operator to make the Laplace noise measurement
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
pub fn make_laplace_expr(
    input_domain: ExprDomain,
    input_metric: L1Distance<f64>,
    scale: f64,
) -> Fallible<Measurement<ExprDomain, Expr, L1Distance<f64>, MaxDivergence<f64>>> {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    // Create Laplace measurement
    let k = None; // TODO: ask do I need to set it
    let scalar_laplace_measurement = make_base_laplace(
        VectorDomain::default(),
        LpDistance::default(),
        scale.clone(),
        k.clone(),
    )?;
    let output_type = GetOutput::from_type(DataType::Float64);
    let laplace_measurement_privacy_map = &scalar_laplace_measurement.privacy_map;
    Measurement::new(
        input_domain.clone(),
        Function::new_fallible(move |(lf, expr): &(LazyFrame, Expr)| -> Fallible<Expr> {
            let active_column = input_domain
                .active_column
                .clone()
                .ok_or_else(|| err!(MakeTransformation, "No active column"))?;

            let expr = expr.clone().map(
                move |s: Series| {
                    if s.name() == active_column {
                        let vec: Vec<f64> = s
                            .unpack::<Float64Type>()?
                            .into_iter()
                            .filter_map(|opt_value| opt_value)
                            .collect();
                        let noisy_vec = scalar_laplace_measurement.invoke(&vec).unwrap();
                        let noisy_serie = Series::new(&active_column, noisy_vec);
                        Ok(Some(noisy_serie))
                    } else {
                        Ok(Some(s))
                    }
                },
                output_type,
            );

            Ok(expr)
        }),
        input_metric,
        MaxDivergence::default(),
        laplace_measurement_privacy_map.clone(),
    )
}

#[cfg(test)]
mod test_make_laplace_expr {
    use crate::domains::{AtomDomain, Context, LazyFrameDomain, SeriesDomain};
    use super::*;

    // How to get it as in other branches
    fn get_test_data() -> Fallible<(ExprDomain, LazyFrame)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::new_closed((1, 4))?),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 5.5))?),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]]?.lazy())?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: Context::Agg {
                columns: vec!["A".to_string()],
            },
            active_column: Some("B".to_string()),
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],)?
        .lazy();

        Ok((expr_domain, lazy_frame))
    }

    fn test_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;
        let scale: f64 = 1.0;

        // Get the measurement 
        let meas = make_laplace_expr(
            expr_domain, LpDistance<1, f64>, scale
        );
        Ok(())
    }
}

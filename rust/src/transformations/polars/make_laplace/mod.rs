use crate::{
    core::{Function, Measurement},
    domains::{ExprDomain, VectorDomain},
    error::Fallible,
    measurements::{make_base_laplace},
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
    use super::*;
    fn test_make_sum_expr() -> Fallible<()> {
        
        Ok(())
    }
}

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, ExprDomain},
    error::Fallible,
    measurements::{get_discretization_consts, make_base_laplace},
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{InfAdd, InfDiv},
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
        AtomDomain::default(),
        AbsoluteDistance::default(),
        scale.clone(),
        k.clone(),
    )?;
    let (_k, relaxation): (i32, f64) = get_discretization_consts(k.clone())?;
    //let laplace_measurement_privacy_map = scalar_laplace_measurement.privacy_map;
    Measurement::new(
        input_domain.clone(),
        Function::new_fallible(move |(lf, expr): &(LazyFrame, Expr)| -> Fallible<Expr> {
            let active_column = input_domain
                .active_column
                .clone()
                .ok_or_else(|| err!(MakeTransformation, "No active column"))?;
            
            let output_type = GetOutput::from_type(DataType::Float64);
        
            // let expr = expr.clone().map(
            //     |value: Series| 
            //         Ok(Some(value
            //             .unpack::<Float64Type>()?
            //             .into_iter()
            //             .map(|v| v.and_then(|v| scalar_laplace_measurement.invoke(&v).ok()))
            //             .collect::<Float64Chunked>()
            //             .into_series())),
            //         output_type,
            // );

            let expr = expr.clone().map(
                move |value: Series|
                    Ok(Some(value
                        .unpack::<Float64Type>()?
                        .into_iter()
                        .map(|v| v.and_then(|v| scalar_laplace_measurement.invoke(&v).ok()))
                        .collect::<Float64Chunked>()
                        .into_series())),
                    output_type,
            );

            Ok(expr)
        }),
        input_metric,
        MaxDivergence::default(),
        //laplace_measurement_privacy_map,
        // bug because expected struct `PrivacyMap<LpDistance<1, f64>, _>` found struct `PrivacyMap<AbsoluteDistance<f64>, _>`
        // but make_base_laplace only accepts AbsoluteDistance::default(),
        PrivacyMap::new_fallible(move |d_in: &f64| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale == 0.0 {
                return Ok(f64::INFINITY);
            }
            // increase d_in by the worst-case rounding of the discretization
            let d_in = d_in.inf_add(&relaxation)?;

            // d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}

#[cfg(test)]
mod test_make_laplace_expr {
    use super::*;
    fn test_make_sum_expr() -> Fallible<()> {
        // let expr: Vec<Option<f64>> = lf
        //     .clone()
        //     .collect()?
        //     .column(&active_column)?
        //     .f64()?
        //     .into_iter()
        //     //.apply(|value| scalar_laplace_measurement.invoke(&value))
        //     .map(|v| v.map(|v| scalar_laplace_measurement.invoke(&v).unwrap()))
        //     .collect();

        // let active_column = input_domain
        //     .active_column
        //     .clone()
        //     .ok_or_else(|| err!(MakeTransformation, "No active column"))?;

        // let res: Vec<Option<f64>> = lf
        //     .clone()
        //     .collect()?
        //     .column(&active_column)?
        //     .f64()?
        //     .into_iter()
        //     .map(|v| v.map(|v| scalar_laplace_measurement.invoke(&v).unwrap()))
        //     .collect();

        // let output_type = GetOutput::from_type(DataType::Float64);

        // let expr = expr.clone().map(
        //     |value: Series| -> Result<Series> {
        //         let mapped = value
        //             .unpack::<Float64Type>()?
        //             .into_iter()
        //             .map(|v| v.and_then(|v| scalar_laplace_measurement.invoke(&v).ok()))
        //             .collect::<Float64Chunked>()
        //             .into_series();
        //         Ok(mapped)
        //     },
        //     GetOutput::default(),
        // );

        Ok(())
    }
}

use polars::{
    lazy::dsl::{col, Expr, GetOutput},
    prelude::{DataType, Float64Type, LazyFrame},
};

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, ExprDomain},
    error::Fallible,
    measurements::{get_discretization_consts, make_base_laplace},
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{InfAdd, InfDiv},
};

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

            // let expr = col(&active_column);
            // // Retreive series of active_column
            // let active_serie = lf.clone().collect()?.column(&active_column)?.clone();

            // // Add noise to series
            // let mut active_serie_with_noise = Series::from_iter(
            //     active_serie
            //         .unpack::<Float64Type>()?
            //         .into_iter()
            //         .map(|v| v.map(|v| scalar_laplace_measurement.invoke(&v).unwrap())),
            // );
            // active_serie_with_noise.rename(active_serie.name());

            // let exprs: Vec<Expr> = trans_outputs.iter().map(|(_, expr)| expr.clone()).collect();
            let output_type = GetOutput::from_type(DataType::Float64);

            // Create an expression to add noise to the series
            let noisy_expr = expr.map(
                |s| {
                    Ok(s.unpack::<Float64Type>()?
                        .into_iter()
                        .map(|v| v.map(|v| scalar_laplace_measurement.invoke(&v).unwrap())))
                },
                output_type,
            );

            // Rename the resulting noisy series expression
            let noisy_expr = noisy_expr.alias(&active_column);

            Ok(noisy_expr.clone())
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

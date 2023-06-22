use polars::{
    lazy::dsl::Expr,
    prelude::{Float64Type, IntoLazy, LazyFrame},
    series::Series,
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

pub fn make_laplace_expr(
    input_domain: ExprDomain,
    input_metric: L1Distance<f64>,
    scale: f64,
) -> Fallible<Measurement<ExprDomain, Expr, L1Distance<f64>, MaxDivergence<f64>>> {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    // Create Laplace measurement
    let k = Some(4); // TODO The noise granularity in terms of 2^k
    let scalar_laplace_measurement = make_base_laplace(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        scale.clone(),
        k.clone(),
    )?;
    let (_k, relaxation): (i32, f64) = get_discretization_consts(k.clone())?;
    Measurement::new(
        input_domain.clone(),
        Function::new_fallible(move |(expr, lf): &(Expr, LazyFrame)| -> Fallible<Expr> {
            // Get last column name and position
            let last_column_id = input_domain.lazy_frame_domain.series_domains.len() - 1;
            let last_column_name = lf.clone().collect()?;
            let last_column_name = last_column_name.get_column_names()[last_column_id].clone();

            // Retreive series of last column
            let s = lf.clone().collect()?.column(last_column_name)?.clone();

            // Add noise to series
            let mut s_with_noise = Series::from_iter(
                s.unpack::<Float64Type>()?
                    .into_iter()
                    .map(|v| v.map(|v| scalar_laplace_measurement.invoke(&v).unwrap())),
            );
            s_with_noise.rename(s.name());

            Ok(expr.clone()) // TODO: placeholder
        }),
        input_metric,
        MaxDivergence::default(),
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

use polars::prelude::*;
use rand::{prelude::*, distributions::Uniform};
use std::ops::Mul;

use crate::{
    domains::{ExprDomain, Context}, 
    metrics::L1Distance, 
    core::{Measurement, Function, PrivacyMap}, 
    error::Fallible,
    measures::MaxDivergence,
};

/// Polars operator to make the Laplace noise measurement
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
pub fn make_continous_quantile_expr<C: Context>(
    input_domain: ExprDomain<C>,
    input_metric: L1Distance<f64>,
    scale: f64,
) -> Fallible<Measurement<ExprDomain<C>, Expr, L1Distance<f64>, MaxDivergence<f64>>>{

    let epsilon = scale; // placeholder TODO: get epsilon based on scale
    Measurement::new(
        input_domain,
        Function::new_fallible(move |(_frame, expr): &(C::Value, Expr)| -> Fallible<Expr> {
            // Incoming expr = make_score_elts_expr(expr.clone(), alpha) = |rank - alpha*N|
            
            // exp(-epsilon*|rank - alpha*N|)
            let exp_expr = expr.clone().mul(lit(-epsilon)).exp(); 

            // Z_{i+1} - Z_{i}
            let sorted_expr = expr.clone().sort(false);
            let shifted_expr = (sorted_expr.clone().shift(1) - sorted_expr)
                .slice(lit(1), lit(NULL));

            // (Z_{i+1} - Z_{i}) * exp(-eps*|rank - alpha*N|)
            let full_expr = exp_expr.mul(shifted_expr);

            // TODO here ? gumpel max expr to sample an i
            Ok(full_expr)
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &f64| {
            Ok(d_in.clone()) //placeholder TODO privacy map 
        })
    )
}

fn uniform_draw(lower_bound: f64, upper_bound: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let sample: f64 = rng.sample(Uniform::new(lower_bound, upper_bound));
    sample
}
use polars::prelude::*;
use rand::{prelude::*, distributions::Uniform};
use std::ops::Mul;

use crate::{
    domains::{ExprDomain, Context}, 
    metrics::LInfDiffDistance, 
    core::{Measurement, Function, PrivacyMap, MetricSpace}, 
    error::Fallible,
    measures::MaxDivergence, transformations::make_score_elts_expr, traits::{DistanceConstant, Float, Number},
};

use crate::traits::samplers::CastInternalRational;

/// Polars operator to compute quantiles.
/// Measurement for gumbel_max_expr and continuous_quantile_expr
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
/// * `alpha` - a value in [0, 1]. Choose 0.5 for median
/// * `temperature` - Higher temperatures are more private.
/// 
/// # Generics
/// * `C` - Context of the LazyFrame
pub fn make_continous_quantile_expr<C: Context, QO, TIA>(
    input_domain: ExprDomain<C>,
    input_metric: LInfDiffDistance<TIA>,
    scale: f64,
    alpha: f64,
    temperature: QO
) -> Fallible<Measurement<ExprDomain<C>, Expr, LInfDiffDistance<TIA>, MaxDivergence<QO>>>
where
    (ExprDomain<C>, LInfDiffDistance<TIA>): MetricSpace,
    TIA: Number + CastInternalRational,
    QO: CastInternalRational + DistanceConstant<TIA> + Float,
{

    let epsilon = scale; // placeholder TODO: get epsilon based on scale

    if temperature.is_sign_negative() || temperature.is_zero() {
        return fallible!(FailedFunction, "temperature must be positive");
    }

    let temp_frac = temperature.clone().into_rational()?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |(_frame, expr): &(C::Value, Expr)| -> Fallible<Expr> {
            // exp(-epsilon*|rank - alpha*N|)
            let exp_expr = make_score_elts_expr(expr.clone(), alpha)
                .clone()
                .mul(lit(-epsilon))
                .exp(); 

            // Z_{i+1} - Z_{i}
            let sorted_expr = expr.clone().sort(false);
            let shifted_expr = (sorted_expr.clone().shift(1) - sorted_expr)
                .slice(lit(1), lit(NULL));

            // (Z_{i+1} - Z_{i}) * exp(-eps*|rank - alpha*N|)
            let full_expr = exp_expr.mul(shifted_expr);

            // TODO here: gumpel max expr to sample an i


            // TODO here: get associated bounds of frame
            // frame sort and then get i and i+1 
            //let i = frame.select(expr.clone().sort(false));
            // let index = if C::GROUPBY {
            //     2.0
            // } else {
            //     1.0
            // };

            // TODO here: uniform draw in-between

            Ok(full_expr)
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            let d_in = QO::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if temperature.is_zero() {
                return Ok(QO::infinity());
            }
            // d_out >= d_in / temperature
            d_in.inf_div(&temperature)
        })
    )
}

fn uniform_draw(lower_bound: f64, upper_bound: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let sample: f64 = rng.sample(Uniform::new(lower_bound, upper_bound));
    sample
}
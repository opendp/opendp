use polars::prelude::*;
use std::ops::Mul;

use crate::{
    domains::{ExprDomain, Context}, 
    metrics::LInfDiffDistance, 
    core::{Measurement, Function, MetricSpace}, 
    error::Fallible,
    measures::MaxDivergence, transformations::make_score_elts_expr, 
    traits::{DistanceConstant, Float, Number, InfCast, InfDiv},
    measurements::{make_base_discrete_exponential, Optimize},
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
    QO: CastInternalRational + DistanceConstant<TIA> + Float + InfCast<f64>,
{
    let sensitivity: f64 = alpha.max(1.0 - alpha);
    let epsilon = sensitivity.inf_div(&scale)?;

    // let bounds = input_domain
    //     .active_series()?
    //     .atom_domain::<f64>()?
    //     .get_closed_bounds()?;
    // let (upper, lower) = bounds;

    let discrete_exponential = make_base_discrete_exponential::<TIA, QO>(
        Default::default(),
        Default::default(),
        temperature.clone(),
        Optimize::Max
    )?;

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

            // Exponential mechanism then max index sampling and uniform draw
            let full_expr = full_expr.clone().map(
                move |s: Series| {
                    let vec: Vec<f64> = s
                        .unpack::<Float64Type>()?
                        .into_no_null_iter()
                        .collect::<Vec<_>>();
                    // let d_e_f = discrete_exponential.function.clone();
                    // let index = d_e_f(vec);

                    // let sample = lower + f64::sample_standard_uniform(false)*(upper - lower);
                
                    // Ok(Some(Series::new(&s.name(), sample)))
                    Ok(Some(Series::new(&s.name(), vec)))
                },
                GetOutput::same_type(),
            );
            
            Ok(full_expr)
        }),
        input_metric,
        MaxDivergence::default(),
        discrete_exponential.privacy_map.clone()
    )
}

use polars::prelude::*;
use std::ops::Mul;

use crate::{
    core::{Function, Measurement, MetricSpace, PrivacyMap},
    domains::{Context, ExprDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::LInfDiffDistance,
    traits::{DistanceConstant, Float, InfDiv, Number}, //samplers::SampleUniform
    transformations::make_score_elts_expr,
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
/// * `TIA` - Atomic Input Type
pub fn make_continous_quantile_expr<C: Context, QO, TIA>(
    input_domain: ExprDomain<C>,
    input_metric: LInfDiffDistance<TIA>,
    scale: f64,
    alpha: f64,
    temperature: QO,
) -> Fallible<Measurement<ExprDomain<C>, Expr, LInfDiffDistance<TIA>, MaxDivergence<QO>>>
where
    (ExprDomain<C>, LInfDiffDistance<TIA>): MetricSpace,
    TIA: Number + CastInternalRational,
    QO: CastInternalRational + DistanceConstant<TIA> + Float,
{
    let sensitivity: f64 = alpha.max(1.0 - alpha);
    let epsilon = sensitivity.inf_div(&scale)?;

    // let discrete_exponential = make_base_discrete_exponential::<TIA, QO>(
    //     Default::default(),
    //     Default::default(),
    //     temperature.clone(),
    //     Optimize::Max
    // )?;

    Measurement::new(
        input_domain,
        Function::new_fallible(
            move |(_frame, expr): &(C::Value, Expr)| -> Fallible<Expr> {
            // exp(-epsilon*|rank - alpha*N|)
            let exp_expr = make_score_elts_expr(expr.clone(), alpha)
                .clone()
                .mul(lit(-epsilon))
                .exp();

            // Z_{i+1} - Z_{i}
            let sorted_expr = expr.clone().sort(false);
            let shifted_expr =
                (sorted_expr.clone().shift(1) - sorted_expr).slice(lit(1), lit(NULL));

            // (Z_{i+1} - Z_{i}) * exp(-eps*|rank - alpha*N|)
            let full_expr = exp_expr.mul(shifted_expr);

            // TODO here: gumpel max expr to sample an i
            // TODO here: get associated bounds of frame
            // TODO here: uniform draw in-between
            // call Function of Measurement of make_base_discrete_exponential as part of expression
            full_expr.clone().map(
                move |s: Series| {
                    let vec: Vec<f64> = s
                        .unpack::<Float64Type>()?
                        .into_no_null_iter()
                        .collect::<Vec<_>>();
                    // let r  = discrete_exponential.function.clone()(&vec);
                        // .map(|value| discrete_exponential.function.clone()());
                        //.enumerate()
                        // .map(|(i, value)| {
                        //     let mut shift = value.into_rational()? / &temp_frac;
                        //     if optimize == Optimize::Min {
                        //         shift.neg_assign();
                        //     }
                        //     Ok((i, GumbelPSRN::new(shift)))
                        // })
                        // .reduce(|l, r| {
                        //     let (mut l, mut r) = (l?, r?);
                        //     Ok(if l.1.greater_than(&mut r.1)? { l } else { r })
                        // })
                        // .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
                        // .map(|v| v.0);
                    Ok(Some(Series::new(&s.name(), vec)))
                },
                GetOutput::same_type(),
            );
            Ok(full_expr)
        }),
        input_metric,
        MaxDivergence::default(),
        // discrete_exponential.privacy_map.clone()
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
        }),
    )
}

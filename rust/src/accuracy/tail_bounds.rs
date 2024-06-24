use std::ops::Neg;

use crate::{
    measurements::expr_noise::Distribution,
    traits::{InfAdd, InfDiv, InfExp},
};

use super::Fallible;

pub(crate) fn integrate_discrete_noise_tail(
    distribution: Distribution,
    scale: f64,
    tail_bound: f64,
) -> Fallible<f64> {
    match distribution {
        Distribution::Laplace => integrate_discrete_laplace_tail(scale, tail_bound),
        Distribution::Gaussian => fallible!(
            MakeMeasurement,
            "gaussian tail bounds are not currently implemented"
        ),
    }
}

fn integrate_discrete_laplace_tail(scale: f64, tail_bound: f64) -> Fallible<f64> {
    let numer = tail_bound.neg_inf_div(&-scale)?.inf_exp()?;
    let denom = scale.neg().recip().neg_inf_exp()?.neg_inf_add(&1.)?;

    numer.inf_div(&denom)
}

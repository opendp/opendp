use dashu::{
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
    rbig,
};
use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{Approximate, MaxDivergence},
    metrics::AbsoluteDistance,
    traits::samplers::{CanonicalRV, PartialSample},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(features("contrib"))]
/// Make a Measurement that adds noise from a canonical noise distribution.
/// The implementation is tailored towards approximate-DP,
/// resulting in noise sampled from the Tulap distribution.
///
/// # Citations
/// - [AV23 Canonical Noise Distributions and Private Hypothesis Tests](https://projecteuclid.org/journals/annals-of-statistics/volume-51/issue-2/Canonical-noise-distributions-and-private-hypothesis-tests/10.1214/23-AOS2259.short)
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric of the input.
/// * `d_in` - Sensitivity
/// * `d_out` - Privacy parameters (ε, δ)
pub fn make_canonical_noise(
    input_domain: AtomDomain<f64>,
    input_metric: AbsoluteDistance<f64>,
    d_in: f64,
    d_out: (f64, f64),
) -> Fallible<Measurement<AtomDomain<f64>, AbsoluteDistance<f64>, Approximate<MaxDivergence>, f64>>
{
    if input_domain.nan() {
        return fallible!(MakeMeasurement, "input_domain must consist of non-nan data");
    }
    if d_in.is_sign_negative() || !d_in.is_finite() {
        return fallible!(
            MakeMeasurement,
            "d_in ({d_in}) must be a finite non-negative number"
        );
    }

    let (tradeoff, fixed_point) = approximate_to_tradeoff(d_out)?;
    let r_d_in = RBig::try_from(d_in)?;

    Measurement::new(
        input_domain,
        input_metric,
        Approximate(MaxDivergence),
        Function::new_fallible(move |&arg: &f64| {
            let canonical_rv = CanonicalRV {
                shift: RBig::try_from(arg.clamp(f64::MIN, f64::MAX)).unwrap_or(RBig::ZERO),
                scale: &r_d_in,
                tradeoff: &tradeoff,
                fixed_point: &fixed_point,
            };
            PartialSample::new(canonical_rv).value()
        }),
        PrivacyMap::new_fallible(move |d_in_p: &f64| {
            if !(0.0..=d_in).contains(d_in_p) {
                return fallible!(
                    FailedMap,
                    "d_in from the map ({d_in_p}) must be in [0, {d_in}]"
                );
            }
            if d_in.is_zero() {
                return Ok((0.0, 0.0));
            }
            Ok(d_out.clone())
        }),
    )
}

/// # Proof Definition
/// Given epsilon and delta, return the corresponding f-DP tradeoff curve
/// with conservative arithmetic,
/// as well as the fixed point `c` where `c = f(c)`.
/// Returns an error if epsilon or delta are invalid.
pub(crate) fn approximate_to_tradeoff(
    (epsilon, delta): (f64, f64),
) -> Fallible<(impl Fn(RBig) -> RBig + 'static + Clone + Send + Sync, RBig)> {
    if epsilon.is_sign_negative() || epsilon.is_zero() {
        return fallible!(
            MakeMeasurement,
            "epsilon ({epsilon}) must not be positive (greater than zero)"
        );
    }
    if !(0.0..=1.0).contains(&delta) {
        return fallible!(MakeMeasurement, "delta ({delta}) must be within [0, 1]");
    }

    let epsilon = FBig::<Down>::try_from(epsilon)?;
    let delta = RBig::try_from(delta)?;

    // exp(ε)
    let exp_eps = epsilon.clone().with_rounding::<Down>().exp();
    let exp_eps = RBig::try_from(exp_eps)?;

    // exp(-ε)
    let exp_neg_eps = (-epsilon).with_rounding::<Up>().exp();
    let exp_neg_eps = RBig::try_from(exp_neg_eps)?;

    //              = (1 - δ) / (1 + exp(ε))
    let fixed_point = (rbig!(1) - &delta) / (rbig!(1) + &exp_eps);

    // greater than 1/2 means the tradeoff curve is greater than 1 - x, which is invalid
    // exactly 1 / 2 means perfect privacy, and results in an infinite loop when sampling "infinite" noise
    if fixed_point >= rbig!(1 / 2) {
        return fallible!(
            MakeMeasurement,
            "fixed-point of the f-DP tradeoff curve must be less than 1/2. This indicates that your privacy parameters are too small."
        );
    }

    let tradeoff = move |alpha: RBig| {
        let t1 = rbig!(1) - &delta - &exp_eps * &alpha;
        let t2 = &exp_neg_eps * (rbig!(1) - &delta - alpha);
        t1.max(t2).max(rbig!(0))
    };
    Ok((tradeoff, fixed_point))
}

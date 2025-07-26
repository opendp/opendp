use dashu::rational::RBig;
use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{Approximate, MaxDivergence, f_dp::approximate_to_tradeoff},
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
) -> Fallible<Measurement<AtomDomain<f64>, f64, AbsoluteDistance<f64>, Approximate<MaxDivergence>>>
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
        Function::new_fallible(move |&arg: &f64| {
            let canonical_rv = CanonicalRV {
                shift: RBig::try_from(arg).unwrap_or(RBig::ZERO),
                scale: &r_d_in,
                tradeoff: &tradeoff,
                fixed_point: &fixed_point,
            };
            PartialSample::new(canonical_rv).value()
        }),
        input_metric,
        Approximate(MaxDivergence),
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

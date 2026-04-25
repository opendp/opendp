use dashu::{rational::RBig, rbig};
use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{PrivacyCurve, PrivacyCurveDP},
    metrics::AbsoluteDistance,
    traits::{
        InfCast,
        samplers::{CanonicalRV, PartialSample},
    },
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(d_out(c_type = "AnyObject *", rust_type = b"null", hint = "PrivacyCurve"))
)]
/// Make a Measurement that adds noise from a canonical noise distribution using f-DP.
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
/// * `d_out` - Privacy curve used through its tradeoff view
pub fn make_canonical_noise(
    input_domain: AtomDomain<f64>,
    input_metric: AbsoluteDistance<f64>,
    d_in: f64,
    d_out: PrivacyCurve,
) -> Fallible<Measurement<AtomDomain<f64>, AbsoluteDistance<f64>, PrivacyCurveDP, f64>> {
    if input_domain.nan() {
        return fallible!(MakeMeasurement, "input_domain must consist of non-nan data");
    }
    if d_in.is_sign_negative() || !d_in.is_finite() {
        return fallible!(
            MakeMeasurement,
            "d_in ({d_in}) must be a finite non-negative number"
        );
    }

    let tradeoff = enclose!(d_out, move |alpha: RBig| Ok(RBig::try_from(
        d_out.beta(f64::inf_cast(alpha)?)?
    )?));

    let fixed_point = find_fixed_point(&tradeoff)?;

    // A fixed point at or above 1/2 corresponds to perfect privacy or an invalid
    // tradeoff curve, and causes the canonical quantile recursion to diverge.
    if fixed_point >= rbig!(1 / 2) {
        return fallible!(
            MakeMeasurement,
            "fixed-point of the f-DP tradeoff curve must be less than 1/2. This indicates that your privacy parameters are too small."
        );
    }

    let r_d_in = RBig::try_from(d_in)?;

    Measurement::new(
        input_domain,
        input_metric,
        PrivacyCurveDP,
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
            if *d_in_p != d_in {
                return fallible!(
                    FailedMap,
                    "d_in from the map ({d_in_p}) must equal the measurement sensitivity ({d_in})"
                );
            }
            if d_in_p.is_zero() {
                return Ok(PrivacyCurve::new_tradeoff(|_alpha| Ok(1.0)));
            }
            Ok(d_out.clone())
        }),
    )
}

fn find_fixed_point(tradeoff: &impl Fn(RBig) -> Fallible<RBig>) -> Fallible<RBig> {
    let mut lo = 0.0;
    let mut hi = 0.5;
    let mut mid = -1.0;

    loop {
        let new_mid = lo + (hi - lo) / 2.0;
        if new_mid == mid {
            return Ok(RBig::try_from(lo)?);
        }
        mid = new_mid;

        let mid_r = RBig::try_from(mid)?;
        if tradeoff(mid_r.clone())? >= mid_r {
            lo = mid;
        } else {
            hi = mid;
        }
    }
}

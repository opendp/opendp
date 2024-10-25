use dashu::{float::FBig, rational::RBig};
use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{Approximate, MaxDivergence},
    metrics::AbsoluteDistance,
    traits::samplers::{PartialSample, TulapRV},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Make a Measurement that adds noise from the Tulap distribution to the input.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric of the input.
/// * `epsilon` - Privacy parameter ε.
/// * `delta` - Privacy parameter δ.
#[bootstrap(features("contrib"))]
pub fn make_tulap(
    input_domain: AtomDomain<f64>,
    input_metric: AbsoluteDistance<f64>,
    epsilon: f64,
    delta: f64,
) -> Fallible<Measurement<AtomDomain<f64>, f64, AbsoluteDistance<f64>, Approximate<MaxDivergence>>>
{
    if input_domain.nan() {
        return fallible!(MakeMeasurement, "input_domain members must be non-null");
    }
    if epsilon.is_sign_negative() || delta.is_sign_negative() {
        return fallible!(MakeMeasurement, "epsilon and delta must not be negative");
    }
    if delta > 1. {
        return fallible!(MakeMeasurement, "delta must not exceed 1");
    }
    let f_epsilon = FBig::try_from(epsilon)?;
    let r_delta = RBig::try_from(delta)?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |&arg: &f64| {
            let shift = RBig::try_from(arg).unwrap_or(RBig::ZERO);
            let tulap = TulapRV::new(shift, f_epsilon.clone(), r_delta.clone())?;
            PartialSample::new(tulap).value()
        }),
        input_metric,
        Approximate::default(),
        PrivacyMap::new_fallible(move |&d_in: &f64| {
            if d_in.is_sign_negative() {
                return fallible!(FailedMap, "sensitivity must be non-negative");
            }
            if d_in.is_zero() {
                return Ok((0., 0.));
            }
            // TODO: future work, generalize this?
            if d_in > 1.0 {
                return fallible!(FailedMap, "sensitivity must be at most one");
            }
            Ok((epsilon, delta))
        }),
    )
}

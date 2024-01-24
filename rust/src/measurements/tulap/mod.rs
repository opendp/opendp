use dashu::float::FBig;
use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::FixedSmoothedMaxDivergence,
    metrics::AbsoluteDistance,
    traits::samplers::{pinpoint, TulapPSRN},
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
) -> Fallible<
    Measurement<AtomDomain<f64>, f64, AbsoluteDistance<f64>, FixedSmoothedMaxDivergence<f64>>,
> {
    if input_domain.nullable() {
        return fallible!(
            MakeMeasurement,
            "input_domain must consist of non-null data"
        );
    }
    if epsilon.is_sign_negative() || delta.is_sign_negative() {
        return fallible!(FailedMap, "epsilon and delta must not be negative");
    }
    if delta > 1. {
        return fallible!(FailedMap, "delta must not exceed 1");
    }
    let f_epsilon = FBig::try_from(epsilon)?;
    let f_delta = FBig::try_from(delta)?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |&arg: &f64| {
            let shift = FBig::try_from(arg).unwrap_or(FBig::ZERO);
            let mut tulap = TulapPSRN::new(shift, f_epsilon.clone(), f_delta.clone());
            pinpoint::<TulapPSRN, f64>(&mut tulap)
        }),
        input_metric,
        FixedSmoothedMaxDivergence::default(),
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

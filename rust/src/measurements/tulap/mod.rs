use opendp_derive::bootstrap;
use rug::{float::Round, Float, Rational};

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::FixedSmoothedMaxDivergence,
    metrics::AbsoluteDistance,
    traits::{samplers::TulapPSRN, InfMul},
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
    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &f64| {
            let shift = Rational::try_from(*arg)
                .map_err(|_| err!(FailedFunction, "Expected non-null input"))?;
            let mut tulap = TulapPSRN::new(
                shift,
                Float::with_val(52, epsilon),
                Float::with_val(52, delta),
            );
            loop {
                let lower = tulap.value(Round::Down)?.to_f64();
                let upper = tulap.value(Round::Up)?.to_f64();
                if lower == upper {
                    return Ok(lower);
                }
                tulap.refine()?;
            }
        }),
        input_metric,
        FixedSmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &f64| {
            Ok((epsilon.inf_mul(d_in)?, delta.inf_mul(d_in)?))
        }),
    )
}

use crate::{
    core::{Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    traits::{samplers::sample_geometric_exp_fast, CastInternalRational, InfLn, InfMul, InfSub},
};
use dashu::integer::UBig;
use opendp_derive::bootstrap;
use std::{fmt::Debug, ops::Neg};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(measurement(rust_type = "AnyMeasurement"),),
    generics(DI(suppress), MI(suppress), TO(suppress))
)]
/// Select a private candidate whose score is above a threshold.
///
/// Given `measurement` that satisfies ε-DP, returns new measurement M' that satisfies 2ε-DP.
/// M' releases the first invocation of `measurement` whose score is above `threshold`.
///
/// Each time a score is below `threshold`
/// the algorithm may terminate with probability `stop_probability` and return nothing.
///
/// `measurement` should make releases in the form of (score, candidate).
/// If you are writing a custom scorer measurement in Python,
/// specify the output type as `TO=(float, "ExtrinsicObject")`.
/// This ensures that the float value is accessible to the algorithm.
/// The candidate, left as arbitrary Python data, is held behind the ExtrinsicObject.
///
/// Algorithm 1 in `Private selection from private candidates <https://arxiv.org/pdf/1811.07971.pdf#page=7>`_ (Liu and Talwar, STOC 2019).
///
/// # Arguments
/// * `measurement` - A measurement that releases a 2-tuple of (score, candidate)
/// * `stop_probability` - The probability of stopping early at any iteration.
/// * `threshold` - The threshold score. Return immediately if the score is above this threshold.
///
/// # Returns
/// A measurement that returns a release from `measurement` whose score is greater than `threshold`, or none.
pub fn make_select_private_candidate<
    DI: 'static + Domain,
    MI: 'static + Metric,
    TO: 'static + Debug,
>(
    measurement: Measurement<DI, (f64, TO), MI, MaxDivergence>,
    stop_probability: f64,
    threshold: f64,
) -> Fallible<Measurement<DI, Option<(f64, TO)>, MI, MaxDivergence>>
where
    (DI, MI): MetricSpace,
{
    // If stop_probability is 1, the measurement is executed only once with double the privacy budget,
    // so we prevent this inefficient case.
    if !(0f64..1f64).contains(&stop_probability) {
        return fallible!(MakeMeasurement, "stop_probability must be in [0, 1)");
    }

    if !threshold.is_finite() {
        return fallible!(MakeMeasurement, "threshold must be finite");
    }

    let scale = if stop_probability > 0.0 {
        let ln_cp = 1.0.neg_inf_sub(&stop_probability)?.inf_ln()?;
        Some(ln_cp.recip().neg().into_rational()?)
    } else {
        None
    };

    let function = measurement.function.clone();
    let privacy_map = measurement.privacy_map.clone();

    Measurement::new(
        measurement.input_domain.clone(),
        Function::new_fallible(move |arg| {
            let mut remaining_iterations = (scale.clone())
                .map(|s| sample_geometric_exp_fast(s).map(|v| v + UBig::ONE))
                .transpose()?;

            loop {
                let (score, output) = function.eval(arg)?;

                if score >= threshold {
                    return Ok(Some((score, output)));
                }

                if let Some(i) = remaining_iterations.as_mut() {
                    *i -= UBig::ONE;
                    if i == &UBig::ZERO {
                        return Ok(None);
                    }
                }
            }
        }),
        measurement.input_metric.clone(),
        measurement.output_measure.clone(),
        PrivacyMap::new_fallible(move |d_in| privacy_map.eval(d_in)?.inf_mul(&2.0)),
    )
}

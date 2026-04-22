use opendp_derive::bootstrap;
use std::os::raw::c_char;

use crate::{
    core::{FfiResult, Function, Measure, Measurement, PrivacyMap},
    error::Fallible,
    ffi::{
        any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
        util::{self, Type, to_str},
    },
    measures::{MaxDivergence, RenyiDivergence},
    traits::RoundCast,
};

use crate::combinators::select_private_candidate::Repetitions;

fn to_f64<T: 'static>(obj: AnyObject) -> Fallible<f64>
where
    f64: RoundCast<T>,
{
    f64::round_cast(obj.downcast::<T>()?)
}

#[bootstrap(
    features("contrib", "private-selection-v2"),
    arguments(
        threshold(c_type = "AnyObject *", default = b"null"),
        distribution(c_type = "char *", default = "geometric"),
        eta(c_type = "AnyObject *", default = b"null")
    )
)]
/// Select a private candidate from repeated private candidates.
///
/// `measurement` should make releases in the form of `(score, candidate)`.
/// If you are writing a custom scorer measurement in Python,
/// specify the output type as `TO=(float, "ExtrinsicObject")`.
/// This ensures that the float value is accessible to the algorithm.
/// The candidate, left as arbitrary Python data, is held behind the ExtrinsicObject.
///
/// Supported parameter combinations:
///
/// * `measurement` under `MaxDivergence` and `threshold=Some(_)`:
///   Liu and Talwar (2019), with `distribution=Repetitions::Geometric` only,
///   and privacy cost `2 * measurement.map(d_in)`.
/// * `measurement` under `MaxDivergence` and `threshold=None`:
///   Papernot and Steinke (2021) Corollary 3,
///   with `distribution` in the negative-binomial family, and privacy cost
///   `(2 + eta) * measurement.map(d_in)`.
/// * `measurement` under `RenyiDivergence` and `threshold=Some(_)`:
///   Appendix C Corollary 16 of Papernot and Steinke (2021), with
///   `distribution=Repetitions::Geometric` only.
/// * `measurement` under `RenyiDivergence` and `threshold=None`:
///   Papernot and Steinke (2021), with `distribution=Repetitions::Poisson` (Theorem 6) or any
///   negative-binomial family member (Theorem 2).
///
/// Unsupported combinations raise a construction-time error.
///
/// # Arguments
/// * `measurement` - A measurement that releases a 2-tuple of (score, candidate)
/// * `mean` - The requested mean number of repetitions
/// * `threshold` - If set, release the first candidate whose score is at least this threshold. Otherwise, return the best candidate among the sampled repetitions.
/// * `distribution` - One of `"poisson"`, `"geometric"`, `"logarithmic"`, or `"negative_binomial"`
/// * `eta` - The shape parameter for `distribution="negative_binomial"`, omit otherwise
fn make_select_private_candidate(
    measurement: &AnyMeasurement,
    mean: f64,
    threshold: Option<f64>,
    distribution: String,
    eta: Option<f64>,
) -> Fallible<AnyMeasurement> {
    fn monomorphize<MO: 'static + Measure>(
        measurement: &AnyMeasurement,
        mean: f64,
        threshold: Option<f64>,
        distribution: Repetitions,
    ) -> Fallible<AnyMeasurement>
    where
        MO: super::PrivateCandidateMeasure,
    {
        let function = measurement.function.clone();
        let privacy_map = measurement.privacy_map.clone();
        let measurement = Measurement::new(
            measurement.input_domain.clone(),
            measurement.input_metric.clone(),
            measurement.output_measure.downcast_ref::<MO>()?.clone(),
            Function::new_fallible(move |arg: &AnyObject| {
                let release = function.eval(arg)?;

                Ok(if release.type_ == Type::of::<(f64, AnyObject)>() {
                    release.downcast::<(f64, AnyObject)>()?
                } else if let Ok(val) = release.downcast::<Vec<AnyObject>>() {
                    if let Ok([score, value]) = <[AnyObject; 2]>::try_from(val) {
                        let score = dispatch!(to_f64, [(score.type_, @numbers)], (score));
                        (score.unwrap_or(f64::NAN), value)
                    } else {
                        (f64::NAN, AnyObject::new(()))
                    }
                } else {
                    (f64::NAN, AnyObject::new(()))
                })
            }),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in)?.downcast()),
        )?;

        let m = super::make_select_private_candidate(measurement, mean, threshold, distribution)?;

        let privacy_map = m.privacy_map.clone();
        let function = m.function.clone();

        Measurement::new(
            m.input_domain.clone(),
            m.input_metric.clone(),
            AnyMeasure::new(m.output_measure.clone()),
            Function::new_fallible(move |arg: &AnyObject| function.eval(arg).map(AnyObject::new)),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in).map(AnyObject::new)
            }),
        )
    }

    let distribution = match distribution.as_str() {
        "poisson" => {
            if eta.is_some() {
                return fallible!(FFI, "eta must be omitted when distribution is \"poisson\"");
            }
            Repetitions::Poisson
        }
        "geometric" => {
            if eta.is_some() {
                return fallible!(
                    FFI,
                    "eta must be omitted when distribution is \"geometric\""
                );
            }
            Repetitions::Geometric
        }
        "logarithmic" => {
            if eta.is_some() {
                return fallible!(
                    FFI,
                    "eta must be omitted when distribution is \"logarithmic\""
                );
            }
            Repetitions::Logarithmic
        }
        "negative_binomial" => Repetitions::NegativeBinomial {
            eta: eta.ok_or_else(|| {
                err!(
                    FFI,
                    "eta must be provided when distribution is \"negative_binomial\""
                )
            })?,
        },
        _ => return fallible!(FFI, "unrecognized distribution value: {}", distribution),
    };

    let MO_ = measurement.output_measure.type_.clone();
    dispatch!(
        monomorphize,
        [(MO_, [MaxDivergence, RenyiDivergence])],
        (measurement, mean, threshold, distribution)
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_select_private_candidate(
    measurement: *const AnyMeasurement,
    mean: f64,
    threshold: *const AnyObject,
    distribution: *const c_char,
    eta: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let threshold = if let Some(param) = util::as_ref(threshold) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };
    let distribution = try_!(to_str(distribution)).to_string();
    let eta = if let Some(param) = util::as_ref(eta) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };

    FfiResult::from(make_select_private_candidate(
        try_as_ref!(measurement),
        mean,
        threshold,
        distribution,
        eta,
    ))
}

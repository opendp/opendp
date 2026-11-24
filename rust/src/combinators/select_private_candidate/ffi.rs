use opendp_derive::bootstrap;
use std::os::raw::c_char;

use crate::{
    core::{FfiResult, Function, Measure, Measurement, PrivacyMap},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util::{self, Type, to_str},
    },
    measures::{MaxDivergence, RenyiDivergence},
    traits::RoundCast,
};

use super::{PrivateCandidateMeasure, Repetitions};

fn to_f64<T: 'static>(obj: AnyObject) -> Fallible<f64>
where
    f64: RoundCast<T>,
{
    f64::round_cast(obj.downcast::<T>()?)
}

// Adapt the FFI's dynamically typed scored candidate to the strongly typed
// (score, candidate) representation expected by both selection algorithms.
fn into_scored_measurement<MO: 'static + Measure>(
    measurement: &AnyMeasurement,
) -> Fallible<Measurement<AnyDomain, AnyMetric, MO, (f64, AnyObject)>> {
    let function = measurement.function.clone();
    let privacy_map = measurement.privacy_map.clone();

    Measurement::new(
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
    )
}

fn into_any_measurement<MO: 'static + Measure>(
    measurement: Measurement<AnyDomain, AnyMetric, MO, Option<(f64, AnyObject)>>,
) -> Fallible<AnyMeasurement> {
    let privacy_map = measurement.privacy_map.clone();
    let function = measurement.function.clone();

    Measurement::new(
        measurement.input_domain.clone(),
        measurement.input_metric.clone(),
        AnyMeasure::new(measurement.output_measure.clone()),
        Function::new_fallible(move |arg: &AnyObject| function.eval(arg).map(AnyObject::new)),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in).map(AnyObject::new)
        }),
    )
}

fn monomorphize<MO: 'static + Measure + PrivateCandidateMeasure>(
    measurement: &AnyMeasurement,
    mean: f64,
    threshold: Option<f64>,
    repetitions: Repetitions,
) -> Fallible<AnyMeasurement> {
    let scored = into_scored_measurement::<MO>(measurement)?;
    let selected = super::make_select_private_candidate_impl(scored, mean, threshold, repetitions)?;
    into_any_measurement(selected)
}

#[bootstrap(features("contrib"))]
/// Select the first private candidate whose score is at least `min_score`.
///
/// The repetition budget is geometric, with expected size at most `mean`.
/// `measurement` releases a pair `(score, candidate)`; for a Python user
/// measurement specify `TO=(float, "ExtrinsicObject")`.
///
/// The privacy cost is twice the base pure-DP cost, or the appropriate
/// conditional-selection bound for a Rényi-DP base measurement.
fn make_select_at_least(
    measurement: &AnyMeasurement,
    mean: f64,
    min_score: f64,
) -> Fallible<AnyMeasurement> {
    let MO_ = measurement.output_measure.type_.clone();
    dispatch!(
        monomorphize,
        [(MO_, [MaxDivergence, RenyiDivergence])],
        (measurement, mean, Some(min_score), Repetitions::Geometric)
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_select_at_least(
    measurement: *const AnyMeasurement,
    mean: f64,
    min_score: f64,
) -> FfiResult<*mut AnyMeasurement> {
    FfiResult::from(make_select_at_least(
        try_as_ref!(measurement),
        mean,
        min_score,
    ))
}

#[bootstrap(
    features("contrib"),
    arguments(
        distribution(c_type = "char *", default = "geometric"),
        eta(c_type = "AnyObject *", default = b"null")
    )
)]
/// Return the highest-scoring private candidate among a random number of draws.
///
/// `measurement` releases a pair `(score, candidate)`; for a Python user
/// measurement specify `TO=(float, "ExtrinsicObject")`.
///
/// `mean` bounds the expected number of repetitions. `distribution` is one
/// of `"geometric"`, `"logarithmic"`, `"negative_binomial"`, or
/// `"poisson"`; `eta` is required only for `"negative_binomial"`.
/// Poisson repetitions require a Rényi-DP base measurement.
fn make_select_best_candidate(
    measurement: &AnyMeasurement,
    mean: f64,
    distribution: String,
    eta: Option<f64>,
) -> Fallible<AnyMeasurement> {
    let repetitions = match distribution.as_str() {
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
            eta: eta.ok_or_else(|| err!(FFI, "eta must be provided for negative_binomial"))?,
        },
        _ => return fallible!(FFI, "unrecognized distribution value: {}", distribution),
    };

    let MO_ = measurement.output_measure.type_.clone();
    dispatch!(
        monomorphize,
        [(MO_, [MaxDivergence, RenyiDivergence])],
        (measurement, mean, None, repetitions)
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_select_best_candidate(
    measurement: *const AnyMeasurement,
    mean: f64,
    distribution: *const c_char,
    eta: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let distribution = try_!(to_str(distribution)).to_string();
    let eta = if let Some(param) = util::as_ref(eta) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };

    FfiResult::from(make_select_best_candidate(
        try_as_ref!(measurement),
        mean,
        distribution,
        eta,
    ))
}

fn make_select_private_candidate(
    measurement: &AnyMeasurement,
    stop_probability: f64,
    threshold: f64,
) -> Fallible<AnyMeasurement> {
    let scored = into_scored_measurement::<MaxDivergence>(measurement)?;
    #[allow(deprecated)]
    let selected = super::make_select_private_candidate(scored, stop_probability, threshold)?;
    into_any_measurement(selected)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_select_private_candidate(
    measurement: *const AnyMeasurement,
    stop_probability: f64,
    threshold: f64,
) -> FfiResult<*mut AnyMeasurement> {
    FfiResult::from(make_select_private_candidate(
        try_as_ref!(measurement),
        stop_probability,
        threshold,
    ))
}

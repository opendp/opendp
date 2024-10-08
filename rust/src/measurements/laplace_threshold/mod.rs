#[cfg(feature = "ffi")]
mod ffi;

/// Algorithms that depend on the propose-test-release framework.
use std::collections::HashMap;

use dashu::float::FBig;
use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::{Error, ErrorVariant, Fallible};
use crate::measures::{Approximate, MaxDivergence};
use crate::metrics::L1Distance;
use crate::traits::samplers::{LaplaceRV, PartialSample};
use crate::traits::{Float, Hashable, InfCast, InfDiv, InfExp, InfLn, InfSub, RoundCast};
use dashu::base::ConversionError;
use num::Zero;

#[cfg(all(test, feature = "partials"))]
mod tests;

#[bootstrap(
    features("contrib"),
    arguments(threshold(c_type = "void *")),
    generics(TK(suppress), TV(suppress)),
    derived_types(TV = "$get_distance_type(input_metric)")
)]
/// Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric for the input domain.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `threshold` - Exclude counts that are less than this minimum value.
///
/// # Generics
/// * `TK` - Type of Key. Must be hashable/categorical.
/// * `TV` - Type of Value. Must be float.
pub fn make_laplace_threshold<TK, TV>(
    input_domain: MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
    input_metric: L1Distance<TV>,
    scale: f64,
    threshold: TV,
) -> Fallible<
    Measurement<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        HashMap<TK, TV>,
        L1Distance<TV>,
        Approximate<MaxDivergence>,
    >,
>
where
    TK: Hashable,
    TV: Float + RoundCast<FBig>,

    FBig: TryFrom<TV, Error = ConversionError> + TryFrom<f64>,
    f64: InfCast<TV>,
{
    if input_domain.value_domain.nullable() {
        return fallible!(MakeMeasurement, "values must be non-null");
    }

    if threshold < TV::zero() {
        return fallible!(MakeMeasurement, "threshold must be non-negative");
    }

    if scale.is_sign_negative() {
        return fallible!(FailedFunction, "scale must be non-negative");
    }

    let _2 = TV::exact_int_cast(2)?;

    let f_scale = FBig::try_from(scale)?;
    let f_threshold = FBig::try_from(threshold)?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |data: &HashMap<TK, TV>| {
            data.clone()
                .into_iter()
                .filter_map(|(k, value)| Some((k, FBig::try_from(value).ok()?)))
                // noise output count
                .map(|(key, f_value)| {
                    let mut sample = PartialSample::new(LaplaceRV::new(f_value, f_scale.clone())?);
                    let is_above = sample.is_above(&f_threshold)?;
                    Ok((key, sample, is_above))
                })
                // only keep keys with values above threshold
                .filter(|res| res.as_ref().map(|(_, _, above)| *above).unwrap_or(true))
                .map(|res: Fallible<(TK, PartialSample<LaplaceRV>, bool)>| {
                    let (k, mut v, _) = res?;
                    Ok((k, v.value()?))
                })
                // fail the whole computation if any cast or noise addition failed
                .collect()
        }),
        input_metric,
        Approximate::default(),
        PrivacyMap::new_fallible(move |d_in: &TV| {
            let d_in = f64::inf_cast(*d_in)?;
            if d_in.is_sign_negative() {
                return fallible!(FailedMap, "d_in must be not be negative");
            }

            if d_in.is_zero() {
                return Ok((0.0, 0.0));
            }

            if scale.is_zero() {
                return Ok((f64::INFINITY, 1.0));
            }

            let epsilon = d_in.inf_div(&scale)?;
            let threshold = f64::neg_inf_cast(threshold)?;

            // delta is the probability that noise will push the count beyond the threshold
            // δ = d_in / exp(distance_to_instability) / 2

            // however, computing exp is unstable, so it is computed last
            // δ = exp(ln(d_in / 2) - distance_to_instability)

            // compute the distance to instability, conservatively rounding down
            //                         = (threshold -            d_in)             / scale
            let distance_to_instability = threshold.neg_inf_sub(&d_in)?.neg_inf_div(&scale)?;

            if distance_to_instability <= 1.0 {
                return Ok((epsilon, 1.0));
            }

            // ln(delta) = ln(d_in / 2) - distance_to_instability
            let ln_delta = d_in
                .inf_div(&2.0)?
                .inf_ln()?
                .inf_sub(&distance_to_instability)?;

            // delta =        exp(ln(delta))
            let delta = match ln_delta.inf_exp() {
                // catch error on overflowing delta as just infinity
                Err(Error {
                    variant: ErrorVariant::Overflow,
                    ..
                }) => f64::INFINITY,
                result => result?,
            };

            // delta is only sensibly at most 1
            Ok((epsilon, delta.min(1.0)))
        }),
    )
}

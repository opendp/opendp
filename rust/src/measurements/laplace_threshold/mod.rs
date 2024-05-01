#[cfg(feature = "ffi")]
mod ffi;

/// Algorithms that depend on the propose-test-release framework.
use std::collections::HashMap;

use dashu::base::ConversionError;
use dashu::float::FBig;
use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::{Error, ErrorVariant, Fallible};
use crate::measures::FixedSmoothedMaxDivergence;
use crate::metrics::L1Distance;
use crate::traits::samplers::{check_above, pinpoint, LaplacePSRN};
use crate::traits::{ExactIntCast, Float, Hashable, RoundCast};

#[cfg(all(test, feature = "partials"))]
mod tests;

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *"), threshold(c_type = "void *")),
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
    scale: TV,
    threshold: TV,
) -> Fallible<
    Measurement<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        HashMap<TK, TV>,
        L1Distance<TV>,
        FixedSmoothedMaxDivergence<TV>,
    >,
>
where
    TK: Hashable,
    TV: Float + RoundCast<FBig>,
    u32: ExactIntCast<TV::Bits>,
    FBig: TryFrom<TV, Error = ConversionError>,
    (MapDomain<AtomDomain<TK>, AtomDomain<TV>>, L1Distance<TV>): MetricSpace,
{
    if input_domain.value_domain.nullable() {
        return fallible!(MakeMeasurement, "values must be non-null");
    }

    if threshold < TV::zero() {
        return fallible!(MakeMeasurement, "threshold must be non-negative");
    }

    if scale < TV::zero() {
        return fallible!(MakeMeasurement, "scale must be non-negative");
    }

    let _2 = TV::exact_int_cast(2)?;

    let f_scale = FBig::try_from(scale)?;
    let f_threshold = FBig::try_from(threshold)?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |data: &HashMap<TK, TV>| {
            data.clone()
                .into_iter()
                .filter(|(_, value)| !value.is_null())
                // noise output count
                .map(|(key, value)| {
                    // value should be non-null, but if it is, attempt to degrade gracefully
                    let f_value = FBig::try_from(value).expect("value is non-null due to filter");

                    let mut psrn = LaplacePSRN::new(f_value, f_scale.clone())?;
                    let is_above = check_above(&mut psrn, &f_threshold)?;
                    Ok((key, psrn, is_above))
                })
                // only keep keys with values above threshold
                .filter(|res| res.as_ref().map(|(_, _, above)| *above).unwrap_or(true))
                .map(|res: Fallible<(TK, LaplacePSRN, bool)>| {
                    let (k, mut v, _) = res?;
                    Ok((k, pinpoint::<LaplacePSRN, TV>(&mut v)?))
                })
                // fail the whole computation if any cast or noise addition failed
                .collect()
        }),
        input_metric,
        FixedSmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TV| {
            if d_in.is_sign_negative() {
                return fallible!(FailedMap, "d_in must be not be negative");
            }

            if d_in.is_zero() {
                return Ok((TV::zero(), TV::zero()));
            }

            if scale.is_zero() {
                return Ok((TV::infinity(), TV::one()));
            }

            let epsilon = d_in.inf_div(&scale)?;

            // delta is the probability that noise will push the count beyond the threshold
            // δ = d_in / exp(distance_to_instability) / 2

            // however, computing exp is unstable, so it is computed last
            // δ = exp(ln(d_in / 2) - distance_to_instability)

            // compute the distance to instability, conservatively rounding down
            //                         = (threshold -            d_in)             / scale
            let distance_to_instability = threshold.neg_inf_sub(&d_in)?.neg_inf_div(&scale)?;

            if distance_to_instability <= TV::zero() {
                return Ok((epsilon, TV::one()));
            }

            // ln(delta) = ln(d_in / 2) - distance_to_instability
            let ln_delta = d_in
                .inf_div(&_2)?
                .inf_ln()?
                .inf_sub(&distance_to_instability)?;

            // delta =        exp(ln(delta))
            let delta = match ln_delta.inf_exp() {
                // catch error on overflowing delta as just infinity
                Err(Error {
                    variant: ErrorVariant::Overflow,
                    ..
                }) => TV::infinity(),
                result => result?,
            };

            // delta is only sensibly at most 1
            Ok((epsilon, delta.min(TV::one())))
        }),
    )
}

#[cfg(feature = "ffi")]
mod ffi;

/// Algorithms that depend on the propose-test-release framework.
use std::collections::HashMap;

use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::{Error, ErrorVariant, Fallible};
use crate::measures::{Approximate, MaxDivergence};
use crate::metrics::L1Distance;
use crate::traits::samplers::sample_discrete_laplace_Z2k;
use crate::traits::{
    CastInternalRational, ExactIntCast, Float, Hashable, InfAdd, InfCast, InfDiv, InfExp, InfLn,
    InfSub,
};
use num::Zero;

use super::get_discretization_consts;

#[bootstrap(
    features("contrib", "floating-point"),
    arguments(
        threshold(c_type = "void *"),
        k(default = -1074, rust_type = "i32", c_type = "uint32_t")),
    generics(TK(suppress), TV(suppress)),
    derived_types(TV = "$get_distance_type(input_metric)")
)]
/// Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric for the input domain.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `threshold` - Exclude counts that are less than this minimum value.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `TK` - Type of Key. Must be hashable/categorical.
/// * `TV` - Type of Value. Must be float.
pub fn make_laplace_threshold<TK, TV>(
    input_domain: MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
    input_metric: L1Distance<TV>,
    scale: f64,
    threshold: TV,
    k: Option<i32>,
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
    TV: Float + CastInternalRational,
    i32: ExactIntCast<TV::Bits>,
    f64: InfCast<TV>,
    (MapDomain<AtomDomain<TK>, AtomDomain<TV>>, L1Distance<TV>): MetricSpace,
{
    if input_domain.value_domain.nullable() {
        return fallible!(FailedFunction, "values must be non-null");
    }

    if threshold < TV::zero() {
        return fallible!(FailedFunction, "threshold must be non-negative");
    }

    if scale.is_sign_negative() {
        return fallible!(FailedFunction, "scale must be non-negative");
    }

    let _2 = TV::exact_int_cast(2)?;
    let (k, relaxation) = get_discretization_consts(k)?;

    // actually reject noisy values below threshold + relaxation, to account for discretization
    let true_threshold = threshold.inf_add(&relaxation)?;

    let relaxation = f64::inf_cast(relaxation)?;
    let threshold = f64::neg_inf_cast(threshold)?;
    let f_scale = scale.into_rational()?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |data: &HashMap<TK, TV>| {
            data.clone()
                .into_iter()
                // noise output count
                .map(|(key, v)| {
                    sample_discrete_laplace_Z2k(v.into_rational()?, f_scale.clone(), k)
                        .map(|v| (key, TV::from_rational(v)))
                })
                // only keep keys with values gte threshold
                .filter(|res| {
                    res.as_ref()
                        .map(|(_k, v)| v >= &true_threshold)
                        .unwrap_or(true)
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

            let d_in = d_in.inf_add(&relaxation)?;
            let epsilon = d_in.inf_div(&scale)?;

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

#[cfg(all(test, feature = "partials"))]
mod test;

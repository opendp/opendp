#[cfg(feature = "ffi")]
mod ffi;

/// Algorithms that depend on the propose-test-release framework.
use std::collections::HashMap;

use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::{Error, ErrorVariant, Fallible};
use crate::measures::FixedSmoothedMaxDivergence;
use crate::metrics::L1Distance;
use crate::traits::samplers::{sample_discrete_laplace_Z2k, CastInternalRational};
use crate::traits::{ExactIntCast, Float, Hashable};

use super::get_discretization_consts;

#[bootstrap(
    features("contrib", "floating-point"),
    arguments(
        scale(c_type = "void *"),
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
    scale: TV,
    threshold: TV,
    k: Option<i32>,
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
    TV: Float + CastInternalRational,
    i32: ExactIntCast<TV::Bits>,
    (MapDomain<AtomDomain<TK>, AtomDomain<TV>>, L1Distance<TV>): MetricSpace,
{
    if input_domain.value_domain.nullable() {
        return fallible!(FailedFunction, "values must be non-null");
    }

    if threshold < TV::zero() {
        return fallible!(FailedFunction, "threshold must be non-negative");
    }

    if scale < TV::zero() {
        return fallible!(FailedFunction, "scale must be non-negative");
    }

    let _2 = TV::exact_int_cast(2)?;
    let (k, relaxation) = get_discretization_consts(k)?;

    // actually reject noisy values below threshold + relaxation, to account for discretization
    let true_threshold = threshold.inf_add(&relaxation)?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |data: &HashMap<TK, TV>| {
            data.clone()
                .into_iter()
                // noise output count
                .map(|(key, v)| sample_discrete_laplace_Z2k(v, scale, k).map(|v| (key, v)))
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

            let d_in = d_in.inf_add(&relaxation)?;
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

#[cfg(all(test, feature = "partials"))]
mod test;

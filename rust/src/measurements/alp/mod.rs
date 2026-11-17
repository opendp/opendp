use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dashu::rational::RBig;
use num::{ToPrimitive, Zero as _};
use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::measures::MaxDivergence;
use crate::metrics::{AbsoluteDistance, L01InfDistance};
use crate::traits::samplers::{fill_bytes, sample_bernoulli_rational};
use crate::traits::{Hashable, InfCast, InfMul, Integer};
use std::collections::hash_map::DefaultHasher;

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

const ALPHA_DEFAULT: u32 = 4;
const SIZE_FACTOR_DEFAULT: u32 = 50;

/// Implementation of a mechanism for representing sparse integer queries e.g. a sparse histogram.
/// The mechanism was introduced in the paper:
///
/// "Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access"
///
/// Available here: <https://arxiv.org/abs/2106.10068>

/// Input domain. The mechanism is designed for settings where the domain of K is huge.
type SparseDomain<K, C> = MapDomain<AtomDomain<K>, AtomDomain<C>>;

// Types used to store the DP projection.
type BitVector = Vec<bool>;
type HashFunction<K> = Arc<dyn Fn(&K) -> usize + Send + Sync>;

#[derive(Clone)]
#[doc(hidden)]
pub struct AlpState<K> {
    alpha: f64,
    scale: f64,
    hashers: Vec<HashFunction<K>>,
    z: BitVector,
}

/// Hash a key of type K into a u64
fn pre_hash<K: Hash>(x: K) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

/// A hash function with type: [2^64] -> [2^l]
///
/// Computes ((a*x + b) mod 2^64) div 2^(64-l)
/// * where a and b are sampled uniformly at random from [2^64]
/// * where a must be odd
/// * where l must be lt 64
///
/// The hash function is 2-approximate universal and uniform.
/// See <http://hjemmesider.diku.dk/~jyrki/Paper/CP-11.4.1997.pdf>
/// "A Reliable Randomized Algorithm for the Closest-Pair Problem"
fn hash(x: u64, a: u64, b: u64, l: u32) -> usize {
    if l == 0 {
        return 0;
    }
    (a.wrapping_mul(x).wrapping_add(b) >> (64 - l)) as usize
}

/// Samples a random hash function with type: K -> [2^l]
/// * where l must be lt 64
fn sample_hash_function<K: Hash>(l: u32) -> Fallible<Arc<dyn Fn(&K) -> usize + Send + Sync>> {
    let mut buf = [0u8; 8];
    fill_bytes(&mut buf)?;
    let a = u64::from_ne_bytes(buf) | 1u64;
    fill_bytes(&mut buf)?;
    let b = u64::from_ne_bytes(buf);
    Ok(Arc::new(move |x: &K| hash(pre_hash(x), a, b, l)))
}

/// Computes ceil(log_2(x))
fn exponent_next_power_of_two(x: u64) -> u32 {
    let exp = 63 - x.leading_zeros().min(63);
    if x > (1 << exp) { exp + 1 } else { exp }
}

// Interpret the public floating-point parameters as exact rationals.
fn make_rate(alpha: f64, scale: f64) -> Fallible<RBig> {
    if !scale.is_finite() || scale <= 0.0 {
        return fallible!(
            MakeMeasurement,
            "scale ({}) must be positive and finite",
            scale
        );
    }
    if !alpha.is_finite() || alpha <= 0.0 {
        return fallible!(
            MakeMeasurement,
            "alpha ({}) must be positive and finite",
            alpha
        );
    }
    Ok(RBig::try_from(scale)? / RBig::try_from(alpha)?)
}

// Exact randomized rounding, capped at the number of hash functions.
// Values above this cap result in the same projected bits.
fn scale_and_round<CI>(x: CI, rate: &RBig, max: usize) -> Fallible<usize>
where
    CI: Integer + ToPrimitive,
{
    if max == 0 || x <= CI::zero() {
        return Ok(0);
    }

    let x = x
        .to_u128()
        .ok_or_else(|| err!(FailedCast, "failed to convert nonnegative count to u128"))?;
    let r = RBig::from(x) * rate;
    if r >= RBig::from(max) {
        return Ok(max);
    }

    let floor = r.floor();
    let fractional = r - RBig::from(floor.clone());
    let floor = floor
        .to_usize()
        .ok_or_else(|| err!(FailedCast, "rounded count exceeds usize"))?;
    Ok(floor + usize::from(sample_bernoulli_rational(fractional)?))
}

/// Computes the DP projection.
///
/// Corresponds to Algorithm 4 in the paper
fn compute_projection<K, CI>(
    x: &HashMap<K, CI>,
    hashers: &Vec<HashFunction<K>>, // h
    rate: &RBig,
    flip_prob: &RBig,
    projection_size: usize, // s
) -> Fallible<BitVector>
where
    CI: Integer + ToPrimitive,
{
    let mut z = vec![false; projection_size];

    for (k, v) in x.iter() {
        let round = scale_and_round(v.clone(), rate, hashers.len())?;
        // ^= true TODO: Hash collisions can be handled using OR or XOR
        (hashers.iter().take(round)).for_each(|h_i| z[h_i(k) % projection_size] = true);
    }

    z.iter()
        .map(|b| sample_bernoulli_rational(flip_prob.clone()).map(|flip| b ^ flip))
        .collect()
}

/// Estimate the value of an entry based on its noisy bit representation.
///
/// Corresponds to Algorithm 3 in the paper
fn estimate_unary(v: &Vec<bool>) -> f64 {
    let mut prefix_sum = Vec::with_capacity(v.len() + 1usize);
    prefix_sum.push(0);

    v.iter()
        .map(|b| if *b { 1 } else { -1 })
        .for_each(|x| prefix_sum.push(prefix_sum.last().unwrap() + x));

    let high = prefix_sum.iter().max().unwrap();
    let peaks = prefix_sum
        .iter()
        .enumerate()
        .filter_map(|(idx, height)| if high == height { Some(idx) } else { None })
        .collect::<Vec<_>>();

    // Return the average position
    peaks.iter().sum::<usize>() as f64 / peaks.len() as f64
}

/// Estimates the value of an entry based on its noisy bit representation.
///
/// Corresponds to Algorithm 5 in the paper
fn compute_estimate<K>(state: &AlpState<K>, key: &K) -> f64 {
    let v = (state.hashers.iter())
        .map(|h_i| state.z[h_i(key) % state.z.len()])
        .collect::<Vec<_>>();

    estimate_unary(&v) * state.alpha as f64 / state.scale
}

/// Measurement to compute a DP projection of bounded sparse data.
///
/// This function allows the user to create custom hash functions. The mechanism provides no utility guarantees
/// if hash functions are chosen poorly. It is recommended to use [`make_alp_queryable`].
///
/// # Citations
/// * [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access](https://arxiv.org/abs/2106.10068)
///   Algorithm 4
///
/// # Arguments
/// * `scale` - Privacy loss parameter. Equal to epsilon/sensitivity.
/// * `alpha` - Parameter used for scaling and determining p in randomized response step. The default value is 4.
/// * `projection_size` - Should be sufficiently large to limit hash collisions.
/// * `hashers` - Hash functions used to project and estimate entries. The hash functions are not allowed to panic on any input.
/// The hash functions in `h` should have type K -> \[s\]. To limit collisions the functions should be universal and uniform.
/// The evaluation time of post-processing is O(h.len()).
pub fn make_alp_state_with_hashers<K, CI>(
    input_domain: SparseDomain<K, CI>,
    input_metric: L01InfDistance<AbsoluteDistance<CI>>,
    scale: f64,
    alpha: f64,
    projection_size: usize,
    hashers: Vec<HashFunction<K>>,
) -> Fallible<
    Measurement<
        SparseDomain<K, CI>,
        L01InfDistance<AbsoluteDistance<CI>>,
        MaxDivergence,
        AlpState<K>,
    >,
>
where
    K: 'static + Hashable,
    CI: 'static + Integer + ToPrimitive,
    f64: InfCast<CI>,
    (SparseDomain<K, CI>, L01InfDistance<AbsoluteDistance<CI>>): MetricSpace,
{
    if input_domain.value_domain.nan() {
        return fallible!(MakeMeasurement, "value domain must be non-nan");
    }

    let rate = make_rate(alpha, scale)?;
    if projection_size == 0 {
        return fallible!(MakeMeasurement, "projection_size must be positive");
    }
    let flip_prob = RBig::ONE / (RBig::try_from(alpha)? + RBig::from(2));

    Measurement::new(
        input_domain,
        input_metric,
        MaxDivergence,
        Function::new_fallible(move |x: &HashMap<K, CI>| {
            let z = compute_projection(x, &hashers, &rate, &flip_prob, projection_size)?;
            Ok(AlpState {
                alpha,
                scale,
                hashers: hashers.clone(),
                z,
            })
        }),
        PrivacyMap::new_fallible(move |(_l0, l1, _li)| f64::inf_cast(*l1)?.inf_mul(&scale)),
    )
}

/// Measurement to compute a DP projection of bounded sparse data.
///
/// See [`make_alp_queryable`] for details.
pub fn make_alp_state<K, CI>(
    input_domain: SparseDomain<K, CI>,
    input_metric: L01InfDistance<AbsoluteDistance<CI>>,
    scale: f64,
    total_limit: CI,
    value_limit: Option<CI>,
    size_factor: Option<u32>,
    alpha: Option<u32>,
) -> Fallible<
    Measurement<
        SparseDomain<K, CI>,
        L01InfDistance<AbsoluteDistance<CI>>,
        MaxDivergence,
        AlpState<K>,
    >,
>
where
    K: 'static + Hashable,
    CI: 'static + Integer + InfCast<f64> + ToPrimitive,
    f64: InfCast<CI> + InfCast<u32>,
    (SparseDomain<K, CI>, L01InfDistance<AbsoluteDistance<CI>>): MetricSpace,
{
    let value_limit = value_limit
        .or_else(|| {
            input_domain
                .value_domain
                .bounds
                .as_ref()
                .and_then(|b| b.upper())
                .cloned()
        })
        .ok_or_else(|| {
            err!(
                MakeMeasurement,
                "value_limit is required when data is unbounded"
            )
        })?;

    if value_limit < CI::zero() || total_limit < CI::zero() {
        return fallible!(
            MakeMeasurement,
            "value_limit and total_limit must be nonnegative"
        );
    }

    let value_limit = value_limit
        .to_u128()
        .ok_or_else(|| err!(MakeMeasurement, "failed to convert value_limit"))?;
    let total_limit = total_limit
        .to_u128()
        .ok_or_else(|| err!(MakeMeasurement, "failed to convert total_limit"))?;

    let alpha = alpha.unwrap_or(ALPHA_DEFAULT) as f64;
    let rate = make_rate(alpha, scale)?;

    let m = (RBig::from(value_limit) * &rate)
        .ceil()
        .to_usize()
        .ok_or_else(|| err!(MakeMeasurement, "number of hash functions exceeds usize"))?;
    let size =
        (RBig::from(size_factor.unwrap_or(SIZE_FACTOR_DEFAULT)) * RBig::from(total_limit) * &rate)
            .ceil()
            .to_u64()
            .ok_or_else(|| err!(MakeMeasurement, "projection size is too large"))?;
    let exp = exponent_next_power_of_two(size);
    let projection_size = 1usize
        .checked_shl(exp)
        .ok_or_else(|| err!(MakeMeasurement, "projection size is too large"))?;
    let hashers = (0..m)
        .map(|_| sample_hash_function(exp))
        .collect::<Fallible<Vec<HashFunction<K>>>>()?;

    make_alp_state_with_hashers(
        input_domain,
        input_metric,
        scale,
        alpha,
        projection_size,
        hashers,
    )
}

/// Make a postprocessor that wraps the AlpState in a Queryable object.
///
/// The Queryable object works similar to a dictionary.
/// Note that the access time is O(state.h.len()).
pub fn post_alp_state_to_queryable<K>() -> Function<AlpState<K>, Queryable<K, f64>>
where
    K: 'static + Clone,
{
    Function::new(move |state: &AlpState<K>| {
        let state = state.clone();
        Queryable::new_raw_external(move |key: &K| Ok(compute_estimate(&state, key)))
    })
}

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *"),
        total_limit(c_type = "void *"),
        value_limit(c_type = "void *", default = b"null"),
        size_factor(default = 50),
        alpha(default = 4),
    ),
    generics(K(suppress), CI(suppress)),
    derived_types(CI = "$get_value_type(get_carrier_type(input_domain))")
)]
/// Measurement to release a queryable containing a DP projection of bounded sparse data.
///
/// The size of the projection is O(total * size_factor * scale / alpha).
/// The evaluation time of post-processing is O(beta * scale / alpha).
///
/// `size_factor` is an optional multiplier (defaults to 50) for setting the size of the projection.
/// There is a memory/utility trade-off.
/// The value should be sufficiently large to limit hash collisions.
///
/// # Citations
/// * [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access](https://arxiv.org/abs/2106.10068) Algorithm 4
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `scale` - Privacy loss parameter. This is equal to epsilon/sensitivity.
/// * `value_limit` - Upper bound on individual values (referred to as β). Entries above β are clamped.
/// * `total_limit` - Either the true value or an upper bound estimate of the sum of all values in the input.
/// * `size_factor` - Optional multiplier (default of 50) for setting the size of the projection.
/// * `alpha` - Optional parameter (default of 4) for scaling and determining p in randomized response step.
pub fn make_alp_queryable<K, CI>(
    input_domain: MapDomain<AtomDomain<K>, AtomDomain<CI>>,
    input_metric: L01InfDistance<AbsoluteDistance<CI>>,
    scale: f64,
    total_limit: CI,
    value_limit: Option<CI>,
    size_factor: Option<u32>,
    alpha: Option<u32>,
) -> Fallible<
    Measurement<
        MapDomain<AtomDomain<K>, AtomDomain<CI>>,
        L01InfDistance<AbsoluteDistance<CI>>,
        MaxDivergence,
        Queryable<K, f64>,
    >,
>
where
    K: 'static + Hashable,
    CI: 'static + Integer + InfCast<f64> + ToPrimitive,
    f64: InfCast<CI>,
    (
        MapDomain<AtomDomain<K>, AtomDomain<CI>>,
        L01InfDistance<AbsoluteDistance<CI>>,
    ): MetricSpace,
{
    // this constructor is a simple wrapper for make_alp_state that adds a postprocessing step
    make_alp_state(
        input_domain,
        input_metric,
        scale,
        total_limit,
        value_limit,
        size_factor,
        alpha,
    )? >> post_alp_state_to_queryable()
}

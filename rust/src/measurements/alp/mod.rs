use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use num::ToPrimitive;
use opendp_derive::bootstrap;
use rug::{float::Round, ops::AddAssignRound, ops::DivAssignRound, Float as RugFloat};

use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::measures::MaxDivergence;
use crate::metrics::L1Distance;
use crate::traits::samplers::{fill_bytes, SampleBernoulli};
use crate::traits::{Float, Hashable, InfCast, Integer};
use std::collections::hash_map::DefaultHasher;

#[cfg(feature = "ffi")]
mod ffi;

const ALPHA_DEFAULT: u32 = 4;
const SIZE_FACTOR_DEFAULT: u32 = 50;

/// Implementation of a mechanism for representing sparse integer queries e.g. a sparse histogram.
/// The mechanism was introduced in the paper:
///
/// "Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access"
///
/// Available here: arxiv.org/abs/2106.10068

/// Input domain. The mechanism is designed for settings where the domain of K is huge.
type SparseDomain<K, C> = MapDomain<AtomDomain<K>, AtomDomain<C>>;

// Types used to store the DP projection.
type BitVector = Vec<bool>;
type HashFunction<K> = Arc<dyn Fn(&K) -> usize + Send + Sync>;

#[derive(Clone)]
#[doc(hidden)]
pub struct AlpState<K, T> {
    alpha: T,
    scale: T,
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
/// See http://hjemmesider.diku.dk/~jyrki/Paper/CP-11.4.1997.pdf
/// "A Reliable Randomized Algorithm for the Closest-Pair Problem"
fn hash(x: u64, a: u64, b: u64, l: u32) -> usize {
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
    let exp = 63 - x.leading_zeros();
    if x > (1 << exp) {
        exp + 1
    } else {
        exp
    }
}

// Multiplies x with scale/alpha and applies randomized rounding to return an integer
fn scale_and_round<CI, CO>(x: CI, alpha: CO, scale: CO) -> Fallible<usize>
where
    CI: Integer + ToPrimitive,
    CO: InfCast<RugFloat>,
    RugFloat: InfCast<CO>,
{
    let mut scale = RugFloat::neg_inf_cast(scale)?;
    scale.div_assign_round(RugFloat::inf_cast(alpha)?, Round::Down);
    // Truncate bits that represents values below 2^-53
    scale.set_prec_round(
        (f64::MANTISSA_DIGITS as i32 - scale.get_exp().unwrap()).max(1) as u32,
        Round::Down,
    );

    let r = RugFloat::with_val(
        f64::MANTISSA_DIGITS * 2,
        x.max(CI::zero()).to_u64().unwrap_or_else(|| u64::MAX),
    ) * scale;
    let floored = f64::inf_cast(r.clone().floor())? as usize;

    match bool::sample_bernoulli(f64::inf_cast(r.fract())?, false)? {
        true => Ok(floored + 1),
        false => Ok(floored),
    }
}

// Probability of flipping bits = 1 / (alpha + 2)
fn compute_prob<T: InfCast<RugFloat>>(alpha: T) -> f64
where
    RugFloat: InfCast<T>,
{
    let mut alpha = RugFloat::neg_inf_cast(alpha).expect("impl is infallible");
    alpha.add_assign_round(2, Round::Down);
    alpha.recip_round(Round::Up);
    // Round up to preserve privacy
    f64::inf_cast(alpha).expect("impl is infallible")
}

/// Reject any choice of `scale` or `alpha` such that `scale / alpha < 2^-52`
///
/// Due to privacy concerns the current implementation discards bits with significance less than 2^-52 from scale/alpha
fn are_parameters_invalid<T: InfCast<RugFloat>>(alpha: T, scale: T) -> bool
where
    RugFloat: InfCast<T>,
{
    let scale = RugFloat::inf_cast(scale).expect("impl is infallible");
    let alpha = RugFloat::neg_inf_cast(alpha).expect("impl is infallible");
    scale * RugFloat::with_val(53, 52).exp2() < alpha
}

/// Computes the DP projection.
///
/// Corresponds to Algorithm 4 in the paper
fn compute_projection<K, CI, CO>(
    x: &HashMap<K, CI>,
    hashers: &Vec<HashFunction<K>>, // h
    alpha: CO,
    scale: CO,
    projection_size: usize, // s
) -> Fallible<BitVector>
where
    CI: Integer + ToPrimitive,
    CO: Clone + InfCast<RugFloat>,
    RugFloat: InfCast<CO>,
{
    let mut z = vec![false; projection_size];

    for (k, v) in x.iter() {
        let round = scale_and_round(v.clone(), alpha.clone(), scale.clone())?;
        // ^= true TODO: Hash collisions can be handled using OR or XOR
        (hashers.iter().take(round)).for_each(|h_i| z[h_i(k) % projection_size] = true);
    }

    let p = compute_prob(alpha);

    z.iter()
        .map(|b| bool::sample_bernoulli(p, false).map(|flip| b ^ flip))
        .collect()
}

/// Estimate the value of an entry based on its noisy bit representation.
///
/// Corresponds to Algorithm 3 in the paper
fn estimate_unary<T: num::Float>(v: &Vec<bool>) -> T {
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
    T::from(peaks.iter().sum::<usize>()).unwrap() / T::from(peaks.len()).unwrap()
}

/// Estimates the value of an entry based on its noisy bit representation.
///
/// Corresponds to Algorithm 5 in the paper
fn compute_estimate<K, C: num::Float>(state: &AlpState<K, C>, key: &K) -> C {
    let v = (state.hashers.iter())
        .map(|h_i| state.z[h_i(key) % state.z.len()])
        .collect::<Vec<_>>();

    estimate_unary::<C>(&v) * C::from(state.alpha).unwrap() / state.scale
}

/// Measurement to compute a DP projection of bounded sparse data.
///
/// This function allows the user to create custom hash functions. The mechanism provides no utility guarantees
/// if hash functions are chosen poorly. It is recommended to use make_base_alp.
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
pub fn make_alp_state_with_hashers<K, CI, CO>(
    input_domain: SparseDomain<K, CI>,
    input_metric: L1Distance<CI>,
    scale: CO,
    alpha: CO,
    projection_size: usize,
    hashers: Vec<HashFunction<K>>,
) -> Fallible<Measurement<SparseDomain<K, CI>, AlpState<K, CO>, L1Distance<CI>, MaxDivergence<CO>>>
where
    K: 'static + Hashable,
    CI: 'static + Integer + ToPrimitive,
    CO: 'static + Float + InfCast<RugFloat> + InfCast<CI>,
    RugFloat: InfCast<CO>,
    (SparseDomain<K, CI>, L1Distance<CI>): MetricSpace,
{
    if input_domain.value_domain.nullable() {
        return fallible!(MakeMeasurement, "value domain must be non-nullable");
    }

    if scale.is_sign_negative() || scale.is_zero() {
        return fallible!(MakeMeasurement, "scale must be positive");
    }
    if alpha.is_sign_negative() || alpha.is_zero() {
        return fallible!(MakeMeasurement, "alpha must be positive");
    }
    if projection_size == 0 {
        return fallible!(MakeMeasurement, "projection_size must be positive");
    }
    if are_parameters_invalid(alpha, scale) {
        return fallible!(
            MakeMeasurement,
            "scale divided by alpha must be above 2^-52"
        );
    }

    Measurement::new(
        input_domain,
        Function::new_fallible(move |x: &HashMap<K, CI>| {
            let z = compute_projection(x, &hashers, alpha, scale, projection_size)?;
            Ok(AlpState {
                alpha,
                scale,
                hashers: hashers.clone(),
                z,
            })
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_from_constant(scale),
    )
}

/// Measurement to compute a DP projection of bounded sparse data.
///
/// See [`make_alp_queryable`] for details.
pub fn make_alp_state<K, CI, CO>(
    input_domain: SparseDomain<K, CI>,
    input_metric: L1Distance<CI>,
    scale: CO,
    total_limit: CI,
    value_limit: Option<CI>,
    size_factor: Option<u32>,
    alpha: Option<u32>,
) -> Fallible<Measurement<SparseDomain<K, CI>, AlpState<K, CO>, L1Distance<CI>, MaxDivergence<CO>>>
where
    K: 'static + Hashable,
    CI: 'static + Integer + InfCast<CO> + ToPrimitive,
    CO: 'static + Float + InfCast<RugFloat> + InfCast<CI>,
    RugFloat: InfCast<CO>,
    (SparseDomain<K, CI>, L1Distance<CI>): MetricSpace,
{
    let value_limit: f64 = value_limit
        // if value limit is None, read it from the domain
        .or_else(|| {
            (input_domain.value_domain.bounds())
                .and_then(|b| b.upper())
                .cloned()
        })
        // if value limit is still None, return an error
        .ok_or_else(|| {
            err!(
                MakeTransformation,
                "value_limit is required when data is unbounded"
            )
        })?
        .to_f64()
        .ok_or_else(|| err!(MakeTransformation, "failed to parse value_limit"))?;

    let total_limit: f64 = total_limit
        .to_f64()
        .ok_or_else(|| err!(MakeTransformation, "failed to parse total_limit"))?;

    let size_factor = size_factor.unwrap_or(SIZE_FACTOR_DEFAULT) as f64;

    let alpha = CO::inf_cast(alpha.unwrap_or(ALPHA_DEFAULT))?;

    let quotient = (scale / alpha)
        .to_f64()
        .ok_or_else(|| err!(MakeTransformation, "failed to parse scale"))?;
    let m = usize::inf_cast(value_limit * quotient)?;

    let exp = exponent_next_power_of_two((size_factor * total_limit * quotient) as u64);
    let hashers = (0..m)
        .map(|_| sample_hash_function(exp))
        .collect::<Fallible<Vec<HashFunction<K>>>>()?;

    make_alp_state_with_hashers(input_domain, input_metric, scale, alpha, 1 << exp, hashers)
}

/// Make a postprocessor that wraps the AlpState in a Queryable object.
///
/// The Queryable object works similar to a dictionary.
/// Note that the access time is O(state.h.len()).
pub fn post_alp_state_to_queryable<K, C>() -> Function<AlpState<K, C>, Queryable<K, C>>
where
    K: 'static + Clone,
    C: 'static + Float,
{
    Function::new(move |state: &AlpState<K, C>| {
        let state = state.clone();
        Queryable::new_raw_external(move |key: &K| Ok(compute_estimate(&state, key)))
    })
}

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *"),
        scale(c_type = "void *"),
        total_limit(c_type = "void *"),
        value_limit(c_type = "void *", default = b"null"),
        size_factor(c_type = "void *", default = 50),
        alpha(c_type = "void *", default = 4),
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
/// * `scale` - Privacy loss parameter. This is equal to epsilon/sensitivity.
/// * `value_limit` - Upper bound on individual values (referred to as β). Entries above β are clamped.
/// * `total_limit` - Either the true value or an upper bound estimate of the sum of all values in the input.
/// * `size_factor` - Optional multiplier (default of 50) for setting the size of the projection.
/// * `alpha` - Optional parameter (default of 4) for scaling and determining p in randomized response step.
pub fn make_alp_queryable<K, CI, CO>(
    input_domain: MapDomain<AtomDomain<K>, AtomDomain<CI>>,
    input_metric: L1Distance<CI>,
    scale: CO,
    total_limit: CI,
    value_limit: Option<CI>,
    size_factor: Option<u32>,
    alpha: Option<u32>,
) -> Fallible<
    Measurement<
        MapDomain<AtomDomain<K>, AtomDomain<CI>>,
        Queryable<K, CO>,
        L1Distance<CI>,
        MaxDivergence<CO>,
    >,
>
where
    K: 'static + Hashable,
    CI: 'static + Integer + InfCast<CO> + ToPrimitive,
    CO: 'static + Float + InfCast<RugFloat> + InfCast<CI>,
    RugFloat: InfCast<CO>,
    (MapDomain<AtomDomain<K>, AtomDomain<CI>>, L1Distance<CI>): MetricSpace,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn idx<T>(i: usize) -> Arc<dyn Fn(&T) -> usize + Send + Sync> {
        Arc::new(move |_| i)
    }

    // Functions that always return its index
    fn index_identify_functions<T>(n: usize) -> Vec<HashFunction<T>> {
        (0..n).map(|i| idx(i)).collect::<Vec<HashFunction<T>>>()
    }

    #[test]
    fn test_exponent_next_power_of_two() -> Fallible<()> {
        assert_eq!(exponent_next_power_of_two(1 as u64), 0);

        assert_eq!(exponent_next_power_of_two(2 as u64), 1);

        assert_eq!(exponent_next_power_of_two(3 as u64), 2);

        assert_eq!(exponent_next_power_of_two(7 as u64), 3);

        Ok(())
    }

    #[test]
    fn test_hash() -> Fallible<()> {
        assert_eq!(hash(3, 4, 5, 64), 17);
        assert_eq!(hash(3, 4, 5, 63), 8);

        assert_eq!(hash(1, u64::MAX, 0, 2), 3);
        assert_eq!(hash(1, u64::MAX, 0, 3), 7);

        assert_eq!(hash(4, u64::MAX, 0, 16), (1 << 16) - 1);

        Ok(())
    }

    #[test]
    fn test_sample_hash() -> Fallible<()> {
        let h = sample_hash_function(5)?;

        for i in 0u64..20u64 {
            assert!(h(&i) < (1 << 5));
        }

        Ok(())
    }

    #[test]
    fn test_alp_construction() -> Fallible<()> {
        let beta = 10;
        let alp = make_alp_state_with_hashers::<u32, u32, f64>(
            MapDomain::default(),
            L1Distance::default(),
            1.0,
            1.,
            beta,
            index_identify_functions(beta),
        )?;

        assert_eq!(alp.map(&1)?, 1.);

        let mut x = HashMap::new();
        x.insert(42, 10);

        alp.function.eval(&x.clone())?;

        // Values exceeding beta is truncated internally
        x.insert(42, 10000);
        alp.function.eval(&x.clone())?;

        Ok(())
    }

    #[test]
    fn test_alp_construction_out_of_range() -> Fallible<()> {
        let s = 5;
        // Hash functions return values out of range
        // Handle silently using modulo
        // Returning an error would violate privacy
        let h = index_identify_functions(20);
        let alp = make_alp_state_with_hashers::<u32, u32, f64>(
            MapDomain::default(),
            L1Distance::default(),
            1.0,
            1.,
            s,
            h,
        )?;

        let mut x = HashMap::new();
        x.insert(42, 3);

        alp.function.eval(&x.clone())?;

        Ok(())
    }

    #[test]
    fn test_estimate_unary() -> Fallible<()> {
        let z1 = vec![true, true, true, false, true, false, false, true];
        assert!(estimate_unary::<f64>(&z1) == 4.0);

        let z2 = vec![true, false, false, false, true, false, false, true];
        assert!(estimate_unary::<f64>(&z2) == 1.0);

        let z3 = vec![false, true, true, false, false, true, false, true];
        assert!(estimate_unary::<f64>(&z3) == 3.0);

        Ok(())
    }

    #[test]
    fn test_compute_estimate() -> Fallible<()> {
        let z1 = vec![true, true, true, false, true, false, false, true];
        assert!(
            compute_estimate(
                &AlpState {
                    alpha: 3.,
                    scale: 1.0,
                    hashers: index_identify_functions(8),
                    z: z1
                },
                &0
            ) == 12.0
        );

        let z2 = vec![true, false, false, false, true, false, false, true];
        assert!(
            compute_estimate(
                &AlpState {
                    alpha: 1.,
                    scale: 2.0,
                    hashers: index_identify_functions(8),
                    z: z2
                },
                &0
            ) == 0.5
        );

        let z3 = vec![false, true, true, false, false, true, false, true];
        assert!(
            compute_estimate(
                &AlpState {
                    alpha: 1.,
                    scale: 0.5,
                    hashers: index_identify_functions(8),
                    z: z3
                },
                &0
            ) == 6.0
        );

        Ok(())
    }

    #[test]
    fn test_construct_and_post_process() -> Fallible<()> {
        let mut x = HashMap::new();
        x.insert(0, 7);
        x.insert(42, 12);
        x.insert(100, 5);

        let alp_meas = make_alp_state::<i32, i32, f64>(
            MapDomain::default(),
            L1Distance::default(),
            2.,
            24,
            Some(24),
            None,
            None,
        )?;
        let alp_state = alp_meas.invoke(&x)?;

        let postprocessor = post_alp_state_to_queryable();
        let mut queryable = postprocessor.eval(&alp_state)?;

        queryable.eval(&0)?;
        queryable.eval(&42)?;
        queryable.eval(&100)?;
        queryable.eval(&1000)?;

        Ok(())
    }

    #[test]
    fn test_post_process_measurement() -> Fallible<()> {
        let mut x = HashMap::new();
        x.insert(0, 7);
        x.insert(42, 12);
        x.insert(100, 5);

        let alp_meas = make_alp_queryable::<i32, i32, f64>(
            MapDomain {
                key_domain: AtomDomain::default(),
                value_domain: AtomDomain::default(),
            },
            L1Distance::default(),
            2.,
            24,
            Some(24),
            None,
            None,
        )?;

        assert_eq!(alp_meas.map(&1)?, 2.);
        let mut queryable = alp_meas.invoke(&x)?;

        queryable.eval(&0)?;
        queryable.eval(&42)?;
        queryable.eval(&100)?;
        queryable.eval(&1000)?;

        Ok(())
    }
}

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use num::{Integer, ToPrimitive};
use rug::{float::Round, ops::AddAssignRound, ops::DivAssignRound, Float};

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{AllDomain, MapDomain};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::measures::MaxDivergence;
use crate::metrics::L1Distance;
use crate::traits::samplers::{fill_bytes, SampleBernoulli};
use crate::traits::{CheckNull, DistanceConstant, Float as TFloat, InfCast};
use std::collections::hash_map::DefaultHasher;

const ALPHA_DEFAULT: u32 = 4;
const SIZE_FACTOR_DEFAULT: u32 = 50;

/// Implementation of a mechanism for representing sparse integer queries e.g. a sparse histogram.
/// The mechanism was introduced in the paper:
///
/// "Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access"
///
/// Available here: arxiv.org/abs/2106.10068

/// Input domain. The mechanism is designed for settings where the domain of K is huge.
type SparseDomain<K, C> = MapDomain<AllDomain<K>, AllDomain<C>>;

// Types used to store the DP projection.
type BitVector = Vec<bool>;
type HashFunctions<K> = Vec<Rc<dyn Fn(&K) -> usize>>;
#[derive(Clone)]
#[doc(hidden)]
pub struct AlpState<K, T> {
    alpha: T,
    scale: T,
    h: HashFunctions<K>,
    z: BitVector,
}
impl<K, T> CheckNull for AlpState<K, T> {
    fn is_null(&self) -> bool {
        false
    }
}
type AlpDomain<K, T> = AllDomain<AlpState<K, T>>;

// hash function with type: [2^64] -> [2^l]
// Computes ((a*x + b) mod 2^64) div 2^(64-l)
// a and b are sampled uniformly at random from [2^64]
// a must be odd
// The hash function is 2-approximate universal and uniform
// See http://hjemmesider.diku.dk/~jyrki/Paper/CP-11.4.1997.pdf
// "A Reliable Randomized Algorithm for the Closest-Pair Problem"
fn hash(x: u64, a: u64, b: u64, l: u32) -> usize {
    (a.wrapping_mul(x).wrapping_add(b) >> (64 - l)) as usize
}
fn pre_hash<K: Hash>(x: K) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

fn sample_hash_function<K>(l: u32) -> Fallible<Rc<dyn Fn(&K) -> usize>>
where
    K: Clone + Hash,
{
    let mut buf = [0u8; 8];
    fill_bytes(&mut buf)?;
    let a = u64::from_ne_bytes(buf) | 1u64;
    fill_bytes(&mut buf)?;
    let b = u64::from_ne_bytes(buf);
    Ok(Rc::new(move |x: &K| hash(pre_hash(x), a, b, l)))
}

// Returns ceil(log_2(x))
fn exponent_next_power_of_two(x: u64) -> u32 {
    let exp = 63 - x.leading_zeros();
    if x > (1 << exp) {
        exp + 1
    } else {
        exp
    }
}

// Multiplies x with scale/alpha and applies randomized rounding to return an integer
fn scale_and_round<C, T>(x: C, alpha: T, scale: T) -> Fallible<usize>
where
    C: Integer + ToPrimitive,
    T: InfCast<Float>,
    Float: InfCast<T>,
{
    let mut scalar = Float::neg_inf_cast(scale)?;
    scalar.div_assign_round(Float::inf_cast(alpha)?, Round::Down);
    // Truncate bits that represents values below 2^-53
    scalar.set_prec_round(
        (f64::MANTISSA_DIGITS as i32 - scalar.get_exp().unwrap()).max(1) as u32,
        Round::Down,
    );

    let r = Float::with_val(
        f64::MANTISSA_DIGITS * 2,
        x.max(C::zero()).to_u64().unwrap_or_default(),
    ) * scalar;
    let floored = f64::inf_cast(r.clone().floor())? as usize;

    match bool::sample_bernoulli(f64::inf_cast(r.fract())?, false)? {
        true => Ok(floored + 1),
        false => Ok(floored),
    }
}

// Probability of flipping bits = 1 / (alpha + 2)
fn compute_prob<T: InfCast<Float>>(alpha: T) -> f64
where
    Float: InfCast<T>,
{
    let mut a = Float::neg_inf_cast(alpha).expect("impl is infallible");
    a.add_assign_round(2, Round::Down);
    a.recip_round(Round::Up);
    // Round up to preserve privacy
    f64::inf_cast(a).expect("impl is infallible")
}

// Due to privacy concerns the current implementation discards bits with significance less than 2^-52 from scale/alpha
// Concern (Mike): validate that this constant remains correct when T is parameterized by f32
fn check_parameters<T: InfCast<Float>>(alpha: T, scale: T) -> bool
where
    Float: InfCast<T>,
{
    let scale = Float::inf_cast(scale).expect("impl is infallible");
    let alpha = Float::neg_inf_cast(alpha).expect("impl is infallible");
    scale * Float::with_val(53, 52).exp2() < alpha
}

// Computes the DP projection. This corresponds to Algorithm 4 in the paper
fn compute_projection<K, C, T>(
    x: &HashMap<K, C>,
    h: &HashFunctions<K>,
    alpha: T,
    scale: T,
    s: usize,
) -> Fallible<BitVector>
where
    C: Clone + Integer + ToPrimitive,
    T: Clone + InfCast<Float>,
    Float: InfCast<T>,
{
    let mut z = vec![false; s];

    for (k, v) in x.iter() {
        let round = scale_and_round(v.clone(), alpha.clone(), scale.clone())?;
        h.iter().take(round).for_each(|f| z[f(k) % s] = true); // ^= true TODO: Hash collisions can be handled using OR or XOR
    }

    let p = compute_prob(alpha);

    z.iter()
        .map(|b| bool::sample_bernoulli(p, false).map(|flip| b ^ flip))
        .collect()
}

// Estimate the value of an entry based on its noisy bitrepresentation. This is Algorithm 3 in the paper
fn estimate_unary<T>(v: &Vec<bool>) -> T
where
    T: num::Float,
{
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

// This is Algorithm 5 in the paper
fn compute_estimate<K, T>(state: &AlpState<K, T>, key: &K) -> T
where
    T: num::Float,
{
    let v = state
        .h
        .iter()
        .map(|f| state.z[f(key) % state.z.len()])
        .collect::<Vec<_>>();

    estimate_unary::<T>(&v) * T::from(state.alpha).unwrap() / state.scale
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
/// * `alpha` - Parameter used for scaling and determining p in randomized response step. The default value is 4.
/// * `scale` - Privacy loss parameter. This is equal to epsilon/sensitivity.
/// * `s` - Size of the projection. This should be sufficiently large to limit hash collisions.
/// * `h` - Hash functions used to project and estimate entries. The hash functions are not allowed to panic on any input.
/// The hash functions in `h` should have type K -> \[s\]. To limit collisions the functions should be universal and uniform.
/// The evaluation time of post-processing is O(h.len()).
pub fn make_base_alp_with_hashers<K, C, T>(
    alpha: T,
    scale: T,
    s: usize,
    h: HashFunctions<K>,
) -> Fallible<Measurement<SparseDomain<K, C>, AlpDomain<K, T>, L1Distance<C>, MaxDivergence<T>>>
where
    K: 'static + Eq + Hash + CheckNull,
    C: 'static + Clone + Integer + CheckNull + DistanceConstant<C> + ToPrimitive,
    T: 'static + num::Float + DistanceConstant<T> + InfCast<Float> + InfCast<C>,
    AlpState<K, T>: CheckNull,
    Float: InfCast<T>,
{
    if alpha.is_sign_negative() || alpha.is_zero() {
        return fallible!(MakeMeasurement, "alpha must be positive");
    }
    if scale.is_sign_negative() || scale.is_zero() {
        return fallible!(MakeMeasurement, "scale must be positive");
    }
    if s == 0 {
        return fallible!(MakeMeasurement, "s can not be zero");
    }
    if check_parameters(alpha, scale) {
        return fallible!(
            MakeMeasurement,
            "scale divided by alpha must be above 2^-52"
        );
    }

    Ok(Measurement::new(
        MapDomain {
            key_domain: AllDomain::new(),
            value_domain: AllDomain::new(),
        },
        AllDomain::new(),
        Function::new_fallible(move |x: &HashMap<K, C>| {
            let z = compute_projection(x, &h, alpha, scale, s)?;
            Ok(AlpState {
                alpha,
                scale,
                h: h.clone(),
                z,
            })
        }),
        L1Distance::default(),
        MaxDivergence::default(),
        PrivacyMap::new_from_constant(scale),
    ))
}

/// Measurement to compute a DP projection of bounded sparse data.
///
/// The size of the projection is O(total * size_factor * scale / alpha).
/// The evaluation time of post-processing is O(beta * scale / alpha).
///
/// # Citations
/// * [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access](https://arxiv.org/abs/2106.10068)
///   Algorithm 4
///
/// # Arguments
/// * `total` - Estimate or true value of the sum of all values in the input.
/// This should be an upper bound if the true total is private.
/// * `size_factor` - Optional multiplier for setting the size of the projection. There is a memory/utility trade-off.
/// The value should be sufficient large to limit hash collisions. The default value is 50.
/// * `alpha` - Optional parameter used for scaling and determining p in randomized response step. The default value is 4.
/// * `scale` - Privacy loss parameter. This is equal to epsilon/sensitivity.
/// * `beta` - Upper bound on values. Entries above beta are clamped.
pub fn make_base_alp<K, C, T>(
    total: usize,
    size_factor: Option<u32>,
    alpha: Option<T>,
    scale: T,
    beta: C,
) -> Fallible<Measurement<SparseDomain<K, C>, AlpDomain<K, T>, L1Distance<C>, MaxDivergence<T>>>
where
    K: 'static + Eq + Hash + Clone + CheckNull,
    C: 'static + Clone + Integer + CheckNull + DistanceConstant<C> + InfCast<T> + ToPrimitive,
    T: 'static + num::Float + DistanceConstant<T> + InfCast<Float> + InfCast<C>,
    AlpState<K, T>: CheckNull,
    Float: InfCast<T>,
{
    let factor = size_factor.unwrap_or(SIZE_FACTOR_DEFAULT) as f64;
    let alpha = alpha.unwrap_or_else(|| T::from(ALPHA_DEFAULT).unwrap());

    let beta: f64 = T::inf_cast(beta)?
        .to_f64()
        .ok_or_else(|| err!(MakeTransformation, "failed to parse beta"))?;
    let quotient = (scale / alpha)
        .to_f64()
        .ok_or_else(|| err!(MakeTransformation, "failed to parse scale/alpha"))?;
    let m = (beta * quotient).ceil() as usize;

    let exp = exponent_next_power_of_two((factor * total as f64 * quotient) as u64);
    let h = (0..m)
        .map(|_| sample_hash_function(exp))
        .collect::<Fallible<HashFunctions<K>>>()?;

    make_base_alp_with_hashers(alpha, scale, 1 << exp, h)
}

/// Wrap the AlpState in a Queryable object.
///
/// The Queryable object works similar to a dictionary.
/// Note that the access time is O(state.h.len()).
pub fn post_process<K, T>(state: AlpState<K, T>) -> Queryable<K, AllDomain<T>>
where
    T: 'static + TFloat,
    K: 'static + Clone,
{
    Queryable::new_concrete(move |key: &K| Ok(compute_estimate(&state, key)))
}

/// Wrapper Measurement. See [`post_process`].
pub fn make_alp_histogram_post_process<K, C, T>(
    m: &Measurement<SparseDomain<K, C>, AlpDomain<K, T>, L1Distance<C>, MaxDivergence<T>>,
) -> Fallible<
    Measurement<
        SparseDomain<K, C>,
        AllDomain<Queryable<K, AllDomain<T>>>,
        L1Distance<C>,
        MaxDivergence<T>,
    >,
>
where
    K: 'static + Eq + Hash + CheckNull + Clone,
    C: 'static + Clone + CheckNull,
    T: 'static + TFloat,
    HashMap<K, C>: Clone,
    AlpState<K, T>: Clone,
{
    let function = m.function.clone();
    Ok(Measurement::new(
        m.input_domain.clone(),
        AllDomain::new(),
        Function::new_fallible(move |x| function.eval(x).map(post_process)),
        m.input_metric.clone(),
        m.output_measure.clone(),
        m.privacy_map.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx<T>(i: usize) -> Rc<dyn Fn(&T) -> usize> {
        Rc::new(move |_| i)
    }

    // Functions that always return its index
    fn index_identify_functions<T>(n: usize) -> HashFunctions<T> {
        (0..n).map(|i| idx(i)).collect::<HashFunctions<T>>()
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
        let alp = make_base_alp_with_hashers::<u32, u32, f64>(
            1.,
            1.0,
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
        let alp = make_base_alp_with_hashers::<u32, u32, f64>(1., 1.0, s, h)?;

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
                    h: index_identify_functions(8),
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
                    h: index_identify_functions(8),
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
                    h: index_identify_functions(8),
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

        let alp = make_base_alp::<i32, i32, f64>(24, None, None, 2., 24)?;

        let state = alp.function.eval(&x)?;

        let mut query = post_process(state);

        query.eval(&0)?;
        query.eval(&42)?;
        query.eval(&100)?;
        query.eval(&1000)?;

        Ok(())
    }

    #[test]
    fn test_post_process_measurement() -> Fallible<()> {
        let mut x = HashMap::new();
        x.insert(0, 7);
        x.insert(42, 12);
        x.insert(100, 5);

        let alp = make_base_alp::<i32, i32, f64>(24, None, None, 2., 24)?;

        let wrapped = make_alp_histogram_post_process(&alp)?;

        assert_eq!(wrapped.map(&1)?, 2.);

        let mut query = wrapped.function.eval(&x)?;

        query.eval(&0)?;
        query.eval(&42)?;
        query.eval(&100)?;
        query.eval(&1000)?;

        Ok(())
    }
}

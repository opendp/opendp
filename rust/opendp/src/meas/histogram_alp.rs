use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use num::{Integer, FromPrimitive};
#[cfg(feature="use-mpfr")]
use rug::{Float, ops::DivAssignRound, float::Round};

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L1Distance, MaxDivergence};
use crate::dom::{AllDomain, MapDomain, SizedDomain};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::traits::DistanceCast;
use crate::samplers::{fill_bytes, CastInternalReal, SampleBernoulli};

const ALPHA_DEFAULT : u32 = 4;
const SIZE_FACTOR_DEFAULT : u32 = 30;

type SizedHistogramDomain<K, C> = SizedDomain<MapDomain<AllDomain<K>, AllDomain<C>>>;

type BitVector = Vec<bool>;
type HashFunctions<K> = Vec<Rc<dyn Fn(&K) -> usize>>;
pub struct AlpState<K, T>{
    alpha: u32,
    scale: T,
    h: HashFunctions<K>,
    z: BitVector 
}
type AlpDomain<K, T> = AllDomain<AlpState<K, T>>;

// hash function with type: [2^64] -> [2^l]
// Computes ((a*x + b) mod 2^64) div 2^(64-l)
// The hash function is 2-approximate universal and uniform
// See http://hjemmesider.diku.dk/~jyrki/Paper/CP-11.4.1997.pdf
// a and b are sampled uniformly at random from [2^64]
// a must be odd
fn hash(x: u64, a: u64, b:u64, l: u32) -> usize {
    (a.wrapping_mul(x).wrapping_add(b) >> (64 - l)) as usize
}

fn sample_hash_function<K>(l: u32) -> Fallible<Rc<dyn Fn(&K) -> usize>> 
    where K: Clone + Into<u64> {
    let mut buf = [0u8; 8];
    fill_bytes(&mut buf)?;
    let a = u64::from_be_bytes(buf) | 1u64;
    fill_bytes(&mut buf)?;
    let b = u64::from_be_bytes(buf);
    Ok(Rc::new(move |x: &K| hash(x.clone().into(), a, b, l)))
}

fn exponent_next_power_of_two(x: u64) -> u32 {
    let exp = 63 - x.leading_zeros();
    return if x > (1 << exp) { exp + 1 } else { exp }
}

#[cfg(feature="use-mpfr")]
fn scale_and_round<C, T>(x : C, alpha: u32, scale: T) -> Fallible<usize> 
    where C: Copy + DistanceCast,
          T: Copy + DistanceCast {
    // TODO: Precision is currently simply chosen to be very high
    let mut invalpha = Float::with_val(150, 1);
    invalpha.div_assign_round(Float::with_val(150, alpha), Round::Down);
    let r = Float::with_val(150, f64::distance_cast(x)?) * Float::with_val(150, f64::distance_cast(scale)?) * invalpha;
    let floored = f64::from_internal(r.clone().floor()) as usize;
    // TODO: Potential rounding when casting to f64
    match bool::sample_bernoulli(f64::from_internal(r.fract()), false)? {
        true => Ok(floored + 1),
        false => Ok(floored)
    }
}

#[cfg(not(feature="use-mpfr"))]
fn scale_and_round<C, T>(x : C, alpha: &u32, scale: &T) -> Fallible<usize> {
    unimplemented!()
}

#[cfg(feature="use-mpfr")]
fn compute_prob(alpha: u32) -> f64 {
    let mut p = Float::with_val(53, 1.0);
    p.div_assign_round( Float::with_val(53, alpha + 2), Round::Up); // Round up to preserve privacy
    f64::from_internal(p)
}

#[cfg(not(feature="use-mpfr"))]
fn compute_prob(alpha: &u32) -> f64 {
    unimplemented!()
}

fn compute_projection<K, C, T>(x: &HashMap<K, C>, h: &HashFunctions<K>, alpha: u32, scale: T, s: usize) -> Fallible<BitVector> 
    where C: Copy + DistanceCast,
          T: Copy + CastInternalReal + DistanceCast {
    let mut z = vec![false; s];

    for (k, v) in x.iter() {
        let round = scale_and_round(*v, alpha, scale)?; 
        h.iter().take(round).for_each(|f| z[f(k) % s] = true); // ^= true TODO: Hash collisions can be handled using OR or XOR
    }

    let p = compute_prob(alpha);

    z.iter().map(|b| bool::sample_bernoulli(p , false).map(|flip| b ^ flip)).collect::<Fallible<Vec<bool>>>()
}

fn estimate_unary(v: &Vec<bool>) -> f64 {
    let mut prefix_sum = Vec::with_capacity(v.len() + 1 as usize);
    prefix_sum.push(0);

    v.iter().map(|b| if *b {1} else {-1}).for_each(|x| prefix_sum.push(prefix_sum.last().unwrap() + x));
    
    let high = prefix_sum.iter().max().unwrap();
    let peaks = prefix_sum.iter().enumerate()
            .filter_map(|(idx, height)| if high == height {Some(idx)} else {None}).collect::<Vec<_>>();
    
    // Return the average position
    peaks.iter().sum::<usize>() as f64 / peaks.len() as f64
}

fn compute_estimate<K, T>(state: &AlpState<K, T>, key: &K) -> T 
    where T: FromPrimitive + num::Float {
    let v = state.h.iter().map(|f| state.z[f(key) % state.z.len()]).collect::<Vec<_>>();

    T::from_f64(estimate_unary(&v) * state.alpha as f64).unwrap() / state.scale
}

pub fn make_alp_histogram<K, C, T>(n: usize, alpha: u32, scale: T, s: usize, h: HashFunctions<K>) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash,
          C: 'static + Copy + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast + CastInternalReal {
    
    if alpha == 0 {
        return fallible!(MakeMeasurement, "alpha must be positive")
    }
    if scale.is_sign_negative() || scale.is_zero() {
        return fallible!(MakeMeasurement, "scale must be positive")
    }
    if s == 0 {
        return fallible!(MakeMeasurement, "s can not be zero")
    }
    
    Ok(Measurement::new(
        SizedDomain::new(MapDomain { key_domain: AllDomain::new(), value_domain: AllDomain::new() }, n),
        AllDomain::new(),
        Function::new_fallible(move |x: &HashMap<K, C>| {
            let z = compute_projection(x, &h, alpha, scale, s)?;
            Ok(AlpState { alpha, scale, h:h.clone(), z })
        }),
        L1Distance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale * T::distance_cast(alpha)?)
    ))
}

pub fn make_alp_histogram_parameterized<K, C, T>(n: usize, alpha: u32, scale: T, beta: C, size_factor: u32) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash + Clone + Into<u64>,
          C: 'static + Copy + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast + CastInternalReal {
    
    let m = (f64::distance_cast(beta)? * f64::distance_cast(scale)? / alpha as f64).ceil() as usize;

    let exp = exponent_next_power_of_two(u64::distance_cast(size_factor as f64 * f64::distance_cast(beta)? * f64::distance_cast(scale)? / alpha as f64)?);
    let h = (0..m).map(|_| sample_hash_function(exp)).collect::<Fallible<HashFunctions<K>>>()?;

    make_alp_histogram(n, alpha, scale, 1 << exp, h)
}

pub fn make_alp_histogram_simple<K, C, T>(n: usize, scale: T, beta: C) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash + Clone + Into<u64>,
          C: 'static + Copy + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast + CastInternalReal {
    
    make_alp_histogram_parameterized(n, ALPHA_DEFAULT, scale, beta, SIZE_FACTOR_DEFAULT)
}

pub fn post_process<K, T>(state: AlpState<K, T>) -> Queryable<AlpState<K, T>, K, T> 
    where T: num::Float + FromPrimitive {
    Queryable::new(
        state,
        move |state: AlpState<K, T>, key: &K| {
            let estimate = compute_estimate(&state, key);
            Ok((state, estimate))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn idx<T>(i: usize) -> Rc<dyn Fn(&T) -> usize> {
        Rc::new(move |_| i)
    }

    // Functions that always return its index
    fn index_identify_functions<T> (n: usize) -> HashFunctions<T> {
        (0..n).map(|i| {
            idx(i)
        }).collect::<HashFunctions<T>>()
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
        let alp = make_alp_histogram::<u32, u32, f64>(10, 1, 1.0, beta, index_identify_functions(beta))?;

        assert!(alp.privacy_relation.eval(&1, &1.)?);
        assert!(!alp.privacy_relation.eval(&1, &0.999)?);

        let mut x = HashMap::new();
        x.insert(42, 10);

        alp.function.eval(&x.clone())?;

        // Values exceeding beta is truncated internally
        x.insert(42, 10000);
        alp.function.eval(&x.clone())?;

        Ok(())
    }

    #[test]
    fn test_alp_construction_err() -> Fallible<()> {
        let s = 5;
        // Hash functions return values out of range
        // Handle silently using modulo
        // Returning an error would violate privacy
        let h = index_identify_functions(20);
        let alp = make_alp_histogram::<u32, u32, f64>(3, 1, 1.0, s, h)?;

        let mut x = HashMap::new();
        x.insert(42, 3);

        alp.function.eval(&x.clone())?;

        Ok(())
    }

    #[test]
    fn test_estimate_unary() -> Fallible<()> {
        let z1 = vec![true, true, true, false, true, false, false, true];
        assert!(estimate_unary(&z1) == 4.0);

        let z2 = vec![true, false, false, false, true, false, false, true];
        assert!(estimate_unary(&z2) == 1.0);

        let z3 = vec![false, true, true, false, false, true, false, true];
        assert!(estimate_unary(&z3) == 3.0);

        Ok(())
    }

    #[test]
    fn test_compute_estimate() -> Fallible<()> {
        let z1 = vec![true, true, true, false, true, false, false, true];
        assert!(compute_estimate(&AlpState {alpha:3, scale:1.0, h:index_identify_functions(8), z:z1}, &0) == 12.0);

        let z2 = vec![true, false, false, false, true, false, false, true];
        assert!(compute_estimate(&AlpState {alpha:1, scale:2.0, h:index_identify_functions(8), z:z2}, &0) == 0.5);

        let z3 = vec![false, true, true, false, false, true, false, true];
        assert!(compute_estimate(&AlpState {alpha:1, scale:0.5, h:index_identify_functions(8), z:z3}, &0) == 6.0);

        Ok(())
    }
}
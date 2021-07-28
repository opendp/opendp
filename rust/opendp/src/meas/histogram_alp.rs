use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use num::{Integer, FromPrimitive, ToPrimitive};
#[cfg(feature="use-mpfr")]
use rug::{Float, ops::DivAssignRound, ops::AddAssignRound, float::Round};

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
#[derive(Clone)]
pub struct AlpState<K, T>{
    alpha: T,
    scale: T,
    h: HashFunctions<K>,
    z: BitVector 
}
type AlpDomain<K, T> = AllDomain<AlpState<K, T>>;

// hash function with type: [2^64] -> [2^l]
// Computes ((a*x + b) mod 2^64) div 2^(64-l)
// a and b are sampled uniformly at random from [2^64]
// a must be odd
// The hash function is 2-approximate universal and uniform
// See http://hjemmesider.diku.dk/~jyrki/Paper/CP-11.4.1997.pdf
fn hash(x: u64, a: u64, b:u64, l: u32) -> usize {
    (a.wrapping_mul(x).wrapping_add(b) >> (64 - l)) as usize
}

fn sample_hash_function<K>(l: u32) -> Fallible<Rc<dyn Fn(&K) -> usize>> 
    where K: Clone + ToPrimitive {
    let mut buf = [0u8; 8];
    fill_bytes(&mut buf)?;
    let a = u64::from_ne_bytes(buf) | 1u64;
    fill_bytes(&mut buf)?;
    let b = u64::from_ne_bytes(buf);
    Ok(Rc::new(move |x: &K| hash(x.to_u64().unwrap_or_default(), a, b, l)))
}

fn exponent_next_power_of_two(x: u64) -> u32 {
    let exp = 63 - x.leading_zeros();
    if x > (1 << exp) { exp + 1 } else { exp }
}

#[cfg(feature="use-mpfr")]
fn scale_and_round<C, T>(x : C, alpha: T, scale: T) -> Fallible<usize> 
    where C: Integer + ToPrimitive,
          T: CastInternalReal {
    let mut scalar = scale.into_internal();
    scalar.div_assign_round(alpha.into_internal(), Round::Down);
    // Truncate bits that represents values below 2^-53
    scalar.set_prec_round((f64::MANTISSA_DIGITS as i32 - scalar.get_exp().unwrap()).max(1) as u32, Round::Down);

    let r = Float::with_val(f64::MANTISSA_DIGITS * 2, x.max(C::zero()).to_u64().unwrap()) * scalar;
    let floored = f64::from_internal(r.clone().floor()) as usize;
    
    match bool::sample_bernoulli(f64::from_internal(r.fract()), false)? {
        true => Ok(floored + 1),
        false => Ok(floored)
    }
}

#[cfg(not(feature="use-mpfr"))]
fn scale_and_round<C, T>(x : C, alpha: T, scale: T) -> Fallible<usize> {
    unimplemented!()
}

#[cfg(feature="use-mpfr")]
fn compute_prob<T: CastInternalReal>(alpha: T) -> f64 {
    let mut a = alpha.into_internal();
    a.add_assign_round(2, Round::Down);
    let mut p = 1f64.into_internal();
    p.div_assign_round( a, Round::Up); // Round up to preserve privacy
    f64::from_internal(p)
}

#[cfg(not(feature="use-mpfr"))]
fn compute_prob<T>(alpha: T) -> f64 {
    unimplemented!()
}

#[cfg(feature="use-mpfr")]
fn check_parameters<T : CastInternalReal>(alpha: T, scale: T) -> bool {
    scale.into_internal() * Float::with_val(53, 52).exp2() < alpha.into_internal()
}

#[cfg(not(feature="use-mpfr"))]
fn check_parameters(alpha: T, scale: T) -> bool {
    unimplemented!()
}

fn compute_projection<K, C, T>(x: &HashMap<K, C>, h: &HashFunctions<K>, alpha: T, scale: T, s: usize) -> Fallible<BitVector> 
    where C: Copy + Integer + ToPrimitive,
          T: Copy + CastInternalReal {
    let mut z = vec![false; s];

    for (k, v) in x.iter() {
        let round = scale_and_round(*v, alpha, scale)?; 
        h.iter().take(round).for_each(|f| z[f(k) % s] = true); // ^= true TODO: Hash collisions can be handled using OR or XOR
    }

    let p = compute_prob(alpha);

    z.iter().map(|b| bool::sample_bernoulli(p , false).map(|flip| b ^ flip)).collect()
}

fn estimate_unary<T>(v: &Vec<bool>) -> T
    where T : FromPrimitive + num::Float {
    let mut prefix_sum = Vec::with_capacity(v.len() + 1usize);
    prefix_sum.push(0);

    v.iter().map(|b| if *b {1} else {-1}).for_each(|x| prefix_sum.push(prefix_sum.last().unwrap() + x));
    
    let high = prefix_sum.iter().max().unwrap();
    let peaks = prefix_sum.iter().enumerate()
            .filter_map(|(idx, height)| if high == height {Some(idx)} else {None}).collect::<Vec<_>>();
    
    // Return the average position
    T::from(peaks.iter().sum::<usize>()).unwrap() / T::from(peaks.len()).unwrap()
}

fn compute_estimate<K, T>(state: &AlpState<K, T>, key: &K) -> T 
    where T: FromPrimitive + num::Float {
    let v = state.h.iter().map(|f| state.z[f(key) % state.z.len()]).collect::<Vec<_>>();

    estimate_unary::<T>(&v) * T::from(state.alpha).unwrap() / state.scale
}

pub fn make_alp_histogram<K, C, T>(n: usize, alpha: T, scale: T, s: usize, h: HashFunctions<K>) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash,
          C: 'static + Copy + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast + CastInternalReal {
    
    if alpha.is_sign_negative() || alpha.is_zero() {
        return fallible!(MakeMeasurement, "alpha must be positive")
    }
    if scale.is_sign_negative() || scale.is_zero() {
        return fallible!(MakeMeasurement, "scale must be positive")
    }
    if s == 0 {
        return fallible!(MakeMeasurement, "s can not be zero")
    }
    if check_parameters(alpha, scale) {
        return fallible!(MakeMeasurement, "scale divided by alpha must be above 2^-52")
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
        PrivacyRelation::new_from_constant(scale)
    ))
}

pub fn make_alp_histogram_parameterized<K, C, T>(n: usize, alpha: T, scale: T, beta: C, size_factor: u32) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash + Clone + ToPrimitive,
          C: 'static + Copy + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast + CastInternalReal {
    
    let m = (beta.to_f64().unwrap() * (scale / alpha).to_f64().unwrap()).ceil() as usize;
    
    let exp = exponent_next_power_of_two(u64::distance_cast(size_factor as f64 * n as f64 * (scale / alpha).to_f64().unwrap())?);
    let h = (0..m).map(|_| sample_hash_function(exp)).collect::<Fallible<HashFunctions<K>>>()?;

    make_alp_histogram(n, alpha, scale, 1 << exp, h)
}

pub fn make_alp_histogram_simple<K, C, T>(n: usize, scale: T, beta: C) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash + Clone + ToPrimitive,
          C: 'static + Copy + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast + CastInternalReal {
    
    make_alp_histogram_parameterized(n, T::from(ALPHA_DEFAULT).unwrap(), scale, beta, SIZE_FACTOR_DEFAULT)
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

// TODO: Could be refactored to a general post_processing function
pub fn make_histogram_alp_post_process<K, C, T>(m : Measurement<SizedHistogramDomain<K, C>,AlpDomain<K, T>,L1Distance<C>, MaxDivergence<T>>) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, AllDomain<Queryable<AlpState<K, T>, K, T>>, L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash + Clone,
          C: 'static,
          T: 'static + num::Float + FromPrimitive {
        let f0 = m.function;
        let f1 = Function::new(move |x : &AlpState<K, T>| post_process(x.clone()));
        Ok(Measurement::new(
            m.input_domain, 
            AllDomain::new(), 
            Function::make_chain(&f1, &f0),
            m.input_metric,
            m.output_measure, 
            m.privacy_relation))
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
        let alp = make_alp_histogram::<u32, u32, f64>(10, 1., 1.0, beta, index_identify_functions(beta))?;

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
    fn test_alp_construction_out_of_range() -> Fallible<()> {
        let s = 5;
        // Hash functions return values out of range
        // Handle silently using modulo
        // Returning an error would violate privacy
        let h = index_identify_functions(20);
        let alp = make_alp_histogram::<u32, u32, f64>(3, 1., 1.0, s, h)?;

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
        assert!(compute_estimate(&AlpState {alpha:3., scale:1.0, h:index_identify_functions(8), z:z1}, &0) == 12.0);

        let z2 = vec![true, false, false, false, true, false, false, true];
        assert!(compute_estimate(&AlpState {alpha:1., scale:2.0, h:index_identify_functions(8), z:z2}, &0) == 0.5);

        let z3 = vec![false, true, true, false, false, true, false, true];
        assert!(compute_estimate(&AlpState {alpha:1., scale:0.5, h:index_identify_functions(8), z:z3}, &0) == 6.0);

        Ok(())
    }

    #[test]
    fn test_construct_and_post_process() -> Fallible<()> {
        let mut x = HashMap::new();
        x.insert(0, 7);
        x.insert(42, 12);
        x.insert(100, 5);

        let alp = make_alp_histogram_simple::<i32,i32,f64>(24, 2., 24)?;

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

        let alp = make_alp_histogram_simple::<i32,i32,f64>(24, 2., 24)?;

        let wrapped = make_histogram_alp_post_process(alp)?;
        
        assert!(wrapped.privacy_relation.eval(&1, &2.)?);
        assert!(!wrapped.privacy_relation.eval(&1, &1.999)?);

        let mut query = wrapped.function.eval(&x)?;

        query.eval(&0)?;
        query.eval(&42)?;
        query.eval(&100)?;
        query.eval(&1000)?;

        Ok(())
    }
}
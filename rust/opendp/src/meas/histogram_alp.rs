use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use num::Integer;
#[cfg(feature="use-mpfr")]
use rug::{Float, ops::DivAssignRound, float::Round, Assign, ops::AssignRound};

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L1Distance, MaxDivergence};
use crate::dom::{AllDomain, MapDomain, SizedDomain};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::traits::DistanceCast;
use crate::samplers::{CastInternalReal, SampleBernoulli};

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

fn sample_hash_functions<K>(l: u32, m: usize) -> Fallible<HashFunctions<K>> {
    unimplemented!()
}

#[cfg(feature="use-mpfr")]
fn scale_and_round<C, T>(x : C, alpha: u32, scale: T) -> Fallible<usize> 
    where C: Copy + DistanceCast,
          T: Copy + DistanceCast {
    let r = Float::with_val(150, f64::distance_cast(x)?) * Float::with_val(150, f64::distance_cast(scale)?) / Float::with_val(150, alpha);
    let floored = f64::from_internal(r.clone().floor()) as usize;
    // TODO: Potential rounding when casting to f64. Should be considered for privacy proof
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
        h.iter().take(round).for_each(|f| z[f(k) % s] = true); // TODO: Hash collisions can be handled using OR or XOR
    }

    let p = compute_prob(alpha);

    z.iter().map(|b| bool::sample_bernoulli(p , false).map(|flip| b ^ flip)).collect::<Fallible<Vec<bool>>>()
}

fn compute_estimate<K, T>(state: &AlpState<K, T>, key: &K) -> T {
    unimplemented!()
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

pub fn make_alp_histogram_parameterized<K, C, T>(n: usize, alpha: T, scale: T, beta: C, size_factor: T) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash,
          C: 'static + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast {
    
    unimplemented!()
}

pub fn make_alp_histogram_simple<K, C, T>(n: usize, scale: T, beta: C) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash,
          C: 'static + Integer + DistanceCast,
          T: 'static + num::Float + DistanceCast {
    
    make_alp_histogram_parameterized(n, T::distance_cast(ALPHA_DEFAULT)?, scale, beta, T::distance_cast(SIZE_FACTOR_DEFAULT)?)
}

pub fn post_process<K, T>(state: AlpState<K, T>) -> Queryable<AlpState<K, T>, K, T> {
    Queryable::new(
        state,
        move |state: AlpState<K, T>, key: &K| {
            let estimate = compute_estimate(&state, key);
            Ok((state, estimate))
    })
}
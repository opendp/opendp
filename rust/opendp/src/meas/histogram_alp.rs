use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use num::{Integer, Float};

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L1Distance, MaxDivergence};
use crate::dom::{AllDomain, MapDomain, SizedDomain};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::traits::DistanceCast;

const ALPHA_DEFAULT : u32 = 4;
const SIZE_FACTOR_DEFAULT : u32 = 30;

type SizedHistogramDomain<K, C> = SizedDomain<MapDomain<AllDomain<K>, AllDomain<C>>>;

type BitVector = Vec<bool>;
type HashFunctions<K> = Vec<Rc<dyn Fn(&K) -> usize>>;
pub struct AlpState<K, T>{
    alpha: T,
    scale: T,
    h: HashFunctions<K>,
    z: BitVector 
}
type AlpDomain<K, T> = AllDomain<AlpState<K, T>>;

fn sample_hash_functions<K>(l: u32, m: usize) -> Fallible<HashFunctions<K>> {
    unimplemented!()
}

fn compute_projection<K, C, T>(x: &HashMap<K, C>, h: &HashFunctions<K>, alpha: T, scale: T, s: usize) -> BitVector {
    unimplemented!()
}

fn compute_estimate<K, T>(state: &AlpState<K, T>, key: &K) -> T {
    unimplemented!()
}

pub fn make_alp_histogram<K, C, T>(n: usize, alpha: T, scale: T, s: usize, h: HashFunctions<K>) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash,
          C: 'static + Integer + DistanceCast,
          T: 'static + Float + DistanceCast {
    
    if alpha.is_sign_negative() || alpha.is_zero() {
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
        Function::new(move |x: &HashMap<K, C>| {
            let z = compute_projection(x, &h, alpha, scale, s);
            AlpState { alpha, scale, h:h.clone(), z }
        }),
        L1Distance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale * alpha)
    ))
}

pub fn make_alp_histogram_parameterized<K, C, T>(n: usize, alpha: T, scale: T, beta: C, size_factor: T) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash,
          C: 'static + Integer + DistanceCast,
          T: 'static + Float + DistanceCast {
    
    unimplemented!()
}

pub fn make_alp_histogram_simple<K, C, T>(n: usize, scale: T, beta: C) 
        -> Fallible<Measurement<SizedHistogramDomain<K, C>, 
                                AlpDomain<K, T>, 
                                L1Distance<C>, MaxDivergence<T>>>
    where K: 'static + Eq + Hash,
          C: 'static + Integer + DistanceCast,
          T: 'static + Float + DistanceCast {
    
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
use std::collections::HashSet;
use std::hash::Hash;

use rand::Rng;

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{MaxDivergence, SymmetricDistance, IntDistance};
use crate::dom::AllDomain;
use crate::error::Fallible;
use crate::samplers::SampleBernoulli;
use crate::traits::{ExactIntCast, CheckNull};
use num::Float;

// useful paper: http://csce.uark.edu/~xintaowu/publ/DPL-2014-003.pdf
// p is probability that output is correct
// most tutorials describe two balanced coin flips, so p = .75, giving eps = ln(.75 / .25) = ln(3)

pub trait Half { fn half() -> Self; }
impl Half for f64 { fn half() -> Self {0.5} }
impl Half for f32 { fn half() -> Self {0.5} }

pub fn make_randomized_response_bool<Q>(
    prob: Q, constant_time: bool
) -> Fallible<Measurement<AllDomain<bool>, AllDomain<bool>, SymmetricDistance, MaxDivergence<Q>>>
    where bool: SampleBernoulli<Q>,
          Q: 'static + Float + ExactIntCast<IntDistance> + Half {

    if !(Q::half()..Q::one()).contains(&prob) {
        return fallible!(MakeTransformation, "probability must be within [0.5, 1)")
    }

    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &bool| {
            let lie = bool::sample_bernoulli(prob, constant_time)?;
            Ok(if bool::sample_bernoulli(prob, constant_time)? { *arg } else { lie })
        }),
        SymmetricDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_fallible(move |d_in: &IntDistance, d_out: &Q|
            Ok(*d_out >= Q::exact_int_cast(*d_in)? * (prob / (Q::one() - prob)).ln())),
    ))
}

pub fn make_randomized_response<T, Q>(
    categories: HashSet<T>, prob: Q, constant_time: bool
) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, SymmetricDistance, MaxDivergence<Q>>>
    where T: 'static + Clone + Eq + Hash + CheckNull,
          bool: SampleBernoulli<Q>,
          Q: 'static + Float + ExactIntCast<IntDistance> + Half {

    let categories = categories.into_iter().collect::<Vec<_>>();
    if categories.is_empty() {
        return fallible!(MakeTransformation, "must have at least one category")
    }

    if !(Q::half()..Q::one()).contains(&prob) {
        return fallible!(MakeTransformation, "probability must be within [0.5, 1)")
    }

    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |truth: &T| {

            // TODO: implement openssl CoreRng generator, implement a constant-time int sampler
            let mut rng = rand::thread_rng();

            // find index of truth in category set, or None
            let index = categories.iter().position(|cat| cat == truth);

            // randomly sample a lie from among the categories with equal probability
            // if truth in categories, sample among n - 1 categories
            let mut sample = rng.gen_range(0, categories.len() - if index.is_some() { 1 } else { 0 });
            // shift the sample by one if index is greater or equal to the index of truth
            if let Some(i) = index { if sample >= i { sample += 1 } }
            let lie = &categories[sample];

            // return the truth if we chose to be honest and the truth is in the category set
            let be_honest = bool::sample_bernoulli(prob, constant_time)?;
            let is_member = index.is_some();
            Ok(if be_honest && is_member { truth } else { lie }.clone())
        }),
        SymmetricDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_fallible(move |d_in: &IntDistance, d_out: &Q|
            Ok(*d_out >= Q::exact_int_cast(*d_in)? * (prob / (Q::one() - prob)).ln())),
    ))
}
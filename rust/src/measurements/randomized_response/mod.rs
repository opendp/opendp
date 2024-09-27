#[cfg(feature = "ffi")]
mod ffi;

use std::collections::HashSet;

use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::AtomDomain;
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::DiscreteDistance;
use crate::traits::samplers::sample_bernoulli_float;
use crate::traits::{ExactIntCast, Hashable, InfDiv, InfLn, InfMul, InfSub};

// There are two constructors:
// 1. make_randomized_response_bool
//    a simple implementation specifically for booleans
// 2. make_randomized_response
//    for any categorical type with t > 1 categories
//
// The general rule is eps = (p / p').ln(), where p' = (1 - p) / (t - 1), and t = # categories
// See paper for more details: http://csce.uark.edu/~xintaowu/publ/DPL-2014-003.pdf
//
// In the case of privatizing a balanced coin flip,
//     t = 2, p = .75, giving eps = ln(.75 / .25) = ln(3)

#[bootstrap(features("contrib"), arguments(constant_time(default = false)))]
/// Make a Measurement that implements randomized response on a boolean value.
///
/// # Arguments
/// * `prob` - Probability of returning the correct answer. Must be in `[0.5, 1)`
/// * `constant_time` - Set to true to enable constant time. Slower.
pub fn make_randomized_response_bool(
    prob: f64,
    constant_time: bool,
) -> Fallible<Measurement<AtomDomain<bool>, bool, DiscreteDistance, MaxDivergence>> {
    // number of categories t is 2, and probability is bounded below by 1/t
    if !((0.5)..(1.0)).contains(&prob) {
        return fallible!(MakeMeasurement, "probability must be within [0.5, 1)");
    }

    // d_out = min(d_in, 1) * ln(p / p')
    //             where p' = 1 - p
    //       = min(d_in, 1) * ln(p / (1 - p))
    let privacy_constant = prob.inf_div(&(1.0).neg_inf_sub(&prob)?)?.inf_ln()?;

    Measurement::new(
        AtomDomain::default(),
        Function::new_fallible(move |arg: &bool| {
            Ok(arg ^ !sample_bernoulli_float(prob, constant_time)?)
        }),
        DiscreteDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new(move |d_in| if *d_in == 0 { 0.0 } else { privacy_constant }),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(categories(rust_type = "Vec<T>"), constant_time(default = false)),
    generics(T(example = "$get_first(categories)"))
)]
/// Make a Measurement that implements randomized response on a categorical value.
///
/// # Arguments
/// * `categories` - Set of valid outcomes
/// * `prob` - Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
///
/// # Generics
/// * `T` - Data type of a category.
pub fn make_randomized_response<T: Hashable>(
    categories: HashSet<T>,
    prob: f64,
) -> Fallible<Measurement<AtomDomain<T>, T, DiscreteDistance, MaxDivergence>> {
    use crate::traits::samplers::sample_uniform_uint_below;

    let categories = categories.into_iter().collect::<Vec<_>>();
    if categories.len() < 2 {
        return fallible!(MakeMeasurement, "length of categories must be at least two");
    }
    let num_categories = f64::exact_int_cast(categories.len())?;

    if !(num_categories.recip()..(1.0)).contains(&prob) {
        return fallible!(
            MakeMeasurement,
            "probability must be within [1/num_categories, 1)"
        );
    }

    // d_out = min(d_in, 1) * (p / p').ln()
    //              where p' = the probability of categories off the diagonal
    //                       = (1 - p) / (t - 1)
    //              where t  = num_categories
    //       = min(d_in, 1) * (p / (1 - p) * (t - 1)).ln()
    let privacy_constant = prob
        .inf_div(&(1.0).neg_inf_sub(&prob)?)?
        .inf_mul(&num_categories.inf_sub(&1.0)?)?
        .inf_ln()?;

    Measurement::new(
        AtomDomain::default(),
        Function::new_fallible(move |truth: &T| {
            // find index of truth in category set, or None
            let index = categories.iter().position(|cat| cat == truth);

            // randomly sample a lie from among the categories with equal probability
            // if truth in categories, sample among n - 1 categories
            let mut sample =
                sample_uniform_uint_below(categories.len() - if index.is_some() { 1 } else { 0 })?;
            // shift the sample by one if index is greater or equal to the index of truth
            if let Some(i) = index {
                if sample >= i {
                    sample += 1
                }
            }
            let lie = &categories[sample];

            // return the truth if we chose to be honest and the truth is in the category set
            let be_honest = sample_bernoulli_float(prob, false)?;
            let is_member = index.is_some();
            Ok(if be_honest && is_member { truth } else { lie }.clone())
        }),
        DiscreteDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new(move |d_in| if *d_in == 0 { 0.0 } else { privacy_constant }),
    )
}

#[cfg(test)]
mod test;

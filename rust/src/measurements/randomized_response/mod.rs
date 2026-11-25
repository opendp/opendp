#[cfg(feature = "ffi")]
mod ffi;

use std::collections::HashSet;

use opendp_derive::bootstrap;

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::AtomDomain;
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::DiscreteDistance;
use crate::traits::samplers::{
    bernoulli_u64_probability, bernoulli_u64_threshold, sample_bernoulli_u64,
};
use crate::traits::{ExactIntCast, Hashable, InfDiv, InfLn, InfLn1P, InfMul, InfSub};

#[bootstrap(features("contrib"))]
/// Make a Measurement that implements randomized response on a boolean value.
///
/// # Citations
/// * [Warner65 Randomized Response: A Survey Technique for Eliminating Evasive Answer Bias](https://doi.org/10.1080/01621459.1965.10480775)
///
/// # Runtime
/// Per release, runtime is `O(1)`.
///
/// # Utility
/// The released bit matches the input with probability `prob`,
/// so the expected 0-1 loss is `O(1 - prob)`.
///
/// # Arguments
/// * `prob` - Probability of returning the correct answer. Must be in `[0.5, 1]`
pub fn make_randomized_response_bool(
    prob: f64,
) -> Fallible<Measurement<AtomDomain<bool>, DiscreteDistance, MaxDivergence, bool>> {
    if !(0.5f64..=1.0).contains(&prob) {
        return fallible!(MakeMeasurement, "probability must be within [0.5, 1]");
    }

    let threshold = bernoulli_u64_threshold(prob);

    let privacy_constant = if prob == 1.0 {
        f64::INFINITY
    } else {
        prob.inf_div(&(1.0).neg_inf_sub(&prob)?)?.inf_ln()?
    };

    Measurement::new(
        AtomDomain::default(),
        DiscreteDistance,
        MaxDivergence,
        Function::new_fallible(move |arg: &bool| Ok(arg ^ !sample_bernoulli_u64(threshold)?)),
        PrivacyMap::new(move |d_in| if *d_in == 0 { 0.0 } else { privacy_constant }),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(categories(rust_type = "Vec<T>")),
    generics(T(example = "$get_first(categories)"))
)]
/// Make a Measurement that implements randomized response on a categorical value.
///
/// # Citations
/// * [Warner65 Randomized Response: A Survey Technique for Eliminating Evasive Answer Bias](https://doi.org/10.1080/01621459.1965.10480775)
///
/// # Runtime
/// Per release, runtime is `O(t)`, where `t` is the number of categories,
/// because the implementation linearly searches for the input in `categories`.
///
/// # Utility
/// If the input is a member of `categories`,
/// the mechanism returns the truthful category with approximately probability `prob`,
/// so the expected 0-1 loss is `O(1 - prob)`.
/// Otherwise, the output is sampled uniformly from `categories`.
///
/// # Arguments
/// * `categories` - Set of valid outcomes
/// * `prob` - Probability of returning the correct answer. Must be in `[1/num_categories, 1]`
///
/// # Generics
/// * `T` - Data type of a category.
pub fn make_randomized_response<T: Hashable>(
    categories: HashSet<T>,
    prob: f64,
) -> Fallible<Measurement<AtomDomain<T>, DiscreteDistance, MaxDivergence, T>> {
    use crate::traits::samplers::sample_uniform_uint_below;

    let categories = categories.into_iter().collect::<Vec<_>>();
    if categories.len() < 2 {
        return fallible!(MakeMeasurement, "length of categories must be at least two");
    }
    let num_categories = f64::exact_int_cast(categories.len())?;

    if !(num_categories.recip()..=1f64).contains(&prob) {
        return fallible!(
            MakeMeasurement,
            "probability must be within [1/num_categories, 1]"
        );
    }

    // Mix truthful reporting with a uniform draw from all categories.
    // Round the uniform-mixture probability up to preserve the requested privacy bound.
    let randomize_prob = if prob == num_categories.recip() {
        1.0
    } else {
        (1.0)
            .inf_sub(&prob)?
            .inf_mul(&num_categories)?
            .inf_div(&(num_categories - 1.0))?
            .min(1.0)
    };
    let threshold = bernoulli_u64_threshold(randomize_prob);

    // Bound privacy loss using the actual mixture probability.
    let q = bernoulli_u64_probability(threshold);
    let privacy_constant = if q == 0.0 {
        f64::INFINITY
    } else {
        1.0f64
            .inf_sub(&q)?
            .inf_mul(&num_categories)?
            .inf_div(&q)?
            .inf_ln_1p()?
    };

    Measurement::new(
        AtomDomain::default(),
        DiscreteDistance,
        MaxDivergence,
        Function::new_fallible(move |truth: &T| {
            let sample = sample_uniform_uint_below(categories.len())?;
            let randomize = sample_bernoulli_u64(threshold)?;
            Ok(if randomize || !categories.contains(truth) {
                &categories[sample]
            } else {
                truth
            }
            .clone())
        }),
        PrivacyMap::new(move |d_in| if *d_in == 0 { 0.0 } else { privacy_constant }),
    )
}

#[cfg(test)]
mod test;

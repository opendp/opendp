#[cfg(feature = "ffi")]
mod ffi;

mod kernel;
mod operator;
mod constructor;

use std::collections::HashSet;

use opendp_derive::bootstrap;

use crate::core::{CertifiedMeasurementParts, Function, Measurement, PrivacyMap};
use crate::domains::AtomDomain;
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::DiscreteDistance;
use crate::traits::samplers::sample_bernoulli_float;
use crate::traits::{ExactIntCast, Hashable, InfDiv, InfLn, InfMul, InfSub};

// This is the only trusted callback-erasure adapter for Boolean RR.
// It forwards to the concrete operator; Verus does not verify Arc<dyn Fn>.
struct BoolRRParts(operator::BoolRR);

impl CertifiedMeasurementParts<AtomDomain<bool>, DiscreteDistance, MaxDivergence, bool>
    for BoolRRParts
{
    fn into_parts(
        self,
        _input_domain: &AtomDomain<bool>,
    ) -> Fallible<(Function<bool, bool>, PrivacyMap<DiscreteDistance, MaxDivergence>)> {
        let operator = self.0;
        let map_operator = operator;
        Ok((
            Function::new_fallible(move |truth: &bool| operator.invoke(*truth)),
            PrivacyMap::new(move |distance: &u32| map_operator.map(*distance, 0.0)),
        ))
    }
}

// There are two constructors:
// 1. make_randomized_response_bool
//    a simple implementation specifically for booleans
// 2. make_randomized_response
//    for any categorical type with t > 1 categories
//
// The general rule is eps = (p / p').ln(), where p' = (1 - p) / (t - 1), and t = # categories
// See paper for more details: http://csce.uark.edu/~xintaowu/publ/DPL-2014-003.pdf
//
// In the case of a balanced coin flip,
//     t = 2, p = .75, giving eps = ln(.75 / .25) = ln(3)

#[bootstrap(features("contrib"), arguments(constant_time(default = false)))]
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
/// * `constant_time` - Both values use one fixed-size uniform-byte draw.
///
/// # Verification
/// Verus checks the operator builder and conditional privacy guarantee.
/// Entropy, numerical primitives, and callback erasure are trusted; see
/// rust/opendp_verus/README.md.
pub fn make_randomized_response_bool(
    prob: f64,
    constant_time: bool,
) -> Fallible<Measurement<AtomDomain<bool>, DiscreteDistance, MaxDivergence, bool>> {
    // Parameter validation, exact probability decoding, log-odds
    // arithmetic and operator assembly are the shared Verus-checked path.
    let Some(operator) = constructor::build_bool_rr(prob.to_bits())? else {
        return fallible!(MakeMeasurement, "probability must be within [0.5, 1]");
    };
    let _ = constant_time; // Both settings use one fixed-size entropy draw.

    Measurement::new_certified(
        AtomDomain::default(),
        DiscreteDistance,
        MaxDivergence,
        BoolRRParts(operator),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(categories(rust_type = "Vec<T>"), constant_time(default = false)),
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
/// the mechanism returns the truthful category with probability `prob`,
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

    // d_out = min(d_in, 1) * (p / p').ln()
    //              where p' = the probability of categories off the diagonal
    //                       = (1 - p) / (t - 1)
    //              where t  = num_categories
    //       = min(d_in, 1) * (p / (1 - p) * (t - 1)).ln()

    let privacy_constant = if prob == 1.0 {
        f64::INFINITY
    } else {
        prob.inf_div(&(1.0).neg_inf_sub(&prob)?)?
            .inf_mul(&num_categories.inf_sub(&1.0)?)?
            .inf_ln()?
    };

    Measurement::new(
        AtomDomain::default(),
        DiscreteDistance,
        MaxDivergence,
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
        PrivacyMap::new(move |d_in| if *d_in == 0 { 0.0 } else { privacy_constant }),
    )
}

#[cfg(test)]
mod test;

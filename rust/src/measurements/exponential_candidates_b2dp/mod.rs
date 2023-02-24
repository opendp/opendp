//! Implements the base-2 exponential mechanism.

use crate::core::{Measurement, PrivacyMap, Function};
use crate::domains::{SizedDomain, VectorDomain, AllDomain};
use crate::error::Fallible;
use crate::measurements::b2dp::utilities::exactarithmetic::{
    normalized_sample, ArithmeticConfig,
};
use crate::measurements::b2dp::utilities::params::Eta;
use crate::measures::MaxDivergence;
use crate::metrics::InfDifferenceDistance;
use crate::traits::samplers::GeneratorOpenDP;
use crate::traits::{CheckNull, Float};


pub fn make_base_exponential_candidates_b2dp<T, F>(
    entropy: F,
    size: usize,
    max_utility: usize,
    max_outcomes: usize,
    min_retries: Option<u32>,
) -> Fallible<Measurement<
    SizedDomain<VectorDomain<AllDomain<T>>>,
    AllDomain<usize>,
    InfDifferenceDistance<T>,
    MaxDivergence<F>
>> where T: CheckNull, F: Float {
    let eta = Eta::from_epsilon(entropy.recip())?;
    // Check Parameters
    eta.check()?;
    if max_outcomes < size {
        return fallible!(FailedFunction, "Number of outcomes exceeds max_outcomes.");
    }

    if max_outcomes == 0 {
        return fallible!(FailedFunction, "Must provide a positive value for max_outcomes.");
    }

    Ok(Measurement::new(
        SizedDomain::new(VectorDomain::new(AllDomain::new()), size),
        AllDomain::new(),
        Function::new_fallible(move |utilities| {

            let arith_config = ArithmeticConfig::for_exponential(&eta, max_utility, max_outcomes, min_retries.unwrap_or(1))?;

            // get the base
            let base = eta.get_base(arith_config.precision).unwrap();

            // Enter exact scope
            arith_config.enter_exact_scope()?;
            
            // Generate weights vector
            let mut weights = Vec::new();
            for u in utilities.iter() {
                let w = arith_config.get_float(base.pow(u));
                weights.push(w);
            }

            // Sample
            let sample_index = normalized_sample(
                &weights,
                &arith_config,
                &mut GeneratorOpenDP::default(),
            )?;

            // Exit exact scope
            arith_config.exit_exact_scope()?;

            Ok(sample_index)
        }),
        InfDifferenceDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(|d_in| d_in.inf_div(entropy.recip()))
    ))
}

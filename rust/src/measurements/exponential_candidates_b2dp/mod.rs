//! Implements the base-2 exponential mechanism.

use num::traits::Pow;

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{AllDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::measurements::b2dp::utilities::exactarithmetic::{normalized_sample, ArithmeticConfig};
use crate::measurements::b2dp::utilities::params::Eta;
use crate::measures::MaxDivergence;
use crate::metrics::InfDifferenceDistance;
use crate::traits::samplers::GeneratorOpenDP;
use crate::traits::{ExactIntCast, Float, InfCast, Integer, RoundCast};

pub fn make_base_exponential_candidates_b2dp<T, F>(
    entropy: F,
    size: usize,
    max_utility: T,
    max_outcomes: usize,
    min_retries: Option<u32>,
) -> Fallible<
    Measurement<
        SizedDomain<VectorDomain<AllDomain<T>>>,
        Vec<usize>,
        InfDifferenceDistance<T>,
        MaxDivergence<F>,
    >,
>
where
    T: Integer,
    F: Float + InfCast<T>,
    f64: RoundCast<F>,
    u32: ExactIntCast<T>,
{
    let eta = Eta::from_epsilon(f64::round_cast(entropy.recip())?)?;
    // Check Parameters
    eta.check()?;
    if max_outcomes < size {
        return fallible!(FailedFunction, "Number of outcomes exceeds max_outcomes.");
    }

    if max_outcomes == 0 {
        return fallible!(
            FailedFunction,
            "Must provide a positive value for max_outcomes."
        );
    }

    Ok(Measurement::new(
        SizedDomain::new(VectorDomain::new(AllDomain::new()), size),
        Function::new_fallible(move |utilities: &Vec<T>| {
            let mut arith_config = ArithmeticConfig::for_exponential(
                &eta,
                u32::exact_int_cast(max_utility)?,
                max_outcomes as u32,
                min_retries.unwrap_or(1),
            )?;

            // get the base
            let base = eta.get_base(arith_config.precision).unwrap();

            // Enter exact scope
            arith_config.enter_exact_scope()?;

            // Generate weights vector
            let mut weights = Vec::new();
            for u in utilities.iter() {
                let w = arith_config.get_float(base.clone().pow(u32::exact_int_cast(u.clone())?));
                weights.push(w);
            }

            // Sample
            let sample_index =
                normalized_sample(&weights, &arith_config, &mut GeneratorOpenDP::default())?;

            // Exit exact scope
            arith_config.exit_exact_scope()?;

            Ok(vec![sample_index])
        }),
        InfDifferenceDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &T| {
            F::inf_cast(d_in.clone())?.inf_div(&entropy.recip())
        }),
    ))
}

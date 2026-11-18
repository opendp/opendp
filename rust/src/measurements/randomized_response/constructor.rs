//! Shared production/Verus assembly of the Boolean randomized-response
//! operator. The outer OpenDP constructor only performs the audited erasure.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::{dyadic53, operator, verified_arithmetic};
#[cfg(not(verus_only))]
use super::operator;
#[cfg(not(verus_only))]
use crate::traits::samplers::dyadic53;

#[cfg(verus_only)]
use crate::verified_arithmetic::{upper_log_odds_u53, positive_infinity_f64, ArithmeticError};
#[cfg(not(verus_only))]
use crate::traits::{upper_log_odds_u53, positive_infinity_f64};

#[cfg(not(verus_only))]
type ArithmeticError = crate::error::Error;

/// Validate the input IEEE-754 bits, compute a conservatively rounded
/// privacy cost, and assemble the *actual production* BoolRR value.
///
/// Ok(None) means an invalid probability. Err propagates a numerical
/// failure. When the constructor succeeds, Verus establishes the
/// factor bound directly from the executable numerical helper.
#[cfg_attr(verus_only, verus_spec(result =>
    ensures match result {
        Ok(Some(op)) => match dyadic53::checked_numerator_from_bits(bits) {
            None => false,
            Some(n) => {
                &&& op.numerator == n
                &&& n < 9_007_199_254_740_992u64 ==> {
                    &&& verified_arithmetic::real_exp(
                        verified_arithmetic::float_value(op.privacy_cost)) >= 1real
                    &&& n as real <= verified_arithmetic::real_exp(
                        verified_arithmetic::float_value(op.privacy_cost))
                            * (9_007_199_254_740_992u64 - n) as real
                }
            },
        },
        Ok(None) => dyadic53::checked_numerator_from_bits(bits).is_none(),
        Err(_) => dyadic53::checked_numerator_from_bits(bits).is_some(),
    },
))]
pub fn build_bool_rr(bits: u64) -> Result<Option<operator::BoolRR<f64>>, ArithmeticError> {
    let Some(numerator) = dyadic53::checked_numerator_from_bits(bits) else {
        return Ok(None);
    };
    let privacy_cost = match dyadic53::odds_parts(numerator) {
        None => positive_infinity_f64(),
        Some((n, complement)) => upper_log_odds_u53(n, complement)?,
    };
    Ok(Some(operator::BoolRR { numerator, privacy_cost }))
}

/// The single trusted adapter obligation, parameterized by the *actual*
/// concrete operator returned by build_bool_rr. There is no second RR model.
/// This corresponds to CertifiedBoolRR::into_parts in production.
#[cfg(verus_only)]
verus! {
pub open spec fn adapter_realizes<E>(
    op: operator::BoolRR<f64>,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    callback_expectation: spec_fn(
        operator::BoolRR<f64>, bool, spec_fn(Result<bool, E>) -> real,
    ) -> real,
    callback_map: spec_fn(operator::BoolRR<f64>, u32) -> Option<real>,
) -> bool {
    let factor = if op.numerator == 9_007_199_254_740_992u64 {
        1real
    } else {
        verified_arithmetic::real_exp(
            verified_arithmetic::float_value(op.privacy_cost))
    };
    &&& crate::measurement_proof::realizes_expectation(
        |truth: bool, payoff: spec_fn(Result<bool, E>) -> real|
            callback_expectation(op, truth, payoff),
        |truth: bool, payoff: spec_fn(Result<bool, E>) -> real|
            crate::randomized_response_bool::release_expectation(
                truth, op.numerator, word_expectation, payoff),
    )
    &&& crate::measurement_proof::realizes_privacy_map(
        |d: u32| callback_map(op, d),
        |d: u32| crate::randomized_response_bool::rr_factor_map(
            op.numerator, factor, d),
    )
}

/// Constructor-level conditional theorem. This *executes the same builder*
/// called by make_randomized_response_bool before proving the privacy
/// guarantee of its certified erasure, rather than accepting a separately
/// supplied numerator or epsilon.
///
/// Remaining preconditions are only the probabilistic primitive and the
/// native Arc adapter's correspondence, both explicit trust boundaries.
pub fn verify_built_bool_rr<E>(
    bits: u64,
    Ghost(word_expectation): Ghost<
        spec_fn(spec_fn(Result<u64, E>) -> real) -> real>,
    Ghost(error_expectation): Ghost<
        spec_fn(spec_fn(E) -> real) -> real>,
    Ghost(success): Ghost<real>,
    Ghost(callback_expectation): Ghost<
        spec_fn(operator::BoolRR<f64>, bool,
            spec_fn(Result<bool, E>) -> real) -> real>,
    Ghost(callback_map): Ghost<
        spec_fn(operator::BoolRR<f64>, u32) -> Option<real>>,
) -> (result: Result<Option<operator::BoolRR<f64>>, ArithmeticError>)
    requires
        crate::bernoulli_proof::uniform_word_expectation_rule(
            word_expectation, error_expectation, success),
        forall|op: operator::BoolRR<f64>|
            #[trigger] adapter_realizes(
                op, word_expectation, callback_expectation, callback_map),
    ensures
        match result {
            Ok(Some(op)) =>
                crate::measurement_proof::valid_measurement_expectations::<
                    bool, Result<bool, E>, u32>(
                    |_: bool| true,
                    |left: bool, right: bool, d: u32|
                        crate::randomized_response_bool::rr_discrete_close(
                            left, right, d),
                    |truth: bool, payoff: spec_fn(Result<bool, E>) -> real|
                        callback_expectation(op, truth, payoff),
                    |d: u32| callback_map(op, d),
                ),
            _ => true,
        },
{
    let result = build_bool_rr(bits);
    if let Ok(Some(op)) = result {
        proof {
            let n = op.numerator;
            let callback = |truth: bool, payoff: spec_fn(Result<bool, E>) -> real|
                callback_expectation(op, truth, payoff);
            let map = |d: u32| callback_map(op, d);
            assert(adapter_realizes(
                op, word_expectation, callback_expectation, callback_map));
            assert(4_503_599_627_370_496u64 <= n
                <= 9_007_199_254_740_992u64);
            if n == 9_007_199_254_740_992u64 {
                crate::randomized_response_bool::lemma_bool_rr_new_certified(
                    n, word_expectation, error_expectation, success,
                    1real, callback, map);
            } else {
                let c: u64 = (9_007_199_254_740_992u64 - n) as u64;
                assert(c == 9_007_199_254_740_992u64 - n);
                assert(verified_arithmetic::real_exp(
                    verified_arithmetic::float_value(op.privacy_cost)) >= 1real);
                assert(n as real <= verified_arithmetic::real_exp(
                    verified_arithmetic::float_value(op.privacy_cost)) * c as real);
                crate::randomized_response_bool::lemma_certified_rr_with_computed_epsilon(
                    n, c, op.privacy_cost, word_expectation, error_expectation,
                    success, callback, map);
            }
        }
    }
    result
}

} // verus!

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
use crate::verified_arithmetic::{
    ArithmeticError, exact_u53_cast, inf_div_up_f64, inf_ln_up_f64,
    positive_infinity_f64, float_value, finite, real_exp, real_exp_monotonic,
    real_exp_ln_inverse, real_ln,
};
#[cfg(not(verus_only))]
use crate::traits::verified::{
    ArithmeticError, exact_u53_cast, inf_div_up_f64, inf_ln_up_f64,
    positive_infinity_f64,
};

/// Production floating-point log odds, with exactly the same division and
/// logarithm used elsewhere in OpenDP. The Verus harness checks this body
/// using the trusted primitive contracts above.
// The outer function is executable, not a dual spec function: its proof
// consumes executable contracts for the trusted primitives.
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        4_503_599_627_370_496u64 <= numerator,
        numerator < 9_007_199_254_740_992u64,
        complement == 9_007_199_254_740_992u64 - numerator,
    ensures match result {
        Ok(epsilon) => {
            &&& finite(epsilon)
            &&& real_exp(float_value(epsilon)) >= 1real
            &&& (numerator as real) <= real_exp(float_value(epsilon)) * (complement as real)
        },
        Err(_) => true,
    },
))]
pub fn upper_log_odds_u53(
    numerator: u64,
    complement: u64,
) -> Result<f64, ArithmeticError> {
    let n = exact_u53_cast(numerator);
    let c = exact_u53_cast(complement);
    let quotient = inf_div_up_f64(n, c)?;
    #[cfg(verus_only)]
    proof! {
        let a = numerator as real;
        let b = complement as real;
        let q = float_value(quotient);
        assert(b > 0real);
        assert(a > 0real);
        assert(q >= a / b);
        assert(q > 0real) by(nonlinear_arith)
            requires a > 0real, b > 0real, q >= a / b;
    }
    let epsilon = inf_ln_up_f64(quotient)?;
    #[cfg(verus_only)]
    proof! {
        let a = numerator as real;
        let b = complement as real;
        let q = float_value(quotient);
        let e = float_value(epsilon);
        assert(b > 0real);
        assert(a > 0real);
        assert(q >= a / b);
        assert(q > 0real) by(nonlinear_arith) requires a > 0real, b > 0real, q >= a / b;
        real_exp_monotonic(real_ln(q), e);
        real_exp_ln_inverse(q);
        assert(real_exp(e) >= q);
        assert(real_exp(e) * b >= a) by(nonlinear_arith)
            requires b > 0real, q >= a / b, real_exp(e) >= q;
        assert(real_exp(e) >= 1real) by(nonlinear_arith)
            requires a >= b, b > 0real, real_exp(e) * b >= a;
    }
    Ok(epsilon)
}

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
                &&& crate::bernoulli_proof::accepted_binary64_value(bits)
                    == n as real / 9_007_199_254_740_992real
                &&& n == 9_007_199_254_740_992u64 ==>
                    f64_to_bits(op.privacy_cost) == 0x7ff0_0000_0000_0000u64
                &&& n < 9_007_199_254_740_992u64 ==> {
                    &&& verified_arithmetic::finite(op.privacy_cost)
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
pub fn build_bool_rr(bits: u64) -> Result<Option<operator::BoolRR>, ArithmeticError> {
    let Some(numerator) = dyadic53::checked_numerator_from_bits(bits) else {
        return Ok(None);
    };
    #[cfg(verus_only)]
    proof! {
        crate::bernoulli_proof::lemma_checked_binary64_probability(bits);
    }
    let privacy_cost = match dyadic53::odds_parts(numerator) {
        None => positive_infinity_f64(),
        Some((n, complement)) => upper_log_odds_u53(n, complement)?,
    };
    Ok(Some(operator::BoolRR { numerator, privacy_cost }))
}

/// Interpret the actual numeric privacy map of the production operator.
/// This specification obtains its f64 value from BoolRR::map itself, rather
/// than accepting a caller-supplied callback or abstract factor.
#[cfg(verus_only)]
verus! {
pub open spec fn operator_factor_map(
    op: operator::BoolRR,
    distance: u32,
) -> Option<real> {
    if distance == 0 { Some(1real) }
    else if f64_to_bits(op.map(distance, 0.0f64)) == 0x7ff0_0000_0000_0000u64 {
        None
    } else {
        Some(verified_arithmetic::real_exp(
            verified_arithmetic::float_value(op.map(distance, 0.0f64))))
    }
}

/// Build the *production* BoolRR and prove that its actual release method
/// and actual privacy map satisfy the MaxDivergence contract. Unlike the
/// former callback theorem, this result has no adapter-correspondence
/// hypothesis. Entropy and the separately documented numerical primitives
/// remain trusted. Closure erasure is NOT covered by this theorem.
pub fn verify_built_bool_rr<E>(
    bits: u64,
    Ghost(word_expectation): Ghost<
        spec_fn(spec_fn(Result<u64, E>) -> real) -> real>,
) -> (result: Result<Option<operator::BoolRR>, ArithmeticError>)
    requires
        crate::bernoulli_proof::uniform_word_expectation_rule(word_expectation),
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
                        crate::randomized_response_bool::release_expectation(
                            truth, op, word_expectation, payoff),
                    |d: u32| operator_factor_map(op, d),
                ),
            _ => true,
        },
{
    let result = build_bool_rr(bits);
    if let Ok(Some(op)) = result {
        proof {
            let n = op.numerator;
            assert(4_503_599_627_370_496u64 <= n
                <= 9_007_199_254_740_992u64);
            let factor = if n == 9_007_199_254_740_992u64 {
                1real
            } else {
                verified_arithmetic::real_exp(
                    verified_arithmetic::float_value(op.privacy_cost))
            };
            if n < 9_007_199_254_740_992u64 {
                let c: u64 = 9_007_199_254_740_992u64 - n;
                assert(factor >= 1real);
                assert(n as real <= factor * c as real);
            }
            let member = |_: bool| true;
            let close = |left: bool, right: bool, d: u32|
                crate::randomized_response_bool::rr_discrete_close(left, right, d);
            let expectation = |truth: bool, payoff: spec_fn(Result<bool, E>) -> real|
                crate::randomized_response_bool::release_expectation(
                    truth, op, word_expectation, payoff);
            let model_map = |d: u32|
                crate::randomized_response_bool::rr_factor_map(n, factor, d);
            let actual_map = |d: u32| operator_factor_map(op, d);
            crate::randomized_response_bool::lemma_rr_operator_valid_measurement(
                op, word_expectation, factor);

            assert forall|d: u32| #[trigger] actual_map(d) == model_map(d) by {
                if d == 0 {
                    assert(actual_map(d) == Some(1real));
                    assert(model_map(d) == Some(1real));
                } else if n == 9_007_199_254_740_992u64 {
                    assert(f64_to_bits(op.privacy_cost) == 0x7ff0_0000_0000_0000u64);
                    assert(f64_to_bits(op.map(d, 0.0f64))
                        == 0x7ff0_0000_0000_0000u64);
                } else {
                    assert(verified_arithmetic::finite(op.privacy_cost));
                    let cost_bits = f64_to_bits(op.privacy_cost);
                    assert(((cost_bits >> 52u64) & 2047u64) != 2047u64);
                    verified_arithmetic::lemma_finite_bits_not_positive_infinity(cost_bits);
                    assert(f64_to_bits(op.map(d, 0.0f64)) == cost_bits);
                    assert(f64_to_bits(op.map(d, 0.0f64))
                        != 0x7ff0_0000_0000_0000u64);
                    verified_arithmetic::lemma_same_bits_same_value(
                        op.map(d, 0.0f64), op.privacy_cost);
                }
            }
            assert(crate::measurement_proof::realizes_privacy_map(actual_map, model_map));
            assert(crate::measurement_proof::realizes_expectation(
                expectation, expectation));
            crate::measurement_proof::lemma_valid_measurement_from_equivalent_components(
                member, close, expectation, model_map, expectation, actual_map);
        }
    }
    result
}

} // verus!

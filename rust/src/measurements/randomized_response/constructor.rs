//! Production Boolean randomized-response construction, imported directly by
//! Verus. The actual Measurement::new_certified call is verified here.
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

#[cfg(verus_only)]
use crate::{Domain, Measure, Measurement, Metric, MetricSpace};
#[cfg(not(verus_only))]
use crate::core::{Domain, Measure, Measurement, Metric, MetricSpace};

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

/// Build the actual OpenDP Measurement, including the real certified
/// constructor call. The public API supplies AtomDomain<bool>,
/// DiscreteDistance, and MaxDivergence; this shared function performs the
/// same executable work in the native and Verus builds.
///
/// The whole privacy-validity precondition on new_certified is still to be
/// discharged; importing this executable call makes that obligation visible.
#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures match result {
        Ok(Some(measurement)) => {
            &&& measurement.input_domain == input_domain
            &&& measurement.input_metric == input_metric
            &&& measurement.output_measure == output_measure
            &&& crate::measurement_proof::valid_bool_rr_measurement(&measurement)
            &&& forall|truth: &bool|
                #[trigger] crate::measurement_parts::function_call_requires(
                    &measurement.function, truth)
            &&& forall|d: &u32|
                #[trigger] crate::measurement_parts::privacy_map_call_requires(
                    &measurement.privacy_map, d)
            &&& (dyadic53::checked_numerator_from_bits(bits)
                    == Some(9_007_199_254_740_992u64) ==>
                forall|truth: &bool, value: Result<bool, ()>|
                    crate::measurement_parts::function_call_ensures(
                        &measurement.function, truth, value) && value.is_ok()
                    ==> value == Ok(*truth))
        },
        _ => true,
    },
))]
pub(crate) fn build_bool_rr_measurement<DI, MI, MO>(
    bits: u64,
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Result<Option<Measurement<DI, MI, MO, bool>>, ArithmeticError>
where
    DI: Domain<Carrier = bool>,
    MI: Metric<Distance = u32>,
    MO: Measure<Distance = f64>,
    (DI, MI): MetricSpace,
{
    let Some(operator) = build_bool_rr(bits)? else {
        return Ok(None);
    };
    #[cfg(verus_only)]
    proof! {
        assert(well_formed_bool_rr(operator));
        crate::uniform_word::lemma_uniform_word_expectation_rule();
        lemma_built_bool_rr_valid(
            operator, crate::uniform_word::word_expectation());
    }
    #[cfg(verus_only)]
    proof_decl! {
        let ghost release_model =
            |truth: bool, word: Result<u64, ()>| operator.release(truth, word);
    }
    #[cfg(verus_only)]
    proof! {
        // The deterministic postprocessor has a unique return value for
        // every input/entropy pair, including every observable error.
        assert forall|truth: &bool, word: Result<u64, ()>| {
            &&& call_requires(operator::BoolRR::release_callback, (&operator, truth, word))
            &&& forall|out: Result<bool, ()>|
                call_ensures(operator::BoolRR::release_callback,
                    (&operator, truth, word), out)
                    ==> out == operator.release(*truth, word)
        } by {
            assert(call_requires(operator::BoolRR::release_callback,
                (&operator, truth, word)));
        }
        assert(crate::measurement_parts::exact_word_postprocessing(
            operator, operator::BoolRR::release_callback, release_model));
    }
    #[cfg(verus_only)]
    let measurement = {
        proof_with! { Ghost(release_model) }
        Measurement::new_certified(
            input_domain,
            input_metric,
            output_measure,
            operator,
            operator::BoolRR::release_callback,
            operator::BoolRR::map_callback,
        )
    }?;
    #[cfg(not(verus_only))]
    let measurement = Measurement::new_certified(
        input_domain,
        input_metric,
        output_measure,
        operator,
        operator::BoolRR::release_callback,
        operator::BoolRR::map_callback,
    )?;
    // The distribution of the *returned erased Function* is the same
    // as the executable RR release under the one-word entropy law.
    // This follows from the generic one-word erasure specification and the
    // checked postprocessor, not from a mechanism-specific privacy axiom.
    #[cfg(verus_only)]
    proof! {
        assert forall|truth: bool,
                       payoff: spec_fn(Result<bool, ()>) -> real|
            #[trigger] crate::measurement_parts::function_expectation(
                &measurement.function, &truth, payoff)
                == crate::randomized_response_bool::release_expectation(
                    truth, operator, crate::uniform_word::word_expectation(), payoff)
        by {
            let before = |word: Result<u64, ()>| payoff(release_model(truth, word));
            let after = |word: Result<u64, ()>| payoff(operator.release(truth, word));
            assert forall|word: Result<u64, ()>|
                #[trigger] before(word) == after(word) by {
                assert(release_model(truth, word) == operator.release(truth, word));
            }
            assert(before =~= after);
            assert(crate::measurement_parts::function_expectation(
                &measurement.function, &truth, payoff)
                == crate::uniform_word::one_word_expectation(
                    truth, release_model, payoff));
            assert(crate::uniform_word::one_word_expectation(
                truth, release_model, payoff)
                == crate::uniform_word::word_expectation()(before));
            assert(crate::randomized_response_bool::release_expectation(
                truth, operator, crate::uniform_word::word_expectation(), payoff)
                == crate::uniform_word::word_expectation()(after));
        }
    }
    #[cfg(verus_only)]
    proof! {
        assert forall|d: &u32| #[trigger]
            crate::measurement_parts::privacy_map_call_requires(
                &measurement.privacy_map, d)
        by {
            assert(crate::measurement_parts::privacy_map_call_requires(
                &measurement.privacy_map, d)
                == call_requires(operator::BoolRR::map_callback, (&operator, d)));
            assert(call_requires(operator::BoolRR::map_callback, (&operator, d)));
        }
    }
    // The *returned* Function delegates to the actual invoking method for
    // every Boolean input. A distributional statement requires the separate
    // one-word sampling semantics; call_ensures alone cannot provide one.
    #[cfg(verus_only)]
    proof! {
        assert forall|truth: &bool| #[trigger]
            crate::measurement_parts::function_call_requires(
                &measurement.function, truth)
        by {
            assert(crate::measurement_parts::function_call_requires(
                &measurement.function, truth)
                == call_requires(operator::BoolRR::invoke_callback, (&operator, truth)));
            assert(call_requires(operator::BoolRR::invoke_callback, (&operator, truth)));
        }
        assert forall|truth: &bool, value: Result<bool, ()>|
            crate::measurement_parts::function_call_ensures(
                &measurement.function, truth, value)
                && operator.numerator == 9_007_199_254_740_992u64
                && value.is_ok()
            implies value == Ok(*truth)
        by {
            let word = choose|word: Result<u64, ()>|
                call_ensures(operator::BoolRR::release_callback,
                    (&operator, truth, word), value);
            assert(call_ensures(operator::BoolRR::release_callback,
                (&operator, truth, word), value));
            assert(value == operator.release(*truth, word));
            assert(value == Ok(*truth));
        }
    }
    // This statement is about the *returned erased PrivacyMap*, not just
    // the operator's pre-erasure cost. A wrong method would fail verification.
    #[cfg(verus_only)]
    proof! {
        assert forall|d: &u32, value: Result<f64, ()>|
            crate::measurement_parts::privacy_map_call_ensures(
                &measurement.privacy_map, d, value)
                implies value == Ok(operator.map(*d, 0.0f64))
        by {
            assert(crate::measurement_parts::privacy_map_call_ensures(
                &measurement.privacy_map, d, value)
                == call_ensures(operator::BoolRR::map_callback, (&operator, d), value));
            assert(call_ensures(operator::BoolRR::map_callback, (&operator, d), value)
                ==> value == Ok(operator.map(*d, 0.0f64)));
        }
    }
    // Transfer the established operator privacy bound across the proved
    // distributional and map correspondences of the *returned* Measurement.
    #[cfg(verus_only)]
    proof! {
        let expectation = |truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
            crate::measurement_parts::function_expectation(
                &measurement.function, &truth, payoff);
        let model = |truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
            crate::randomized_response_bool::release_expectation(
                truth, operator, crate::uniform_word::word_expectation(), payoff);
        let member = |_: bool| true;
        let close = |left: bool, right: bool, d: u32|
            crate::randomized_response_bool::rr_discrete_close(left, right, d);
        let factor_map = |d: u32| operator_factor_map(operator, d);
        assert forall|truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
            #[trigger] expectation(truth, payoff) == model(truth, payoff) by {
            assert(expectation(truth, payoff)
                == crate::measurement_parts::function_expectation(
                    &measurement.function, &truth, payoff));
        }
        assert(crate::measurement_proof::realizes_expectation(expectation, model));
        assert(crate::measurement_proof::realizes_privacy_map(factor_map, factor_map));
        assert(crate::measurement_proof::valid_measurement_expectations(
            member, close, model, factor_map));
        crate::measurement_proof::lemma_valid_measurement_from_equivalent_components(
            member, close, model, factor_map, expectation, factor_map);

        assert forall|left: bool, right: bool, distance: u32, epsilon: f64|
            (left == right || distance > 0)
                && crate::measurement_parts::privacy_map_call_ensures(
                    &measurement.privacy_map, &distance, Ok(epsilon))
            implies {
                &&& (distance == 0 ==> f64_to_bits(epsilon) == 0u64)
                &&& match crate::measurement_proof::bool_rr_reported_factor(
                    distance, epsilon) {
                    None => true,
                    Some(factor) => factor >= 1real
                        && forall|payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::measurement_proof::bounded_payoff(payoff) ==>
                                crate::measurement_parts::function_expectation(
                                    &measurement.function, &left, payoff)
                                <= factor *
                                    crate::measurement_parts::function_expectation(
                                        &measurement.function, &right, payoff),
                }
            }
        by {
            assert(epsilon == operator.map(distance, 0.0f64));
            assert(crate::measurement_proof::bool_rr_reported_factor(distance, epsilon)
                == operator_factor_map(operator, distance));
            if distance == 0 {
                assert(left == right);
                assert(f64_to_bits(epsilon) == 0u64);
            }
            assert(close(left, right, distance));
            assert(match factor_map(distance) {
                None => true,
                Some(factor) => factor >= 1real
                    && forall|payoff: spec_fn(Result<bool, ()>) -> real|
                        crate::measurement_proof::bounded_payoff(payoff) ==>
                            expectation(left, payoff) <= factor * expectation(right, payoff),
            });
        }
        assert(crate::measurement_proof::valid_bool_rr_measurement(&measurement));
    }
    Ok(Some(measurement))
}

// Interpret the actual numeric privacy map of the production operator.
// The value comes from BoolRR::map, not a caller-supplied factor.
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

/// Conditions established by the real build_bool_rr success contract.
/// The bound concerns the precise cost stored in this concrete BoolRR value.
pub open spec fn well_formed_bool_rr(op: operator::BoolRR) -> bool {
    &&& 4_503_599_627_370_496u64 <= op.numerator
    &&& op.numerator <= 9_007_199_254_740_992u64
    &&& (op.numerator == 9_007_199_254_740_992u64 ==>
        f64_to_bits(op.privacy_cost) == 0x7ff0_0000_0000_0000u64)
    &&& (op.numerator < 9_007_199_254_740_992u64 ==> {
        &&& verified_arithmetic::finite(op.privacy_cost)
        &&& verified_arithmetic::real_exp(
            verified_arithmetic::float_value(op.privacy_cost)) >= 1real
        &&& (op.numerator as real) <=
            verified_arithmetic::real_exp(
                verified_arithmetic::float_value(op.privacy_cost)) *
            ((9_007_199_254_740_992u64 - op.numerator) as real)
    })
}

/// Actual operator theorem. This consumes the precisely built BoolRR,
/// never reconstructs or substitutes another floating-point privacy cost.
/// Distributional correctness depends only on the entropy primitive law.
pub proof fn lemma_built_bool_rr_valid<E>(
    op: operator::BoolRR,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
)
    requires
        well_formed_bool_rr(op),
        crate::bernoulli_proof::uniform_word_expectation_rule(word_expectation),
    ensures
        crate::measurement_proof::valid_measurement_expectations::<
            bool, Result<bool, E>, u32>(
            |_: bool| true,
            |left: bool, right: bool, d: u32|
                crate::randomized_response_bool::rr_discrete_close(left, right, d),
            |truth: bool, payoff: spec_fn(Result<bool, E>) -> real|
                crate::randomized_response_bool::release_expectation(
                    truth, op, word_expectation, payoff),
            |d: u32| operator_factor_map(op, d),
        ),
{
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
        let c: u64 = (9_007_199_254_740_992u64 - n) as u64;
        assert(c == 9_007_199_254_740_992u64 - n);
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

} // verus!

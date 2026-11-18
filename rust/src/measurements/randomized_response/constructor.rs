//! Production Boolean randomized-response construction, imported directly by
//! Verus. The actual Measurement::new_certified call is verified here.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::{dyadic53, operator, verified_arithmetic};
#[cfg(not(verus_only))]
use super::operator;
#[cfg(not(verus_only))]
use crate::traits::samplers::{dyadic53, OneWordProgram};
#[cfg(verus_only)]
use crate::uniform_word::OneWordProgram;

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
use crate::{Domain, CertifiedDomain, Measurement, MetricSpace};
#[cfg(not(verus_only))]
use crate::core::{Domain, CertifiedDomain, Measurement, MetricSpace};
#[cfg(verus_only)]
use crate::{discrete_distance::DiscreteDistance, max_divergence::MaxDivergence};
#[cfg(not(verus_only))]
use crate::{metrics::DiscreteDistance, measures::MaxDivergence};

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
            &&& float_value(epsilon) >= 0real
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
        assert(q >= 1real) by(nonlinear_arith)
            requires a >= b, b > 0real, q >= a / b;
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
        assert(q >= 1real) by(nonlinear_arith)
            requires a >= b, b > 0real, q >= a / b;
        assert(e >= 0real);
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
                    &&& verified_arithmetic::float_value(op.privacy_cost) >= 0real
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
/// Verus establishes privacy validity of the actual Function/PrivacyMap
/// pair before calling new_certified, which requires that validity.
#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures match result {
        Ok(Some(measurement)) => {
            &&& measurement.input_domain == input_domain
            &&& measurement.input_metric == input_metric
            &&& measurement.output_measure == output_measure
            &&& crate::measurement_proof::valid_certified_measurement(&measurement)
        },
        _ => true,
    },
))]
pub(crate) fn build_bool_rr_measurement<DI>(
    bits: u64,
    input_domain: DI,
    input_metric: DiscreteDistance,
    output_measure: MaxDivergence,
) -> Result<Option<Measurement<DI, DiscreteDistance, MaxDivergence, bool>>, ArithmeticError>
where
    DI: CertifiedDomain<Carrier = bool>,
    (DI, DiscreteDistance): MetricSpace,
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
    // All privacy properties are proved on these concrete components,
    // before anything is stored in Function or PrivacyMap.
    let program = OneWordProgram {
        operator,
        postprocess: operator::BoolRR::release_callback,
    };
    #[cfg(verus_only)]
    proof! {
        assert forall|d: &u32|
            #[trigger] call_requires(
                operator::BoolRR::map_callback, (&operator, d)) by {}
        assert forall|d: &u32, out: Result<f64, ()>|
            #[trigger] call_ensures(
                operator::BoolRR::map_callback, (&operator, d), out)
                implies out == Ok(operator.map(*d, 0.0f64)) by {}
        lemma_built_bool_rr_concrete_valid(
            &input_domain, &input_metric, &output_measure,
            &program, operator::BoolRR::map_callback, release_model);
    }
    #[cfg(verus_only)]
    let measurement = {
        proof_with! { Ghost(release_model) }
        Measurement::new_certified(
            input_domain, input_metric, output_measure,
            program, operator::BoolRR::map_callback)
    }?;
    #[cfg(not(verus_only))]
    let measurement = Measurement::new_certified(
        input_domain, input_metric, output_measure,
        program, operator::BoolRR::map_callback,
    )?;
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
        &&& verified_arithmetic::float_value(op.privacy_cost) >= 0real
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

/// The complete pre-erasure proof obligation of new_certified.
/// No erased Function, PrivacyMap, witness token, or callback adapter occurs
/// here. The proof uses the actual production operator and two named methods.
pub proof fn lemma_built_bool_rr_concrete_valid<DI, F, P>(
    domain: &DI,
    metric: &DiscreteDistance,
    measure: &MaxDivergence,
    program: &OneWordProgram<operator::BoolRR, F>,
    map: P,
    model: crate::uniform_word::OneWordModel<bool, bool>,
)
    where
        DI: crate::CertifiedDomain<Carrier = bool>,
        F: Fn(&operator::BoolRR, &bool, Result<u64, ()>) -> Result<bool, ()>,
        P: Fn(&operator::BoolRR, &u32) -> Result<f64, ()>,
    requires
        well_formed_bool_rr(program.operator),
        crate::measurement_parts::exact_word_postprocessing(
            program.operator, program.postprocess, model),
        forall|truth: bool, word: Result<u64, ()>|
            #[trigger] model(truth, word) == program.operator.release(truth, word),
        forall|d: &u32| #[trigger] call_requires(map, (&program.operator, d)),
        forall|d: &u32, out: Result<f64, ()>|
            #[trigger] call_ensures(map, (&program.operator, d), out)
                ==> out == Ok(program.operator.map(*d, 0.0f64)),
        crate::bernoulli_proof::uniform_word_expectation_rule(
            crate::uniform_word::word_expectation()),
    ensures crate::measurement_proof::valid_one_word_components(
        domain, metric, measure, program, map, model),
{
    use crate::{CertifiedDomain, CertifiedMetric, CertifiedMeasure};
    let op = program.operator;
    let words = crate::uniform_word::word_expectation();
    let expected = |truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
        crate::randomized_response_bool::release_expectation(
            truth, op, words, payoff);
    let actual = |truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
        crate::uniform_word::one_word_expectation(truth, model, payoff);
    assert forall|truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
        #[trigger] actual(truth, payoff) == expected(truth, payoff)
    by {
        let before = |word: Result<u64, ()>| payoff(model(truth, word));
        let after = |word: Result<u64, ()>| payoff(op.release(truth, word));
        assert forall|word: Result<u64, ()>| #[trigger] before(word) == after(word) by {
            assert(model(truth, word) == op.release(truth, word));
        }
        assert(before =~= after);
    }
    lemma_built_bool_rr_valid(op, words);

    assert forall|d: &u32| metric.valid_distance(d) ==>
        #[trigger] call_requires(map, (&op, d)) by {}

    assert forall|d: &u32, output: Result<f64, ()>|
        metric.valid_distance(d)
            && #[trigger] call_ensures(map, (&op, d), output)
        implies match output {
            Err(_) => false,
            Ok(cost) => {
                &&& <MaxDivergence as crate::CertifiedMeasure<Result<bool, ()>>>::valid_cost(
                    measure, &cost)
                &&& forall|left: &bool, right: &bool|
                    domain.contains(left) && domain.contains(right)
                        && metric.close(left, right, d)
                    ==> measure.dominates(
                        |payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::uniform_word::one_word_expectation(*left, model, payoff),
                        |payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::uniform_word::one_word_expectation(*right, model, payoff),
                        &cost)
            },
        }
    by {
        assert(output == Ok(op.map(*d, 0.0f64)));
        match output {
            Err(_) => { assert(false); }
            Ok(epsilon) => {
                if *d == 0 {
                    assert(f64_to_bits(epsilon) == 0u64);
                    let bits = f64_to_bits(epsilon);
                    assert(((bits >> 52u64) & 2047u64) != 2047u64)
                        by(bit_vector) requires bits == 0u64;
                    crate::verified_arithmetic::lemma_positive_zero_real(epsilon);
                    crate::verified_arithmetic::real_exp_zero();
                } else if op.numerator == 9_007_199_254_740_992u64 {
                    assert(f64_to_bits(epsilon) == 0x7ff0_0000_0000_0000u64);
                    let bits = f64_to_bits(epsilon);
                    assert(((bits >> 52u64) & 2047u64) == 2047u64)
                        by(bit_vector) requires bits == 0x7ff0_0000_0000_0000u64;
                    assert(!crate::verified_arithmetic::finite(epsilon));
                } else {
                    assert(crate::verified_arithmetic::finite(op.privacy_cost));
                    assert(f64_to_bits(epsilon) == f64_to_bits(op.privacy_cost));
                    crate::verified_arithmetic::lemma_same_bits_same_value(
                        epsilon, op.privacy_cost);
                }
                assert(<MaxDivergence as crate::CertifiedMeasure<Result<bool, ()>>>::valid_cost(
                    measure, &epsilon));
                assert(MaxDivergence::multiplicative_factor(epsilon)
                    == operator_factor_map(op, *d));

                assert forall|left: &bool, right: &bool|
                    domain.contains(left) && domain.contains(right)
                        && metric.close(left, right, d)
                    implies measure.dominates(
                        |payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::uniform_word::one_word_expectation(*left, model, payoff),
                        |payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::uniform_word::one_word_expectation(*right, model, payoff),
                        &epsilon)
                by {
                    assert(crate::randomized_response_bool::rr_discrete_close(
                        *left, *right, *d));
                    if let Some(factor) = operator_factor_map(op, *d) {
                        assert(factor >= 1real);
                        assert forall|payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::measurement_proof::bounded_payoff(payoff)
                            implies
                                #[trigger] actual(*left, payoff)
                                    <= factor * actual(*right, payoff)
                        by {
                            assert(expected(*left, payoff)
                                <= factor * expected(*right, payoff));
                        }
                    }
                }
            }
        }
    }
}

} // verus!

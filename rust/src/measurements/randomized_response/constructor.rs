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

#[cfg(verus_only)]
type CallbackError = ();
#[cfg(not(verus_only))]
type CallbackError = crate::error::Error;

#[cfg(verus_only)]
use crate::uniform_word::sample_word_then;
#[cfg(not(verus_only))]
use crate::traits::samplers::sample_word_then;

/// This one-line forwarding body is outside Verus's probabilistic proof.
/// The EPT of the called sample_word_then is verified separately.
#[cfg_attr(verus_only, verus_verify(external_body))]
fn run_bool_rr(
    operator: &operator::BoolRR,
    truth: &bool,
) -> Result<bool, CallbackError> {
    sample_word_then(operator, truth, operator::BoolRR::release_callback)
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
    // The one-word computation and deterministic map are proved before
    // building the concrete Fn closures and before any Arc erasure.
    let func = move |truth: &bool| run_bool_rr(&operator, truth);
    let map = move |distance: &u32| operator.map_callback(distance);
    #[cfg(verus_only)]
    proof! {
        trust_bool_rr_closure_forwarding(operator, func, map, release_model);
        lemma_built_bool_rr_concrete_valid(
            &input_domain, &input_metric, &output_measure,
            operator, func, map, release_model);
    }
    let measurement = Measurement::new_certified(
        input_domain, input_metric, output_measure, func, map,
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

/// EXPLICIT TRUST: probabilistic forwarding of the two adjacent literal
/// RR closures. This is not a DP axiom, but it is an unchecked law about
/// their actual execution. The separately verified sampler EPT and the
/// deterministic postprocessor establish the one-word law. The remaining
/// trust is that the Rust Fn closure calls run_bool_rr on its captured operator
/// and the map closure calls the exact map_callback. Verus cannot extract this
/// probabilistic correspondence from a standard Fn closure today.
///
/// This unchecked fact is only justified by the exact literal closure
/// expressions above. Applying it to unrelated callbacks is UNSOUND.
#[verifier::external_body]
proof fn trust_bool_rr_closure_forwarding<F, P>(
    op: operator::BoolRR,
    func: F,
    map: P,
    model: crate::uniform_word::OneWordModel<bool, bool>,
)
    where
        F: Fn(&bool) -> Result<bool, ()> + Copy,
        P: Fn(&u32) -> Result<f64, ()> + Copy,
    requires
        crate::measurement_parts::exact_word_postprocessing(
            op, operator::BoolRR::release_callback, model),
        crate::bernoulli_proof::uniform_word_expectation_rule(
            crate::uniform_word::word_expectation()),
    ensures
        forall|truth: &bool| #[trigger] call_requires(func, (truth,)),
        forall|truth: &bool, payoff: spec_fn(Result<bool, ()>) -> real|
            #[trigger] crate::measurement_proof::callback_expectation(&func, truth, payoff)
                == crate::uniform_word::one_word_expectation(*truth, model, payoff),
        forall|d: &u32| #[trigger] call_requires(map, (d,)),
        forall|d: &u32, output: Result<f64, ()>|
            #[trigger] call_ensures(map, (d,), output)
                ==> output == Ok(op.map(*d, 0.0f64)),
{
}

/// The complete pre-erasure proof obligation of new_certified.
/// No erased Function, PrivacyMap, witness token, or callback adapter occurs
/// here. The proof uses the actual production operator and two named methods.
pub proof fn lemma_built_bool_rr_concrete_valid<DI, F, P>(
    domain: &DI,
    metric: &DiscreteDistance,
    measure: &MaxDivergence,
    op: operator::BoolRR,
    func: F,
    map: P,
    model: crate::uniform_word::OneWordModel<bool, bool>,
)
    where
        DI: crate::CertifiedDomain<Carrier = bool>,
        F: Fn(&bool) -> Result<bool, ()> + Copy,
        P: Fn(&u32) -> Result<f64, ()> + Copy,
    requires
        well_formed_bool_rr(op),
        forall|truth: &bool|
            #[trigger] call_requires(func, (truth,)),
        forall|truth: &bool, payoff: spec_fn(Result<bool, ()>) -> real|
            #[trigger] crate::measurement_proof::callback_expectation(
                &func, truth, payoff)
                == crate::uniform_word::one_word_expectation(*truth, model, payoff),
        forall|truth: bool, word: Result<u64, ()>|
            #[trigger] model(truth, word) == op.release(truth, word),
        forall|d: &u32| #[trigger] call_requires(map, (d,)),
        forall|d: &u32, out: Result<f64, ()>|
            #[trigger] call_ensures(map, (d,), out)
                ==> out == Ok(op.map(*d, 0.0f64)),
        crate::bernoulli_proof::uniform_word_expectation_rule(
            crate::uniform_word::word_expectation()),
    ensures crate::measurement_proof::valid_concrete_components(
        domain, metric, measure, func, map),
{
    use crate::{CertifiedDomain, CertifiedMetric, CertifiedMeasure};
    let words = crate::uniform_word::word_expectation();
    let expected = |truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
        crate::randomized_response_bool::release_expectation(
            truth, op, words, payoff);
    let actual = |truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
        crate::measurement_proof::callback_expectation(&func, &truth, payoff);
    assert forall|truth: bool, payoff: spec_fn(Result<bool, ()>) -> real|
        #[trigger] actual(truth, payoff) == expected(truth, payoff)
    by {
        let before = |word: Result<u64, ()>| payoff(model(truth, word));
        let after = |word: Result<u64, ()>| payoff(op.release(truth, word));
        assert forall|word: Result<u64, ()>| #[trigger] before(word) == after(word) by {
            assert(model(truth, word) == op.release(truth, word));
        }
        assert(before =~= after);
        assert(actual(truth, payoff)
            == crate::uniform_word::one_word_expectation(truth, model, payoff));
    }
    assert forall|input: &bool| domain.contains(input)
        implies #[trigger] call_requires(func, (input,)) by {}
    assert forall|d: &u32| metric.valid_distance(d) ==>
        #[trigger] call_requires(map, (d,)) by {}

    assert forall|d: &u32, output: Result<f64, ()>|
        metric.valid_distance(d)
            && #[trigger] call_ensures(map, (d,), output)
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
                            crate::measurement_proof::callback_expectation(&func, left, payoff),
                        |payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::measurement_proof::callback_expectation(&func, right, payoff),
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
                            crate::measurement_proof::callback_expectation(&func, left, payoff),
                        |payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::measurement_proof::callback_expectation(&func, right, payoff),
                        &epsilon)
                by {
                    assert(DiscreteDistance::bool_close(
                        *left, *right, *d));
                    let left_law = |payoff: spec_fn(Result<bool, ()>) -> real|
                        crate::measurement_proof::callback_expectation(&func, left, payoff);
                    let right_law = |payoff: spec_fn(Result<bool, ()>) -> real|
                        crate::measurement_proof::callback_expectation(&func, right, payoff);
                    if let Some(factor) = operator_factor_map(op, *d) {
                        assert(factor >= 1real);
                        assert forall|payoff: spec_fn(Result<bool, ()>) -> real|
                            crate::measurement_proof::bounded_payoff(payoff)
                            implies
                                #[trigger] left_law(payoff) <= factor * right_law(payoff)
                        by {
                            if *d == 0 {
                                assert(*left == *right);
                            } else {
                                assert(op.numerator < 9_007_199_254_740_992u64);
                                assert(factor == verified_arithmetic::real_exp(
                                    verified_arithmetic::float_value(op.privacy_cost)));
                                assert(op.numerator as real <= factor *
                                    (9_007_199_254_740_992u64 - op.numerator) as real);
                                crate::randomized_response_bool::lemma_bool_rr_dp(
                                    op, words, factor, *left, *right, payoff);
                            }
                            assert(left_law(payoff) == actual(*left, payoff));
                            assert(right_law(payoff) == actual(*right, payoff));
                            assert(actual(*left, payoff) == expected(*left, payoff));
                            assert(actual(*right, payoff) == expected(*right, payoff));
                        }
                    }
                    assert(measure.dominates(left_law, right_law, &epsilon));
                    let displayed_left = |payoff: spec_fn(Result<bool, ()>) -> real|
                        crate::measurement_proof::callback_expectation(&func, left, payoff);
                    let displayed_right = |payoff: spec_fn(Result<bool, ()>) -> real|
                        crate::measurement_proof::callback_expectation(&func, right, payoff);
                    assert(displayed_left =~= left_law);
                    assert(displayed_right =~= right_law);
                    assert(measure.dominates(displayed_left, displayed_right, &epsilon));
                }
            }
        }
    }
}

} // verus!

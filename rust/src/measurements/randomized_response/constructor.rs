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
type RRMapResult = Result<f64, ()>;

#[cfg(verus_only)]
type CallbackError = ();
#[cfg(not(verus_only))]
type CallbackError = crate::error::Error;

#[cfg(verus_only)]
use crate::uniform_word::sample_word_then;
#[cfg(not(verus_only))]
use crate::traits::samplers::sample_word_then;

// Verification-only in-band credit arguments are erased from the native
// signature. The executable function is shared: native and Verus both
// delegate to the production sample_word_then/release_callback.
#[cfg(verus_only)]
type RRWithCredit = (
    Result<bool, CallbackError>, Tracked<crate::one_word_credit::ErrorCredit>,
);
#[cfg(not(verus_only))]
type RRWithCredit = Result<bool, CallbackError>;

#[cfg(verus_only)]
verus! {
pub(crate) fn run_bool_rr_with_effect(
    operator: &operator::BoolRR,
    truth: &bool,
    credit: Tracked<crate::one_word_credit::ErrorCredit>,
    payoff: Ghost<crate::uniform_word::OutputCost<bool>>,
) -> (result: RRWithCredit)
    requires
        operator.numerator <= 9_007_199_254_740_992u64,
        forall|out: Result<bool, CallbackError>|
            0real <= #[trigger] payoff@(out) < 1real,
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::randomized_response_bool::release_expectation(
                *truth, *operator, crate::uniform_word::word_expectation(), payoff@)
        }),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: payoff@(result.0)
        }),
{
    let ghost word_cost: crate::uniform_word::WordCost =
        |word: Result<u64, CallbackError>| {
            payoff@(operator.release(*truth, word))
        };
    proof {
        operator::lemma_release_callback_preserves_cost(*operator, truth, payoff@);
        assert(crate::uniform_word::uniform_word_expectation(word_cost)
            == crate::randomized_response_bool::release_expectation(
                *truth, *operator, crate::uniform_word::word_expectation(), payoff@));
    }
    sample_word_then(
        operator, truth, operator::BoolRR::release_callback,
        credit, Ghost(word_cost), payoff,
    )
}
} // verus!

#[cfg(not(verus_only))]
pub(crate) fn run_bool_rr_with_effect(
    operator: &operator::BoolRR,
    truth: &bool,
) -> RRWithCredit {
    sample_word_then(operator, truth, operator::BoolRR::release_callback)
}

/// The plain callback has no per-call credit argument. Its exact native
/// executable body forwards to the effect-verified implementation above.
/// Connecting the resulting distribution to callback_expectation remains
/// outside ordinary Verus's Fn semantics.
#[cfg_attr(verus_only, verus_verify(external_body))]
fn run_bool_rr(
    operator: &operator::BoolRR,
    truth: &bool,
) -> Result<bool, CallbackError> {
    #[cfg(not(verus_only))]
    {
        run_bool_rr_with_effect(operator, truth)
    }
    #[cfg(verus_only)]
    {
        unimplemented!()
    }
}

/// Build the actual OpenDP Measurement, including the real certified
/// constructor call. The public API supplies AtomDomain<bool>,
/// DiscreteDistance, and MaxDivergence; this shared function performs the
/// same executable work in the native and Verus builds.
///
/// Verus establishes privacy validity of concrete func/map callbacks before
/// calling the generic new_certified, which requires that validity.
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
    // The one-word sampler has its own verified EPT. Verify the map
    // closure and isolate the remaining probabilistic Fn assumption here.
    let func = move |truth: &bool| run_bool_rr(&operator, truth);
    // The two configurations have identical native closure bodies; Verus
    // needs its return postcondition attached to the closure expression.
    #[cfg(verus_only)]
    let map = #[verus_spec(result: RRMapResult =>
        ensures result == Ok(operator.map(*distance, 0.0f64)),
    )]
    move |distance: &u32| operator.map_callback(distance);
    #[cfg(not(verus_only))]
    let map = move |distance: &u32| operator.map_callback(distance);
    #[cfg(verus_only)]
    proof! {
        // The sole unchecked probabilistic correspondence: this precise
        // closure calls run_bool_rr, which forwards to the verified sampler.
        // This is NOT a reusable theorem about arbitrary Fn values.
        assume(forall|truth: &bool, payoff: spec_fn(Result<bool, ()>) -> real|
            crate::measurement_proof::bounded_payoff(payoff) ==>
                #[trigger] crate::measurement_proof::callback_expectation(
                    &func, truth, payoff)
                == crate::randomized_response_bool::release_expectation(
                    *truth, operator, crate::uniform_word::word_expectation(), payoff));
        // Unlike the randomized result law, ordinary Fn postconditions
        // suffice to verify the deterministic privacy-map closure.
        assert forall|truth: &bool|
            #[trigger] call_requires(func, (truth,)) by {}
        assert forall|distance: &u32|
            #[trigger] call_requires(map, (distance,)) by {}
        assert forall|distance: &u32, out: Result<f64, ()>|
            #[trigger] call_ensures(map, (distance,), out)
                implies out == Ok(operator.map(*distance, 0.0f64)) by {}
        lemma_built_bool_rr_concrete_valid(
            &input_domain, &input_metric, &output_measure,
            operator, func, map);
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
            crate::measurement_proof::bounded_payoff(payoff) ==>
                #[trigger] crate::measurement_proof::callback_expectation(
                    &func, truth, payoff)
                == crate::randomized_response_bool::release_expectation(
                    *truth, op, crate::uniform_word::word_expectation(), payoff),
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
        crate::measurement_proof::bounded_payoff(payoff) implies
            #[trigger] actual(truth, payoff) == expected(truth, payoff)
    by {
        assert(crate::measurement_proof::bounded_payoff(payoff));
        assert(crate::measurement_proof::callback_expectation(&func, &truth, payoff)
            == crate::randomized_response_bool::release_expectation(
                truth, op, crate::uniform_word::word_expectation(), payoff));
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
                            assert(crate::measurement_proof::bounded_payoff(payoff));
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

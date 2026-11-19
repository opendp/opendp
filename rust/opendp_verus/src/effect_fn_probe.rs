//! Exercise the actual production Boolean RR sampler through an executable
//! Fn closure with in-band linear credits. This is a verification-only test;
//! it does not assert the expectation law of the plain Measurement Fn.
use vstd::prelude::*;
verus! {

pub fn probe_rr_fn_in_band(
    operator: crate::operator::BoolRR,
    input: bool,
    credit: Tracked<crate::one_word_credit::ErrorCredit>,
    payoff: Ghost<crate::uniform_word::OutputCost<bool>>,
) -> (result: (Result<bool, crate::uniform_word::WordError>,
              Tracked<crate::one_word_credit::ErrorCredit>))
    requires
        operator.numerator <= 9_007_199_254_740_992u64,
        forall|out: Result<bool, crate::uniform_word::WordError>|
            0real <= #[trigger] payoff@(out),
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::randomized_response_bool::release_expectation(
                input, operator, crate::uniform_word::word_expectation(), payoff@),
        }),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: payoff@(result.0),
        }),
{
    let func =
        |truth: &bool,
         token: Tracked<crate::one_word_credit::ErrorCredit>,
         cost: Ghost<crate::uniform_word::OutputCost<bool>>|
         -> (out: (Result<bool, crate::uniform_word::WordError>,
                    Tracked<crate::one_word_credit::ErrorCredit>))
        requires
            operator.numerator <= 9_007_199_254_740_992u64,
            forall|value: Result<bool, crate::uniform_word::WordError>|
                0real <= #[trigger] cost@(value),
            token@@ =~= (crate::one_word_credit::CreditCarrier::Value {
                amount: crate::randomized_response_bool::release_expectation(
                    *truth, operator, crate::uniform_word::word_expectation(), cost@),
            }),
        ensures
            out.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
                amount: cost@(out.0),
            }),
        {
            crate::rr_constructor::run_bool_rr_with_effect(&operator, truth, token, cost)
        };
    func(&input, credit, payoff)
}


/// The indicator takes value *one* on the chosen observable output.
/// This is the payoff needed for a direct event-probability bound.
pub open spec fn true_event_cost() -> crate::uniform_word::OutputCost<bool> {
    |outcome: Result<bool, crate::uniform_word::WordError>|
        if outcome == Ok(true) { 1real } else { 0real }
}

/// An effect-aware call instantiated with a 0/1 event indicator.
/// This would not typecheck under the old strict (< 1) allocation rule.
/// It is deliberately a probabilistic credit judgment, *not* a claim
/// that the observed event is impossible in ordinary Rust execution.
pub fn probe_rr_true_event(
    operator: crate::operator::BoolRR,
    truth: bool,
    credit: Tracked<crate::one_word_credit::ErrorCredit>,
) -> (result: (Result<bool, crate::uniform_word::WordError>,
              Tracked<crate::one_word_credit::ErrorCredit>))
    requires
        operator.numerator <= 9_007_199_254_740_992u64,
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::randomized_response_bool::release_expectation(
                truth, operator, crate::uniform_word::word_expectation(), true_event_cost()),
        }),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: true_event_cost()(result.0),
        }),
{
    assert forall|out: Result<bool, crate::uniform_word::WordError>|
        0real <= #[trigger] true_event_cost()(out) by {};
    crate::rr_constructor::run_bool_rr_with_effect(
        &operator, &truth, credit, Ghost(true_event_cost()))
}

} // verus!

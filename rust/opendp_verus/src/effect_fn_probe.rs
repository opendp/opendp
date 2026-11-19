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

} // verus!

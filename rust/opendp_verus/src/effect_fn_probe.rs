//! Verus-only probe: put linear effect resources in an ordinary Fn's
//! argument and result types, avoiding unsupported `with` on closures.
//! No native/OpenDP measurement contract is changed by this experiment.
use vstd::prelude::*;

verus! {

pub fn probe_tracked_fn(x: u64) {
    let f = |input: u64, token: Tracked<u64>| -> (result: (u64, Tracked<u64>))
        ensures
            result.0 == input,
            result.1@ == token@,
    {
        (input, token)
    };

    let tracked before: u64 = 7;
    let (output, Tracked(after)) = f(x, Tracked(before));
    assert(output == x);
    assert(after == 7);
}

pub fn probe_generic_fn<F>(
    f: F,
    x: u64,
    token: Tracked<u64>,
) -> (result: (u64, Tracked<u64>))
    where F: Fn(u64, Tracked<u64>) -> (u64, Tracked<u64>),
    requires
        f.requires((x, token)),
        forall|out: (u64, Tracked<u64>)|
            f.ensures((x, token), out) ==> out.0 == x,
    ensures result.0 == x,
{
    f(x, token)
}

} // verus!

// The real production sampler now sits behind an ordinary Fn whose *types*
// carry the resource; no closure-specific `with` annotation is needed.
verus! {

pub fn rr_effect_in_band(
    op: &crate::operator::BoolRR,
    truth: &bool,
    start: Tracked<crate::one_word_credit::ErrorCredit>,
    payoff: Ghost<crate::uniform_word::OutputCost<bool>>,
) -> (result: (Result<bool, crate::uniform_word::WordError>,
               Tracked<crate::one_word_credit::ErrorCredit>))
    requires
        op.numerator <= 9_007_199_254_740_992u64,
        forall|out: Result<bool, crate::uniform_word::WordError>|
            0real <= #[trigger] payoff@(out) < 1real,
        start@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::randomized_response_bool::release_expectation(
                *truth, *op, crate::uniform_word::word_expectation(), payoff@),
        }),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: payoff@(result.0),
        }),
{
    let Tracked(start_credit) = start;
    let Ghost(output_cost) = payoff;
    proof_with! { Tracked(start_credit), Ghost(output_cost) => Tracked(end_credit) }
    let sampled = crate::effect_callback::sample_bool_rr_with_effect(op, truth);
    (sampled, Tracked(end_credit))
}

pub fn probe_rr_effect_in_fn(
    op: crate::operator::BoolRR,
    truth: bool,
    start: Tracked<crate::one_word_credit::ErrorCredit>,
    payoff: Ghost<crate::uniform_word::OutputCost<bool>>,
) -> (result: (Result<bool, crate::uniform_word::WordError>,
               Tracked<crate::one_word_credit::ErrorCredit>))
    requires
        op.numerator <= 9_007_199_254_740_992u64,
        forall|out: Result<bool, crate::uniform_word::WordError>|
            0real <= #[trigger] payoff@(out) < 1real,
        start@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::randomized_response_bool::release_expectation(
                truth, op, crate::uniform_word::word_expectation(), payoff@),
        }),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: payoff@(result.0),
        }),
{
    let func =
        |input: &bool,
         credit: Tracked<crate::one_word_credit::ErrorCredit>,
         cost: Ghost<crate::uniform_word::OutputCost<bool>>|
         -> (out: (Result<bool, crate::uniform_word::WordError>,
                    Tracked<crate::one_word_credit::ErrorCredit>))
        requires
            op.numerator <= 9_007_199_254_740_992u64,
            forall|value: Result<bool, crate::uniform_word::WordError>|
                0real <= #[trigger] cost@(value) < 1real,
            credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
                amount: crate::randomized_response_bool::release_expectation(
                    *input, op, crate::uniform_word::word_expectation(), cost@),
            }),
        ensures
            out.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
                amount: cost@(out.0),
            }),
        {
            rr_effect_in_band(&op, input, credit, cost)
        };
    func(&truth, start, payoff)
}

} // verus!

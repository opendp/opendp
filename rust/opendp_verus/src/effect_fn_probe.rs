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


verus! {

// Trusted expectation-preserving RNG, expressed with *in-band* credits.
// This is a prototype of a different interface to the same entropy rule,
// NOT yet a verified bridge to the existing primitive.
#[verifier::external_body]
pub fn draw_word_in_band(
    credit: Tracked<crate::one_word_credit::ErrorCredit>,
    cost: Ghost<crate::uniform_word::WordCost>,
) -> (result: (Result<u64, crate::uniform_word::WordError>,
              Tracked<crate::one_word_credit::ErrorCredit>))
    requires
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::uniform_word::uniform_word_expectation(cost@),
        }),
        forall|word: Result<u64, crate::uniform_word::WordError>|
            0real <= #[trigger] cost@(word) < 1real,
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: cost@(result.0),
        }),
{
    let word = crate::uniform_word::sample_uniform_u64();
    (word, Tracked::assume_new())
}

pub fn rr_in_band(
    operator: &crate::operator::BoolRR,
    truth: &bool,
    credit: Tracked<crate::one_word_credit::ErrorCredit>,
    payoff: Ghost<crate::uniform_word::OutputCost<bool>>,
) -> (result: (Result<bool, crate::uniform_word::WordError>,
              Tracked<crate::one_word_credit::ErrorCredit>))
    requires
        operator.numerator <= 9_007_199_254_740_992u64,
        forall|out: Result<bool, crate::uniform_word::WordError>|
            0real <= #[trigger] payoff@(out) < 1real,
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::randomized_response_bool::release_expectation(
                *truth, *operator, crate::uniform_word::word_expectation(), payoff@),
        }),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: payoff@(result.0),
        }),
{
    let ghost word_cost: crate::uniform_word::WordCost =
        |word: Result<u64, crate::uniform_word::WordError>|
            payoff@(operator.release(*truth, word));
    proof {
        crate::operator::lemma_release_callback_preserves_cost(*operator, truth, payoff@);
        assert(crate::uniform_word::uniform_word_expectation(word_cost)
            == crate::randomized_response_bool::release_expectation(
                *truth, *operator, crate::uniform_word::word_expectation(), payoff@));
    }
    let (word, credit_after) = draw_word_in_band(credit, Ghost(word_cost));
    let output = operator.release_callback(truth, word);
    proof {
        assert(output == operator.release(*truth, word));
        assert(credit_after@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: payoff@(output)
        }));
    }
    (output, credit_after)
}

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
            0real <= #[trigger] payoff@(out) < 1real,
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
                0real <= #[trigger] cost@(value) < 1real,
            token@@ =~= (crate::one_word_credit::CreditCarrier::Value {
                amount: crate::randomized_response_bool::release_expectation(
                    *truth, operator, crate::uniform_word::word_expectation(), cost@),
            }),
        ensures
            out.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
                amount: cost@(out.0),
            }),
        {
            rr_in_band(&operator, truth, token, cost)
        };
    func(&input, credit, payoff)
}

} // verus!


// Reverse direction: if the in-band RNG primitive is the only trusted
// boundary, can the *existing* ghost-output sampler API be verified by
// forwarding its tracked input/output? This is not an RNG axiom.
#[verus_verify]
#[verus_spec(result =>
    with
        Tracked(start): Tracked<crate::one_word_credit::ErrorCredit>,
        Ghost(word_cost): Ghost<crate::uniform_word::WordCost>
            -> end: Tracked<crate::one_word_credit::ErrorCredit>
    requires
        start@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: crate::uniform_word::uniform_word_expectation(word_cost)
        }),
        forall|word: Result<u64, crate::uniform_word::WordError>|
            0real <= #[trigger] word_cost(word) < 1real,
    ensures
        end@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: word_cost(result)
        }),
)]
pub fn back_to_ghost_output() -> Result<u64, crate::uniform_word::WordError> {
    proof_decl! {
        let tracked input_credit = Tracked(start);
        let ghost cost = Ghost(word_cost);
    }
    let (word, output_credit) = draw_word_in_band(input_credit, cost);
    proof_with! { |= output_credit }
    word
}

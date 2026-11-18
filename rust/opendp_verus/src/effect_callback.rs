//! Prototype: an effect-aware *named* callback for the actual production RR sampler.
//!
//! Unlike a normal Fn closure, this function explicitly accepts and returns
//! an erased tracked error credit. The body is verified using the production
//! sample_word_then and BoolRR::release_callback. This is deliberately not
//! a certified Fn adapter or an assertion about callback_expectation.
use vstd::prelude::*;
use crate::{operator, one_word_credit, uniform_word, randomized_response_bool};

#[verus_verify]
#[verus_spec(result =>
    with
        Tracked(start): Tracked<one_word_credit::ErrorCredit>,
        Ghost(output_cost): Ghost<uniform_word::OutputCost<bool>>
            -> end: Tracked<one_word_credit::ErrorCredit>
    requires
        operator.numerator <= 9_007_199_254_740_992u64,
        forall|out: Result<bool, uniform_word::WordError>|
            0real <= #[trigger] output_cost(out) < 1real,
        start@ =~= (one_word_credit::CreditCarrier::Value {
            amount: randomized_response_bool::release_expectation(
                *truth, *operator, uniform_word::word_expectation(), output_cost)
        }),
    ensures
        end@@ =~= (one_word_credit::CreditCarrier::Value {
            amount: output_cost(result)
        }),
)]
pub fn sample_bool_rr_with_effect(
    operator: &operator::BoolRR,
    truth: &bool,
) -> Result<bool, uniform_word::WordError> {
    proof_decl! {
        let ghost word_cost: uniform_word::WordCost =
            |word: Result<u64, uniform_word::WordError>| {
                output_cost(operator.release(*truth, word))
            };
    }
    proof! {
        operator::lemma_release_callback_preserves_cost(*operator, truth, output_cost);
        let lhs = |word: Result<u64, uniform_word::WordError>|
            output_cost(operator.release(*truth, word));
        assert(word_cost =~= lhs);
        assert(uniform_word::uniform_word_expectation(word_cost) ==
            randomized_response_bool::release_expectation(
                *truth, *operator, uniform_word::word_expectation(), output_cost));
        assert(start@ =~= (one_word_credit::CreditCarrier::Value {
            amount: uniform_word::uniform_word_expectation(word_cost)
        }));
    }
    proof_with! {
        Tracked(start), Ghost(word_cost), Ghost(output_cost) => Tracked(draw_credit)
    }
    let sampled = uniform_word::sample_word_then(
        operator, truth, operator::BoolRR::release_callback);
    let _ = sampled;
    proof_with! { |= Tracked(draw_credit) }
    Ok(*truth) // Deliberately wrong output: must fail the EPT contract.
}

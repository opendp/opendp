//! Expectation-preserving proof of OpenDP's *executable* Boolean RR.
//!
//! This is deliberately not a second implementation or a general wp library.
//! BoolRR::release is the production method, imported with Verus dual_spec.
//! The only probabilistic premise describes the production entropy call.
//! Rounding the f64 privacy map and closure erasure remain separate tasks.
use vstd::prelude::*;
use crate::operator;
use crate::bernoulli_proof::{
    prefix_post, prefix_threshold, lemma_success_is_prefix,
    lemma_bernoulli_probability, dyadic53_mass, uniform_word_expectation_rule,
};

verus! {

pub open spec fn bounded_payoff<E>(f: spec_fn(Result<bool, E>) -> real) -> bool {
    forall|outcome: Result<bool, E>| 0real <= #[trigger] f(outcome) <= 1real
}

/// Expected payoff of calling the *actual* BoolRR::release after one word
/// from sample_uniform_u64. The `word_expectation` argument is the semantic
/// expectation functional of that exact executable entropy call.
pub open spec fn release_expectation<E>(
    truth: bool,
    numerator: u64,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    payoff: spec_fn(Result<bool, E>) -> real,
) -> real {
    word_expectation(|sample: Result<u64, E>| payoff(
        (operator::BoolRR { numerator, privacy_cost: 0u64 }).release(truth, sample)
    ))
}

/// Expectation preservation for the production deterministic postprocessor.
/// A caller supplies only the uniform entropy primitive rule; every step
/// above entropy is verified executable Rust.
pub proof fn lemma_release_expectation<E>(
    truth: bool,
    numerator: u64,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    error_expectation: spec_fn(spec_fn(E) -> real) -> real,
    success: real,
    payoff: spec_fn(Result<bool, E>) -> real,
)
    requires
        numerator <= 9_007_199_254_740_992u64,
        bounded_payoff(payoff),
        uniform_word_expectation_rule(word_expectation, error_expectation, success),
    ensures
        release_expectation(truth, numerator, word_expectation, payoff)
            == success * (
                (numerator as real / 9_007_199_254_740_992real) * payoff(Ok(truth))
                + (1real - numerator as real / 9_007_199_254_740_992real) * payoff(Ok(!truth))
            ) + error_expectation(|err: E| payoff(Err(err))),
{
    let threshold = prefix_threshold(numerator);
    let on_error = |err: E| payoff(Err(err));
    let operator = operator::BoolRR { numerator, privacy_cost: 0u64 };
    let before = |sample: Result<u64, E>| payoff(operator.release(truth, sample));
    let prefix = prefix_post(
        threshold, payoff(Ok(truth)), payoff(Ok(!truth)), on_error,
    );
    assert forall|err: E| 0real <= #[trigger] on_error(err) <= 1real by {
        assert(0real <= payoff(Err(err)) <= 1real);
    }
    assert forall|sample: Result<u64, E>| #[trigger] before(sample) == prefix(sample) by {
        match sample {
            Ok(word) => {
                lemma_success_is_prefix(numerator, word);
                if (word as nat) < threshold {} else {}
            }
            Err(err) => {}
        }
    }
    assert(before =~= prefix);
    assert(release_expectation(truth, numerator, word_expectation, payoff)
        == word_expectation(prefix));

    assert(threshold == (numerator as nat) * 2048nat);
    assert((numerator as nat) <= 9_007_199_254_740_992nat);
    assert(threshold <= 18_446_744_073_709_551_616nat)
        by(nonlinear_arith);
    assert(0real <= payoff(Ok(truth)) <= 1real);
    assert(0real <= payoff(Ok(!truth)) <= 1real);
    assert(word_expectation(prefix)
        == success * (
            threshold as real / 18_446_744_073_709_551_616real * payoff(Ok(truth))
            + (1real - threshold as real / 18_446_744_073_709_551_616real) * payoff(Ok(!truth))
        ) + error_expectation(on_error));
    lemma_bernoulli_probability(numerator);
    assert(threshold as real / 18_446_744_073_709_551_616real
        == dyadic53_mass(numerator));
}

/// Pure DP for arbitrary bounded postconditions on the *actual Result* type.
/// Unlike a proof over modeled PMF atoms, error values are never erased.
pub proof fn lemma_bool_rr_dp<E>(
    numerator: u64,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    error_expectation: spec_fn(spec_fn(E) -> real) -> real,
    success: real,
    factor: real,
    left: bool,
    right: bool,
    distance: u32,
    payoff: spec_fn(Result<bool, E>) -> real,
)
    requires
        4_503_599_627_370_496u64 <= numerator <= 9_007_199_254_740_992u64,
        factor >= 1real,
        numerator < 9_007_199_254_740_992u64 ==>
            numerator as real / 9_007_199_254_740_992real
                <= factor * (1real - numerator as real / 9_007_199_254_740_992real),
        left == right || distance > 0,
        bounded_payoff(payoff),
        uniform_word_expectation_rule(word_expectation, error_expectation, success),
    ensures
        distance == 0 ==>
            release_expectation(left, numerator, word_expectation, payoff)
                <= release_expectation(right, numerator, word_expectation, payoff),
        distance > 0 && numerator < 9_007_199_254_740_992u64 ==>
            release_expectation(left, numerator, word_expectation, payoff)
                <= factor * release_expectation(right, numerator, word_expectation, payoff),
        // With p=1 and positive distance, the privacy map is +infinity.
{
    if distance == 0 {
        assert(left == right);
    } else if numerator < 9_007_199_254_740_992u64 {
        lemma_release_expectation(left, numerator, word_expectation, error_expectation, success, payoff);
        lemma_release_expectation(right, numerator, word_expectation, error_expectation, success, payoff);
        let p = numerator as real / 9_007_199_254_740_992real;
        let on_error = |err: E| payoff(Err(err));
        let err = error_expectation(on_error);
        let a = payoff(Ok(left));
        let b = payoff(Ok(!left));
        assert forall|e: E| 0real <= #[trigger] on_error(e) <= 1real by {
            assert(0real <= payoff(Err(e)) <= 1real);
        }
        assert(0real <= err);
        assert(0real <= success);
        assert(0real <= a <= 1real);
        assert(0real <= b <= 1real);
        assert(1real / 2real <= p < 1real);
        if left == right {
            let value = success * (p * a + (1real - p) * b) + err;
            assert(p * a >= 0real) by(nonlinear_arith);
            assert((1real - p) * b >= 0real) by(nonlinear_arith);
            assert(value >= 0real) by(nonlinear_arith);
            assert(release_expectation(left, numerator, word_expectation, payoff)
                == release_expectation(right, numerator, word_expectation, payoff));
        } else {
            assert(right == !left);
            assert(1real - p <= p);
            assert(1real - p <= factor * p) by(nonlinear_arith)
                requires 0real <= p, 1real - p <= p, factor >= 1real;
            let vl = p * a + (1real - p) * b;
            let vr = (1real - p) * a + p * b;
            assert(vl <= factor * vr) by(nonlinear_arith)
                requires
                    0real <= a, 0real <= b,
                    p <= factor * (1real - p),
                    1real - p <= factor * p;
            assert(success * vl + err <= factor * (success * vr + err))
                by(nonlinear_arith)
                requires
                    0real <= success, 0real <= err, factor >= 1real,
                    vl <= factor * vr;
        }
    }
}

} // verus!

//! Expectation-preserving proof of OpenDP's *executable* Boolean RR.
//!
//! This is deliberately not a second implementation or a general wp library.
//! BoolRR::release is the production method, imported with Verus dual_spec.
//! The only probabilistic premise describes the production entropy call.
//! The constructor separately verifies its numeric bound and audited erasure.
use vstd::prelude::*;
use crate::operator;
use crate::measurement_proof::bounded_payoff;
use crate::bernoulli_proof::{
    prefix_post, prefix_threshold, lemma_success_is_prefix,
    lemma_bernoulli_probability, uniform_word_expectation_rule,
    success_mass, error_mass, zero_error_post,
};

verus! {

/// Small, reusable real-number facts keep the RR proof out of nonlinear SMT.
proof fn nonnegative_product(x: real, y: real)
    requires x >= 0real, y >= 0real,
    ensures x * y >= 0real,
{
    assert(x * y >= 0real) by(nonlinear_arith)
        requires x >= 0real, y >= 0real;
}

proof fn multiply_inequality(x: real, y: real, weight: real)
    requires x <= y, weight >= 0real,
    ensures x * weight <= y * weight,
{
    assert(x * weight <= y * weight) by(nonlinear_arith)
        requires x <= y, weight >= 0real;
}

proof fn distribute(x: real, y: real, z: real)
    ensures x * (y + z) == x * y + x * z,
{
    assert(x * (y + z) == x * y + x * z) by(nonlinear_arith);
}

proof fn mul_associative(x: real, y: real, z: real)
    ensures x * (y * z) == (x * y) * z,
{
    assert(x * (y * z) == (x * y) * z) by(nonlinear_arith);
}

proof fn mul_rotate(x: real, y: real, z: real)
    ensures x * (y * z) == y * (x * z),
{
    assert(x * (y * z) == y * (x * z)) by(nonlinear_arith);
}

/// Algebraic core: swap two outcome weights, retain the same error mass.
/// This does not mention Boolean RR, entropy, adjacency, or OpenDP types.
proof fn two_outcome_domination(
    p: real, factor: real, a: real, b: real, success: real, error: real,
)
    requires
        0real <= p <= 1real,
        factor >= 1real,
        p <= factor * (1real - p),
        1real - p <= factor * p,
        a >= 0real, b >= 0real, success >= 0real, error >= 0real,
    ensures
        success * (p * a + (1real - p) * b) + error
            <= factor * (success * ((1real - p) * a + p * b) + error),
{
    let left_one = p * a;
    let left_two = (1real - p) * b;
    let right_one = (1real - p) * a;
    let right_two = p * b;
    let left = left_one + left_two;
    let right = right_one + right_two;
    multiply_inequality(p, factor * (1real - p), a);
    multiply_inequality(1real - p, factor * p, b);
    mul_associative(factor, 1real - p, a);
    mul_associative(factor, p, b);
    assert(left_one <= factor * right_one);
    assert(left_two <= factor * right_two);
    distribute(factor, right_one, right_two);
    assert(left <= factor * right);
    multiply_inequality(left, factor * right, success);
    mul_rotate(success, factor, right);
    multiply_inequality(1real, factor, error);
    distribute(factor, success * right, error);
}

/// Expected payoff of calling the *actual* BoolRR::release after one word
/// from sample_uniform_u64. The `word_expectation` argument is the semantic
/// expectation functional of that exact executable entropy call.
pub open spec fn release_expectation<E>(
    truth: bool,
    op: operator::BoolRR,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    payoff: spec_fn(Result<bool, E>) -> real,
) -> real {
    word_expectation(|sample: Result<u64, E>| payoff(
        op.release(truth, sample)
    ))
}

/// Expectation preservation for the production deterministic postprocessor.
/// A caller supplies only the uniform entropy primitive rule; every step
/// above entropy is verified executable Rust.
pub proof fn lemma_release_expectation<E>(
    truth: bool,
    op: operator::BoolRR,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    payoff: spec_fn(Result<bool, E>) -> real,
)
    requires
        op.numerator <= 9_007_199_254_740_992u64,
        bounded_payoff(payoff),
        uniform_word_expectation_rule(word_expectation),
    ensures
        release_expectation(truth, op, word_expectation, payoff)
            == success_mass(word_expectation) * (
                (op.numerator as real / 9_007_199_254_740_992real) * payoff(Ok(truth))
                + (1real - op.numerator as real / 9_007_199_254_740_992real) * payoff(Ok(!truth))
            ) + error_mass(word_expectation, |err: E| payoff(Err(err))),
{
    let numerator = op.numerator;
    let threshold = prefix_threshold(numerator);
    let on_error = |err: E| payoff(Err(err));
    let before = |sample: Result<u64, E>| payoff(op.release(truth, sample));
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
    assert(release_expectation(truth, op, word_expectation, payoff)
        == word_expectation(prefix));

    assert(threshold == (numerator as nat) * 2048nat);
    assert((numerator as nat) <= 9_007_199_254_740_992nat);
    assert((numerator as nat) * 2048nat
        <= 18_446_744_073_709_551_616nat) by(nonlinear_arith)
        requires (numerator as nat) <= 9_007_199_254_740_992nat;
    assert(threshold <= 18_446_744_073_709_551_616nat);
    assert(0real <= payoff(Ok(truth)) <= 1real);
    assert(0real <= payoff(Ok(!truth)) <= 1real);
    assert(word_expectation(prefix)
        == success_mass(word_expectation) * (
            threshold as real / 18_446_744_073_709_551_616real * payoff(Ok(truth))
            + (1real - threshold as real / 18_446_744_073_709_551_616real) * payoff(Ok(!truth))
        ) + error_mass(word_expectation, on_error));
    lemma_bernoulli_probability(numerator);
    assert(threshold as real / 18_446_744_073_709_551_616real
        == numerator as real / 9_007_199_254_740_992real);
}

/// Indicator of observing a successful true release. It assigns zero to
/// errors and to a successful false release, so this is an event probability
/// rather than an arbitrary payoff expectation.
pub open spec fn true_event<E>() -> spec_fn(Result<bool, E>) -> real {
    |out: Result<bool, E>| match out {
        Ok(true) => 1real,
        _ => 0real,
    }
}

/// Exact true-event mass in the modeled one-word RR distribution.
/// The error branch contributes zero, which requires the RNG model to
/// preserve the expectation of the zero payoff. This theorem uses the
/// production BoolRR::release and introduces no new trusted rule.
pub proof fn lemma_true_event_probability<E>(
    truth: bool,
    op: operator::BoolRR,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
)
    requires
        op.numerator <= 9_007_199_254_740_992u64,
        uniform_word_expectation_rule(word_expectation),
    ensures
        release_expectation(truth, op, word_expectation, true_event())
            == success_mass(word_expectation) *
                (if truth {
                    op.numerator as real / 9_007_199_254_740_992real
                } else {
                    1real - op.numerator as real / 9_007_199_254_740_992real
                }),
{
    let payoff = true_event::<E>();
    assert forall|out: Result<bool, E>|
        0real <= #[trigger] payoff(out) <= 1real by {
        match out {
            Ok(value) => { if value {} else {} }
            Err(_) => {}
        }
    }
    assert(bounded_payoff(payoff));
    lemma_release_expectation(truth, op, word_expectation, payoff);
    let on_error = |err: E| payoff(Err(err));
    let zero = zero_error_post::<E>();
    assert forall|err: E| #[trigger] on_error(err) == zero(err) by {}
    assert(on_error =~= zero);
    assert(error_mass(word_expectation, zero) == 0real);
    assert(error_mass(word_expectation, on_error) == 0real);
    if truth {
        assert(payoff(Ok(truth)) == 1real);
        assert(payoff(Ok(!truth)) == 0real);
    } else {
        assert(payoff(Ok(truth)) == 0real);
        assert(payoff(Ok(!truth)) == 1real);
    }
}

/// Pure DP for arbitrary bounded postconditions on the *actual Result* type.
/// Unlike a proof over modeled PMF atoms, error values are never erased.
pub proof fn lemma_bool_rr_dp<E>(
    op: operator::BoolRR,
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    factor: real,
    left: bool,
    right: bool,
    payoff: spec_fn(Result<bool, E>) -> real,
)
    requires
        4_503_599_627_370_496u64 <= op.numerator < 9_007_199_254_740_992u64,
        factor >= 1real,
        op.numerator as real <= factor
            * (9_007_199_254_740_992u64 - op.numerator) as real,
        bounded_payoff(payoff),
        uniform_word_expectation_rule(word_expectation),
    ensures
        release_expectation(left, op, word_expectation, payoff)
            <= factor * release_expectation(right, op, word_expectation, payoff),
{
    let numerator = op.numerator;
    lemma_release_expectation(left, op, word_expectation, payoff);
    lemma_release_expectation(right, op, word_expectation, payoff);
    let p = numerator as real / 9_007_199_254_740_992real;
    // Convert the constructor's integer odds bound where the
    // finite-cost privacy inequality actually needs it.
    let n = numerator as real;
    let denominator = 9_007_199_254_740_992real;
    let complement = (9_007_199_254_740_992u64 - numerator) as real;
    assert(complement == denominator - n);
    assert(n <= factor * complement);
    assert(n / denominator <= factor * (complement / denominator))
        by(nonlinear_arith)
        requires denominator > 0real, n <= factor * complement;
    assert(complement / denominator == 1real - n / denominator)
        by(nonlinear_arith)
        requires complement == denominator - n, denominator > 0real;
    assert(p <= factor * (1real - p));
    let on_error = |err: E| payoff(Err(err));
    let err = error_mass(word_expectation, on_error);
    let success = success_mass(word_expectation);
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
        let xa = p * a;
        let xb = (1real - p) * b;
        let value = success * (xa + xb) + err;
        nonnegative_product(p, a);
        nonnegative_product(1real - p, b);
        assert(xa + xb >= 0real);
        nonnegative_product(success, xa + xb);
        assert(value >= 0real);
        multiply_inequality(1real, factor, value);
        assert(release_expectation(left, op, word_expectation, payoff)
            == release_expectation(right, op, word_expectation, payoff));
        assert(release_expectation(right, op, word_expectation, payoff)
            == value);
        assert(release_expectation(left, op, word_expectation, payoff)
            <= factor * release_expectation(right, op, word_expectation, payoff));
    } else {
        assert(right == !left);
        assert(1real - p <= p);
        assert(p <= factor * p) by(nonlinear_arith)
            requires p >= 0real, factor >= 1real;
        assert(1real - p <= factor * p);
        two_outcome_domination(p, factor, a, b, success, err);
        assert(release_expectation(left, op, word_expectation, payoff)
            == success * (p * a + (1real - p) * b) + err);
        assert(release_expectation(right, op, word_expectation, payoff)
            == success * ((1real - p) * a + p * b) + err);
    }
}

} // verus!

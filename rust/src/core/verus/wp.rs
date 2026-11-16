//! Probabilistic weakest-preexpectations for verified OpenDP measurements.
//!
//! A weakest-preexpectation assigns a nonnegative expected payoff to an
//! executable probabilistic program. Verus verifies deterministic programs;
//! the semantics of one entropy primitive are supplied as an explicit trusted
//! boundary, never by claiming an ordinary Rust ensures clause is a law.
use vstd::prelude::*;

verus! {

pub open spec fn bounded_post<Y>(post: spec_fn(Y) -> real) -> bool {
    forall|outcome: Y| 0real <= #[trigger] post(outcome) <= 1real
}

/// Pure DP, directly on expected postconditions. A non-finite bound (None)
/// imposes no relational obligation. This is equivalent to event-based pure
/// DP for normalized probability laws; unlike a supplied PMF, the wp must be
/// obtained from the program's own probabilistic semantics.
pub open spec fn valid_wp_max_divergence<X, Y, DI>(
    member: spec_fn(X) -> bool,
    close: spec_fn(X, X, DI) -> bool,
    wp: spec_fn(X, spec_fn(Y) -> real) -> real,
    factor_map: spec_fn(DI) -> Option<real>,
) -> bool {
    forall|left: X, right: X, distance: DI|
        member(left) && member(right)
            && #[trigger] close(left, right, distance)
        ==> match factor_map(distance) {
            None => true,
            Some(factor) => forall|post: spec_fn(Y) -> real|
                bounded_post(post)
                ==> #[trigger] wp(left, post) <= factor * wp(right, post),
        }
}

pub open spec fn word64_cardinality() -> real {
    18_446_744_073_709_551_616real
}

/// A generic threshold payoff on the *actual* fallible uniform-word result.
/// The Err payoff remains a function of the full error payload, rather than
/// collapsing all errors to a single artificial atom.
pub open spec fn word_prefix_post<E>(
    threshold: nat,
    below: real,
    above: real,
    on_error: spec_fn(E) -> real,
) -> spec_fn(Result<u64, E>) -> real {
    |sample: Result<u64, E>| match sample {
        Ok(word) => if (word as nat) < threshold { below } else { above },
        Err(err) => on_error(err),
    }
}

/// The precise probabilistic contract required of sample_uniform_u64().
///
/// `sample_wp` is the true weakest-preexpectation of that production call.
/// `error_wp` is the (subprobability) expectation on each observable error.
/// This predicate is NOT proved by Verus: validating it against the entropy
/// implementation is an explicit trusted/assumed probabilistic boundary.
///
/// For every threshold and every error payoff, successful words have the
/// uniform-u64 CDF. Failure payoffs are independent of the private input.
pub open spec fn uniform_word_wp_contract<E>(
    sample_wp: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    error_wp: spec_fn(spec_fn(E) -> real) -> real,
    success_probability: real,
) -> bool {
    &&& 0real <= success_probability <= 1real
    &&& error_wp(|_: E| 1real) == 1real - success_probability
    &&& forall|on_error: spec_fn(E) -> real|
        bounded_post(on_error) ==> 0real <= #[trigger] error_wp(on_error)
    &&& forall|threshold: nat, below: real, above: real, on_error: spec_fn(E) -> real|
        threshold <= 18_446_744_073_709_551_616nat ==>
            #[trigger] sample_wp(word_prefix_post(threshold, below, above, on_error))
            == success_probability * (
                (threshold as real / word64_cardinality()) * below
                + (1real - threshold as real / word64_cardinality()) * above
            ) + error_wp(on_error)
}

} // verus!

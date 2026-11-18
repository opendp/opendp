//! Generic semantic transfer theorem for valid measurements.
//!
//! This theorem is not a constructor contract or a model of erased Rust
//! callbacks. It can transfer an already-established privacy guarantee across
//! independently established equalities of function and map semantics. The
//! equivalence premises must never be used to assume a mechanism is private.
use vstd::prelude::*;

verus! {

pub open spec fn bounded_payoff<Z>(payoff: spec_fn(Z) -> real) -> bool {
    forall|outcome: Z| 0real <= #[trigger] payoff(outcome) <= 1real
}

/// The source operator and the erased callback have the same output law
/// whenever these expectations agree for all postconditions.
pub open spec fn realizes_expectation<X, Z>(
    callback: spec_fn(X, spec_fn(Z) -> real) -> real,
    model: spec_fn(X, spec_fn(Z) -> real) -> real,
) -> bool {
    forall|input: X, payoff: spec_fn(Z) -> real|
        #[trigger] callback(input, payoff) == model(input, payoff)
}

/// The erased privacy-map callback returns the same abstract factor.
/// None denotes an infinite bound, never a finite zero-cost bound.
pub open spec fn realizes_privacy_map<D>(
    callback: spec_fn(D) -> Option<real>,
    model: spec_fn(D) -> Option<real>,
) -> bool {
    forall|distance: D| #[trigger] callback(distance) == model(distance)
}

/// Pure DP for a fallible measurement, expressed using output expectations.
/// The map already carries the real multiplicative factor, not a rounded
/// binary64 epsilon; connecting it to the reported epsilon is separate.
pub open spec fn valid_measurement_expectations<X, Z, D>(
    member: spec_fn(X) -> bool,
    close: spec_fn(X, X, D) -> bool,
    expectation: spec_fn(X, spec_fn(Z) -> real) -> real,
    factor_map: spec_fn(D) -> Option<real>,
) -> bool {
    forall|left: X, right: X, distance: D|
        member(left) && member(right)
            && #[trigger] close(left, right, distance)
        ==> match factor_map(distance) {
            None => true,
            Some(factor) => factor >= 1real
                && forall|payoff: spec_fn(Z) -> real|
                    bounded_payoff(payoff) ==>
                        #[trigger] expectation(left, payoff)
                            <= factor * expectation(right, payoff),
        }
}

/// A single paired proof: do not put independent certification markers in
/// Function and PrivacyMap. The callback correspondence is a *precondition*,
/// not assumed to hold for arbitrary Arc<dyn Fn> values.
pub proof fn lemma_valid_measurement_from_equivalent_components<X, Z, D>(
    member: spec_fn(X) -> bool,
    close: spec_fn(X, X, D) -> bool,
    model_expectation: spec_fn(X, spec_fn(Z) -> real) -> real,
    model_map: spec_fn(D) -> Option<real>,
    callback_expectation: spec_fn(X, spec_fn(Z) -> real) -> real,
    callback_map: spec_fn(D) -> Option<real>,
)
    requires
        valid_measurement_expectations(
            member, close, model_expectation, model_map),
        realizes_expectation(callback_expectation, model_expectation),
        realizes_privacy_map(callback_map, model_map),
    ensures
        valid_measurement_expectations(
            member, close, callback_expectation, callback_map),
{
    assert forall|left: X, right: X, distance: D|
        member(left) && member(right)
            && #[trigger] close(left, right, distance)
        implies match callback_map(distance) {
            None => true,
            Some(factor) => factor >= 1real
                && forall|payoff: spec_fn(Z) -> real|
                    bounded_payoff(payoff) ==>
                        #[trigger] callback_expectation(left, payoff)
                            <= factor * callback_expectation(right, payoff),
        } by {
        assert(callback_map(distance) == model_map(distance));
        if let Some(factor) = model_map(distance) {
            assert(factor >= 1real);
            assert forall|payoff: spec_fn(Z) -> real|
                bounded_payoff(payoff) implies
                    #[trigger] callback_expectation(left, payoff)
                        <= factor * callback_expectation(right, payoff)
            by {
                assert(callback_expectation(left, payoff) == model_expectation(left, payoff));
                assert(callback_expectation(right, payoff) == model_expectation(right, payoff));
                assert(model_expectation(left, payoff)
                    <= factor * model_expectation(right, payoff));
            }
        }
    }
}

} // verus!

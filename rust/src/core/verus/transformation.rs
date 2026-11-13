//! Generic semantic validity of a fallible stable transformation.
//!
//! The codomain is Result<Y, E>, not Y with a separate data-independent
//! error rule. A Result-distance extends the ordinary output metric:
//! equal errors have distance zero; a success/error pair or unequal errors
//! have infinite distance. The infinity predicate makes this definition
//! independent of the concrete distance representation (u32 has no infinity).

use vstd::prelude::*;

verus! {

/// Closeness under the lifted metric on Result<Y, E>.
///
/// A distance bound marked infinite admits all pairs. Under a finite bound,
/// two successful values must satisfy the underlying output metric, while
/// errors must be exactly equal.
pub open spec fn result_close<Y, E, D>(
    left: Result<Y, E>,
    right: Result<Y, E>,
    distance: D,
    payload_close: spec_fn(Y, Y, D) -> bool,
    is_infinite: spec_fn(D) -> bool,
) -> bool {
    if is_infinite(distance) {
        true
    } else {
        match (left, right) {
            (Ok(y), Ok(y_prime)) => payload_close(y, y_prime, distance),
            (Err(e), Err(e_prime)) => e == e_prime,
            _ => false,
        }
    }
}

/// The mathematical validity predicate for an OpenDP transformation.
///
/// A constructor either fails or returns a transformation satisfying this
/// predicate. Domain, metric and map types are otherwise unrestricted.
/// The function may fail based on its input: failures are observable values
/// in the Result codomain, accounted for by result_close.
///
/// A failed stability-map evaluation imposes no pairwise obligation.
/// Successful map evaluations must justify the lifted output distance.
pub open spec fn valid_transformation<X, Y, DI, DO, E, ME>(
    input_member: spec_fn(X) -> bool,
    output_member: spec_fn(Y) -> bool,
    input_close: spec_fn(X, X, DI) -> bool,
    payload_close: spec_fn(Y, Y, DO) -> bool,
    output_is_infinite: spec_fn(DO) -> bool,
    function: spec_fn(X) -> Result<Y, E>,
    stability_map: spec_fn(DI) -> Result<DO, ME>,
) -> bool {
    &&& forall|x: X| #[trigger] input_member(x) ==> match function(x) {
        Ok(y) => output_member(y),
        Err(_) => true,
    }
    &&& forall|x: X, x_prime: X, d_in: DI|
        input_member(x) && input_member(x_prime)
            && #[trigger] input_close(x, x_prime, d_in)
        ==> match stability_map(d_in) {
            Ok(d_out) => result_close(
                function(x), function(x_prime), d_out,
                payload_close, output_is_infinite,
            ),
            Err(_) => true,
        }
}

/// Exact correspondence between a callback and its declared semantic model.
///
/// This is the only property the erased callback needs to retain: the
/// transformation validity predicate is established for the *pair* of models,
/// not stored separately in Function and StabilityMap.
pub open spec fn realizes_callback<A, B, E>(
    callback: spec_fn(A) -> Result<B, E>,
    model: spec_fn(A) -> Result<B, E>,
) -> bool {
    forall|input: A| #[trigger] callback(input) == model(input)
}

/// A certified constructor uses the pre-erasure validity proof together with
/// the call contracts of the two callbacks. No second transformation-specific
/// validity certificate needs to survive in the runtime object.
pub proof fn lemma_new_certified<X, Y, DI, DO, E, ME>(
    input_member: spec_fn(X) -> bool,
    output_member: spec_fn(Y) -> bool,
    input_close: spec_fn(X, X, DI) -> bool,
    payload_close: spec_fn(Y, Y, DO) -> bool,
    output_is_infinite: spec_fn(DO) -> bool,
    function_model: spec_fn(X) -> Result<Y, E>,
    map_model: spec_fn(DI) -> Result<DO, ME>,
    function: spec_fn(X) -> Result<Y, E>,
    map: spec_fn(DI) -> Result<DO, ME>,
)
    requires
        valid_transformation(
            input_member, output_member, input_close, payload_close,
            output_is_infinite, function_model, map_model),
        realizes_callback(function, function_model),
        realizes_callback(map, map_model),
    ensures
        valid_transformation(
            input_member, output_member, input_close, payload_close,
            output_is_infinite, function, map),
{
    assert forall|x: X|
        #[trigger] input_member(x)
        implies match function(x) {
            Ok(y) => output_member(y),
            Err(_) => true,
        } by {
        assert(function(x) == function_model(x));
    }
    assert forall|x: X, x_prime: X, d_in: DI|
        input_member(x) && input_member(x_prime)
            && #[trigger] input_close(x, x_prime, d_in)
        implies match map(d_in) {
            Ok(d_out) => result_close(
                function(x), function(x_prime), d_out,
                payload_close, output_is_infinite),
            Err(_) => true,
        } by {
        assert(map(d_in) == map_model(d_in));
        assert(function(x) == function_model(x));
        assert(function(x_prime) == function_model(x_prime));
    }
}

/// In the finite-distance case, an error and a success cannot be close.
pub proof fn lemma_mixed_result_requires_infinity<Y, E, D>(
    y: Y,
    e: E,
    distance: D,
    payload_close: spec_fn(Y, Y, D) -> bool,
    is_infinite: spec_fn(D) -> bool,
)
    ensures
        result_close(Err(e), Ok(y), distance, payload_close, is_infinite)
            == is_infinite(distance),
{
}

/// At every finite distance, two errors are close iff they are identical.
pub proof fn lemma_error_result_closeness<Y, E, D>(
    first: E,
    second: E,
    distance: D,
    payload_close: spec_fn(Y, Y, D) -> bool,
    is_infinite: spec_fn(D) -> bool,
)
    requires !is_infinite(distance),
    ensures
        result_close(
            Err(first), Err(second), distance, payload_close, is_infinite,
        ) == (first == second),
{
}

/// Ordinary scalar distances are finite, so the only admitted error pairs
/// have identical observable error values.
pub open spec fn finite_u32(distance: u32) -> bool { false }

} // verus!

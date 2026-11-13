//! Proof of `make_chain_tt`'s Result-valued stability contract.
//!
//! The executable implementation is in the sibling `mod.rs`. This module
//! proves the generic semantic rule used by that combinator. The runtime
//! `Function` and `StabilityMap` use erased Rust closures. Their composition
//! shares the executable, Verus-checked `core/chain_result.rs` implementation;
//! closure contracts are still erased at the native trait-object boundary.

use vstd::prelude::*;
#[cfg(verus_keep_ghost)]
use super::transformation::{result_close, valid_transformation, realizes_callback, lemma_new_certified};

verus! {

/// `Function::make_chain` evaluates the inner function first and propagates
/// its error without invoking the outer function.
pub open spec fn chain_function<X, Y, Z, E>(
    inner: spec_fn(X) -> Result<Y, E>,
    outer: spec_fn(Y) -> Result<Z, E>,
    input: X,
) -> Result<Z, E> {
    match inner(input) {
        Ok(value) => outer(value),
        Err(error) => Err(error),
    }
}

/// `StabilityMap::make_chain` has the same short-circuiting semantics.
pub open spec fn chain_map<DI, DX, DO, ME>(
    inner: spec_fn(DI) -> Result<DX, ME>,
    outer: spec_fn(DX) -> Result<DO, ME>,
    distance: DI,
) -> Result<DO, ME> {
    match inner(distance) {
        Ok(bound) => outer(bound),
        Err(error) => Err(error),
    }
}

/// A composed map must not turn an infinite intermediate distance into a
/// finite output bound. Without this assumption, the first transformation
/// could have mismatched success/error results at the infinite bound.
pub open spec fn preserves_infinity<DX, DO, ME>(
    middle_is_infinite: spec_fn(DX) -> bool,
    output_is_infinite: spec_fn(DO) -> bool,
    outer_map: spec_fn(DX) -> Result<DO, ME>,
) -> bool {
    forall|bound: DX|
        #[trigger] middle_is_infinite(bound)
        ==> match outer_map(bound) {
            Ok(output) => output_is_infinite(output),
            Err(_) => true,
        }
}

/// The semantic contract of `make_chain_tt`, independently of any
/// particular data domain, metric, or operator. The same intermediate
/// membership and distance predicates are used by both transformations.
///
/// This establishes the full Result-valued rule: equal inner errors are
/// propagated, successful inner outputs enter the outer domain, and map
/// errors impose no constraint. Infinite bounds remain infinite on success.
pub proof fn lemma_make_chain_tt<X, Y, Z, DI, DX, DO, E, ME>(
    input_member: spec_fn(X) -> bool,
    middle_member: spec_fn(Y) -> bool,
    output_member: spec_fn(Z) -> bool,
    input_close: spec_fn(X, X, DI) -> bool,
    middle_close: spec_fn(Y, Y, DX) -> bool,
    output_close: spec_fn(Z, Z, DO) -> bool,
    middle_is_infinite: spec_fn(DX) -> bool,
    output_is_infinite: spec_fn(DO) -> bool,
    inner: spec_fn(X) -> Result<Y, E>,
    outer: spec_fn(Y) -> Result<Z, E>,
    inner_map: spec_fn(DI) -> Result<DX, ME>,
    outer_map: spec_fn(DX) -> Result<DO, ME>,
)
    requires
        valid_transformation(
            input_member, middle_member, input_close, middle_close,
            middle_is_infinite, inner, inner_map),
        valid_transformation(
            middle_member, output_member, middle_close, output_close,
            output_is_infinite, outer, outer_map),
        preserves_infinity(middle_is_infinite, output_is_infinite, outer_map),
    ensures
        valid_transformation(
            input_member, output_member, input_close, output_close,
            output_is_infinite,
            |x: X| chain_function(inner, outer, x),
            |d: DI| chain_map(inner_map, outer_map, d)),
{
    assert forall|x: X|
        #[trigger] input_member(x)
        implies match chain_function(inner, outer, x) {
            Ok(value) => output_member(value),
            Err(_) => true,
        } by {
        match inner(x) {
            Ok(middle) => {
                assert(middle_member(middle));
                match outer(middle) {
                    Ok(output) => { assert(output_member(output)); },
                    Err(_) => {},
                }
            },
            Err(_) => {},
        }
    }
    assert forall|x: X, x_prime: X, d_in: DI|
        input_member(x) && input_member(x_prime)
            && #[trigger] input_close(x, x_prime, d_in)
        implies match chain_map(inner_map, outer_map, d_in) {
            Ok(d_out) => result_close(
                chain_function(inner, outer, x),
                chain_function(inner, outer, x_prime),
                d_out, output_close, output_is_infinite),
            Err(_) => true,
        } by {
        match inner_map(d_in) {
            Err(_) => {},
            Ok(d_middle) => {
                match outer_map(d_middle) {
                    Err(_) => {},
                    Ok(d_out) => {
                        assert(result_close(
                            inner(x), inner(x_prime), d_middle,
                            middle_close, middle_is_infinite));
                        if middle_is_infinite(d_middle) {
                            assert(output_is_infinite(d_out));
                        } else {
                            match (inner(x), inner(x_prime)) {
                                (Ok(y), Ok(y_prime)) => {
                                    assert(middle_close(y, y_prime, d_middle));
                                    assert(middle_member(y));
                                    assert(middle_member(y_prime));
                                    assert(result_close(
                                        outer(y), outer(y_prime), d_out,
                                        output_close, output_is_infinite));
                                },
                                (Err(e), Err(e_prime)) => {
                                    assert(e == e_prime);
                                },
                                _ => { assert(false); },
                            }
                        }
                    },
                }
            },
        }
    }
}

/// When the intermediate metric has finite u32 distances, chaining two
/// valid models and their realized callbacks produces a valid native chain.
/// This combines the general stability theorem with callback-to-model
/// correspondence before dynamic closure erasure.
pub proof fn lemma_make_chain_tt_certified_u32<X, Y, Z, DI, E, ME>(
    input_member: spec_fn(X) -> bool,
    middle_member: spec_fn(Y) -> bool,
    output_member: spec_fn(Z) -> bool,
    input_close: spec_fn(X, X, DI) -> bool,
    middle_close: spec_fn(Y, Y, u32) -> bool,
    output_close: spec_fn(Z, Z, u32) -> bool,
    inner_model: spec_fn(X) -> Result<Y, E>,
    outer_model: spec_fn(Y) -> Result<Z, E>,
    inner_map_model: spec_fn(DI) -> Result<u32, ME>,
    outer_map_model: spec_fn(u32) -> Result<u32, ME>,
    inner: spec_fn(X) -> Result<Y, E>,
    outer: spec_fn(Y) -> Result<Z, E>,
    inner_map: spec_fn(DI) -> Result<u32, ME>,
    outer_map: spec_fn(u32) -> Result<u32, ME>,
)
    requires
        valid_transformation(
            input_member, middle_member, input_close, middle_close,
            |_: u32| false, inner_model, inner_map_model),
        valid_transformation(
            middle_member, output_member, middle_close, output_close,
            |_: u32| false, outer_model, outer_map_model),
        realizes_callback(inner, inner_model),
        realizes_callback(outer, outer_model),
        realizes_callback(inner_map, inner_map_model),
        realizes_callback(outer_map, outer_map_model),
    ensures
        valid_transformation(
            input_member, output_member, input_close, output_close,
            |_: u32| false,
            |x: X| chain_function(inner, outer, x),
            |d: DI| chain_map(inner_map, outer_map, d)),
{
    assert(preserves_infinity(
        |_: u32| false, |_: u32| false, outer_map_model,
    ));
    lemma_make_chain_tt(
        input_member, middle_member, output_member, input_close,
        middle_close, output_close,
        |_: u32| false, |_: u32| false,
        inner_model, outer_model, inner_map_model, outer_map_model,
    );
    assert(realizes_callback(
        |x: X| chain_function(inner, outer, x),
        |x: X| chain_function(inner_model, outer_model, x),
    )) by {
        assert forall|x: X|
            #[trigger] chain_function(inner, outer, x)
                == chain_function(inner_model, outer_model, x) by {
            assert(inner(x) == inner_model(x));
            match inner_model(x) {
                Ok(mid) => { assert(outer(mid) == outer_model(mid)); },
                Err(_) => {},
            }
        }
    }
    assert(realizes_callback(
        |d: DI| chain_map(inner_map, outer_map, d),
        |d: DI| chain_map(inner_map_model, outer_map_model, d),
    )) by {
        assert forall|d: DI|
            #[trigger] chain_map(inner_map, outer_map, d)
                == chain_map(inner_map_model, outer_map_model, d) by {
            assert(inner_map(d) == inner_map_model(d));
            match inner_map_model(d) {
                Ok(mid) => { assert(outer_map(mid) == outer_map_model(mid)); },
                Err(_) => {},
            }
        }
    }
    lemma_new_certified(
        input_member, output_member, input_close, output_close,
        |_: u32| false,
        |x: X| chain_function(inner_model, outer_model, x),
        |d: DI| chain_map(inner_map_model, outer_map_model, d),
        |x: X| chain_function(inner, outer, x),
        |d: DI| chain_map(inner_map, outer_map, d),
    );
}

} // verus!

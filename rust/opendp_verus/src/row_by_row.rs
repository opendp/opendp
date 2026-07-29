//! Stability proof for OpenDP's `make_row_by_row_fallible`.
//!
//! A matching identifies records that survive an edit from one dataset to the
//! other. Ordered metrics restrict the matching to increasing index pairs;
//! unordered metrics do not. Bounded metrics require a full matching; unbounded
//! metrics permit unmatched records. Those restrictions are orthogonal to the
//! theorem below: applying the same deterministic row function preserves every
//! matching and cannot turn an equal matched pair into an unequal pair.

use vstd::prelude::*;

verus! {

/// A pair of input indices in a matching.
pub type Match = (int, int);

/// Every matched pair indexes a record in each input dataset.
pub open spec fn matching_in_bounds<A, B>(
    left: Seq<A>,
    right: Seq<B>,
    matching: Seq<Match>,
) -> bool {
    forall |k: int| 0 <= k < matching.len() ==> {
        let pair = #[trigger] matching[k];
        &&& 0 <= pair.0 < left.len()
        &&& 0 <= pair.1 < right.len()
    }
}

/// No input or output record participates in more than one matched pair.
pub open spec fn matching_injective(matching: Seq<Match>) -> bool {
    forall |i: int, j: int| 0 <= i < j < matching.len() ==> {
        &&& (#[trigger] matching[i]).0 != (#[trigger] matching[j]).0
        &&& matching[i].1 != matching[j].1
    }
}

/// The matching respects record order.
pub open spec fn matching_ordered(matching: Seq<Match>) -> bool {
    forall |i: int, j: int| 0 <= i < j < matching.len() ==> {
        &&& (#[trigger] matching[i]).0 < (#[trigger] matching[j]).0
        &&& matching[i].1 < matching[j].1
    }
}

/// Every record on each side participates in the matching.
pub open spec fn matching_full<A, B>(
    left: Seq<A>,
    right: Seq<B>,
    matching: Seq<Match>,
) -> bool {
    &&& matching.len() == left.len()
    &&& matching.len() == right.len()
    &&& matching_in_bounds(left, right, matching)
    &&& matching_injective(matching)
}

/// Every record is matched to the record at the same position.
pub open spec fn matching_positional<A, B>(
    left: Seq<A>,
    right: Seq<B>,
    matching: Seq<Match>,
) -> bool {
    &&& matching_full(left, right, matching)
    &&& forall |k: int| 0 <= k < matching.len()
        ==> #[trigger] matching[k] == (k, k)
}

/// Number of matched pairs whose row values differ.
pub open spec fn mismatch_count<A>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
) -> nat
    decreases matching.len(),
{
    if matching.len() == 0 {
        0
    } else {
        let pair = matching.first();
        let mismatch = if left[pair.0] == right[pair.1] { 0nat } else { 1nat };
        mismatch + mismatch_count(left, right, matching.drop_first())
    }
}

/// Cost induced by a matching.
///
/// `substitution_cost = 2` models symmetric and insert-delete distance.
/// `substitution_cost = 1`, together with the metric's full-matching
/// restriction, models change-one and Hamming distance.
pub open spec fn matching_cost<A>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
    substitution_cost: nat,
) -> nat {
    (left.len() - matching.len()) as nat
        + (right.len() - matching.len()) as nat
        + substitution_cost * mismatch_count(left, right, matching)
}

proof fn lemma_mul_monotonic(a: nat, b: nat, factor: nat)
    requires a <= b,
    ensures factor * a <= factor * b,
    decreases factor,
{
    if factor > 0 {
        lemma_mul_monotonic(a, b, (factor - 1) as nat);
        assert((factor - 1) * a <= (factor - 1) * b);
        assert(factor * a == (factor - 1) * a + a) by (nonlinear_arith);
        assert(factor * b == (factor - 1) * b + b) by (nonlinear_arith);
        assert(factor * a <= factor * b);
    } else {
        assert(factor == 0);
        assert(factor * a == 0);
        assert(factor * b == 0);
    }
    assert(factor * a <= factor * b);
}

/// All rows successfully evaluate under the fallible row function.
pub open spec fn all_succeed<A, B>(
    input: Seq<A>,
    row_function: spec_fn(A) -> Option<B>,
) -> bool {
    forall |i: int| 0 <= i < input.len() ==> (#[trigger] row_function(input[i])).is_some()
}

/// Output of a successful fallible row-wise map.
pub open spec fn successful_map<A, B>(
    input: Seq<A>,
    row_function: spec_fn(A) -> Option<B>,
) -> Seq<B> {
    Seq::new(input.len(), |i: int| row_function(input[i]).unwrap())
}

proof fn lemma_successful_map_len<A, B>(
    input: Seq<A>,
    row_function: spec_fn(A) -> Option<B>,
)
    ensures successful_map(input, row_function).len() == input.len(),
{
}

proof fn lemma_successful_map_index<A, B>(
    input: Seq<A>,
    row_function: spec_fn(A) -> Option<B>,
    i: int,
)
    requires
        0 <= i < input.len(),
        all_succeed(input, row_function),
    ensures
        row_function(input[i]) == Some(successful_map(input, row_function)[i]),
{
}

/// A deterministic fallible row function preserves equality on successful rows.
proof fn lemma_equal_rows_stay_equal<A, B>(
    left: Seq<A>,
    right: Seq<A>,
    row_function: spec_fn(A) -> Option<B>,
    left_index: int,
    right_index: int,
)
    requires
        0 <= left_index < left.len(),
        0 <= right_index < right.len(),
        all_succeed(left, row_function),
        all_succeed(right, row_function),
        left[left_index] == right[right_index],
    ensures
        successful_map(left, row_function)[left_index]
            == successful_map(right, row_function)[right_index],
{
    lemma_successful_map_index(left, row_function, left_index);
    lemma_successful_map_index(right, row_function, right_index);
}

/// Row-wise mapping cannot increase the mismatches in any valid matching.
pub proof fn lemma_mismatch_count_nonexpansive<A, B>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
    row_function: spec_fn(A) -> Option<B>,
)
    requires
        matching_in_bounds(left, right, matching),
        all_succeed(left, row_function),
        all_succeed(right, row_function),
    ensures
        mismatch_count(
            successful_map(left, row_function),
            successful_map(right, row_function),
            matching,
        ) <= mismatch_count(left, right, matching),
    decreases matching.len(),
{
    if matching.len() > 0 {
        let pair = matching.first();
        if left[pair.0] == right[pair.1] {
            lemma_equal_rows_stay_equal(
                left,
                right,
                row_function,
                pair.0,
                pair.1,
            );
        }

        assert(matching_in_bounds(left, right, matching.drop_first()));
        lemma_mismatch_count_nonexpansive(
            left,
            right,
            matching.drop_first(),
            row_function,
        );
    }
}

/// The stability-1 theorem used by `make_row_by_row_fallible`.
///
/// Any matching that witnesses input distance at most `d_in` also witnesses
/// output distance at most `d_in`. Metric-specific matching restrictions remain
/// valid because the output has exactly the same number and order of rows.
pub proof fn lemma_make_row_by_row_fallible_stable<A, B>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
    substitution_cost: nat,
    d_in: nat,
    row_function: spec_fn(A) -> Option<B>,
)
    requires
        matching.len() <= left.len(),
        matching.len() <= right.len(),
        matching_in_bounds(left, right, matching),
        all_succeed(left, row_function),
        all_succeed(right, row_function),
        matching_cost(left, right, matching, substitution_cost) <= d_in,
    ensures
        matching_cost(
            successful_map(left, row_function),
            successful_map(right, row_function),
            matching,
            substitution_cost,
        ) <= d_in,
{
    lemma_successful_map_len(left, row_function);
    lemma_successful_map_len(right, row_function);
    lemma_mismatch_count_nonexpansive(left, right, matching, row_function);

    let mapped_left = successful_map(left, row_function);
    let mapped_right = successful_map(right, row_function);
    let input_mismatches = mismatch_count(left, right, matching);
    let output_mismatches = mismatch_count(mapped_left, mapped_right, matching);

    assert(mapped_left.len() == left.len());
    assert(mapped_right.len() == right.len());
    assert(output_mismatches <= input_mismatches);
    lemma_mul_monotonic(output_mismatches, input_mismatches, substitution_cost);
    assert(
        matching_cost(mapped_left, mapped_right, matching, substitution_cost)
            == (left.len() - matching.len()) as nat
                + (right.len() - matching.len()) as nat
                + substitution_cost * output_mismatches
    );
    assert(
        matching_cost(left, right, matching, substitution_cost)
            == (left.len() - matching.len()) as nat
                + (right.len() - matching.len()) as nat
                + substitution_cost * input_mismatches
    );
    assert(
        matching_cost(mapped_left, mapped_right, matching, substitution_cost)
            <= matching_cost(left, right, matching, substitution_cost)
    );
}

/// Stability 1 under OpenDP's unordered, unbounded symmetric distance.
pub proof fn lemma_stable_symmetric_distance<A, B>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
    d_in: nat,
    row_function: spec_fn(A) -> Option<B>,
)
    requires
        matching.len() <= left.len(),
        matching.len() <= right.len(),
        matching_in_bounds(left, right, matching),
        matching_injective(matching),
        all_succeed(left, row_function),
        all_succeed(right, row_function),
        matching_cost(left, right, matching, 2) <= d_in,
    ensures
        matching_cost(
            successful_map(left, row_function),
            successful_map(right, row_function),
            matching,
            2,
        ) <= d_in,
{
    lemma_make_row_by_row_fallible_stable(
        left, right, matching, 2, d_in, row_function,
    );
}

/// Stability 1 under OpenDP's ordered, unbounded insert-delete distance.
pub proof fn lemma_stable_insert_delete_distance<A, B>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
    d_in: nat,
    row_function: spec_fn(A) -> Option<B>,
)
    requires
        matching.len() <= left.len(),
        matching.len() <= right.len(),
        matching_in_bounds(left, right, matching),
        matching_injective(matching),
        matching_ordered(matching),
        all_succeed(left, row_function),
        all_succeed(right, row_function),
        matching_cost(left, right, matching, 2) <= d_in,
    ensures
        matching_cost(
            successful_map(left, row_function),
            successful_map(right, row_function),
            matching,
            2,
        ) <= d_in,
{
    lemma_make_row_by_row_fallible_stable(
        left, right, matching, 2, d_in, row_function,
    );
}

/// Stability 1 under OpenDP's unordered, bounded change-one distance.
pub proof fn lemma_stable_change_one_distance<A, B>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
    d_in: nat,
    row_function: spec_fn(A) -> Option<B>,
)
    requires
        matching_full(left, right, matching),
        all_succeed(left, row_function),
        all_succeed(right, row_function),
        matching_cost(left, right, matching, 1) <= d_in,
    ensures
        matching_cost(
            successful_map(left, row_function),
            successful_map(right, row_function),
            matching,
            1,
        ) <= d_in,
{
    lemma_make_row_by_row_fallible_stable(
        left, right, matching, 1, d_in, row_function,
    );
}

/// Stability 1 under OpenDP's ordered, bounded Hamming distance.
pub proof fn lemma_stable_hamming_distance<A, B>(
    left: Seq<A>,
    right: Seq<A>,
    matching: Seq<Match>,
    d_in: nat,
    row_function: spec_fn(A) -> Option<B>,
)
    requires
        matching_positional(left, right, matching),
        all_succeed(left, row_function),
        all_succeed(right, row_function),
        matching_cost(left, right, matching, 1) <= d_in,
    ensures
        matching_cost(
            successful_map(left, row_function),
            successful_map(right, row_function),
            matching,
            1,
        ) <= d_in,
{
    lemma_make_row_by_row_fallible_stable(
        left, right, matching, 1, d_in, row_function,
    );
}

}

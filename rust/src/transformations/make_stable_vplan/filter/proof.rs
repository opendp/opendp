//! Executable filter node and row-domain preservation certificate.

use super::verified::{self, VExpr, VField, VPlan, VValue};
use super::stability;
use vstd::prelude::*;
use vstd::multiset::Multiset;

verus! {

/// Adding an input row cannot invalidate an existing submultiset bound.
/// The lemma handles both retaining and dropping that row, with multiplicity.
pub proof fn lemma_subset_insert(
    small: Multiset<Seq<Option<VValue>>>,
    big: Multiset<Seq<Option<VValue>>>,
    row: Seq<Option<VValue>>,
)
    requires small.subset_of(big),
    ensures
        small.subset_of(big.insert(row)),
        small.insert(row).subset_of(big.insert(row)),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    assert forall|value: Seq<Option<VValue>>|
        small.count(value) <= big.insert(row).count(value) by {
        assert(small.count(value) <= big.count(value));
    }
    assert forall|value: Seq<Option<VValue>>|
        small.insert(row).count(value) <= big.insert(row).count(value) by {
        assert(small.count(value) <= big.count(value));
    }
}

/// A filter retains individual input rows; it never duplicates them.
pub proof fn lemma_filter_submultiset(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(rows, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match verified::apply_spec(plan, rows) {
            Ok(out) => out.to_multiset().subset_of(rows.to_multiset()),
            Err(_) => false,
        },
    decreases rows.len(),
{
    verified::lemma_apply_prefix_type_sound(plan, rows, schema, schema, rows.len());
    if rows.len() > 0 {
        stability::lemma_rows_drop_last_valid(rows, schema);
        let before = rows.drop_last();
        let row = rows.last();
        lemma_filter_submultiset(plan, before, schema);
        stability::lemma_apply_prefix_ignores_last(plan, rows, before.len());
        let previous = verified::apply_spec(plan, before).unwrap();
        let old_bag = previous.to_multiset();
        let input_bag = before.to_multiset();
        assert(old_bag.subset_of(input_bag));
        before.to_multiset_ensures();
        assert(rows =~= before.push(row));
        assert(rows.to_multiset() =~= input_bag.insert(row));
        previous.to_multiset_ensures();
        match plan {
            VPlan::Filter { predicate, .. } => {
                match verified::filter::row_spec(predicate, row) {
                    Ok(Some(retained)) => {
                        assert(retained == row);
                        assert(verified::apply_spec(plan, rows) == Ok(previous.push(row)));
                        lemma_subset_insert(old_bag, input_bag, row);
                        assert(previous.push(row).to_multiset() =~= old_bag.insert(row));
                    },
                    Ok(None) => {
                        assert(verified::apply_spec(plan, rows) == Ok(previous));
                        lemma_subset_insert(old_bag, input_bag, row);
                    },
                    Err(_) => { assert(false); },
                }
            },
            _ => { assert(false); },
        }
    } else {
        assert(rows =~= Seq::empty());
        assert(verified::apply_spec(plan, rows) == Ok(rows));
    }
}

/// The node certificate retains multiplicities as well as row constraints.
pub open spec fn preserves_filter_multiset(plan: &VPlan, schema: Seq<VField>) -> bool {
    forall|rows: Seq<Seq<Option<VValue>>>|
        verified::rows_in_schema(rows, schema)
        ==> match verified::apply_spec(plan, rows) {
            Ok(out) => out.to_multiset().subset_of(rows.to_multiset()),
            Err(_) => false,
        }
}

/// On each well-typed row value, filtering either retains every copy
/// or drops every copy. This is stronger than submultiset containment.
pub proof fn lemma_filter_exact_count(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    value: Seq<Option<VValue>>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(rows, schema),
        verified::row_in_schema(value, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match verified::apply_spec(plan, rows) {
            Ok(out) => {
                let selected = verified::local_row_spec(plan, value) == Ok(Some(value));
                out.to_multiset().count(value) ==
                    if selected { rows.to_multiset().count(value) } else { 0 }
            },
            Err(_) => false,
        },
    decreases rows.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    verified::lemma_apply_prefix_type_sound(plan, rows, schema, schema, rows.len());
    verified::lemma_local_type_sound(plan, schema, value, schema);
    if rows.len() == 0 {
        let out = verified::apply_spec(plan, rows).unwrap();
        assert(out.len() == 0);
        out.to_multiset_ensures();
        rows.to_multiset_ensures();
        assert(out.to_multiset().count(value) == 0);
        assert(rows.to_multiset().count(value) == 0);
    } else {
        stability::lemma_rows_drop_last_valid(rows, schema);
        let previous = rows.drop_last();
        let last = rows.last();
        lemma_filter_exact_count(plan, previous, schema, value);
        stability::lemma_apply_prefix_ignores_last(plan, rows, previous.len());
        assert(rows =~= previous.push(last));
        let old_out = verified::apply_spec(plan, previous).unwrap();
        previous.to_multiset_ensures();
        old_out.to_multiset_ensures();
        assert(rows.to_multiset() =~= previous.to_multiset().insert(last));
        match plan {
            VPlan::Filter { predicate, .. } => {
                match verified::filter::row_spec(predicate, last) {
                    Ok(Some(retained)) => {
                        assert(retained == last);
                        assert(verified::apply_spec(plan, rows) == Ok(old_out.push(last)));
                        old_out.to_multiset_ensures();
                        assert(old_out.push(last).to_multiset()
                            =~= old_out.to_multiset().insert(last));
                    },
                    Ok(None) => {
                        assert(verified::apply_spec(plan, rows) == Ok(old_out));
                    },
                    Err(_) => { assert(false); },
                }
            },
            _ => { assert(false); },
        }
        if last == value {
            assert(verified::local_row_spec(plan, last)
                == verified::local_row_spec(plan, value));
        }
        assert(verified::apply_spec(plan, rows).is_Ok());
    }
}

/// Pointwise contraction of the multiplicity difference for two datasets.
/// Each row's grouping key is unchanged by filtering, so this is the
/// per-row algebra underlying every grouped symmetric-distance bound.
pub proof fn lemma_filter_count_contraction(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    value: Seq<Option<VValue>>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::row_in_schema(value, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) => {
                let a = left.to_multiset().count(value) as int;
                let b = right.to_multiset().count(value) as int;
                let x = out_left.to_multiset().count(value) as int;
                let y = out_right.to_multiset().count(value) as int;
                (if x >= y { x - y } else { y - x })
                    <= (if a >= b { a - b } else { b - a })
            },
            _ => false,
        },
{
    lemma_filter_exact_count(plan, left, schema, value);
    lemma_filter_exact_count(plan, right, schema, value);
}

/// A finite, possibly overcomplete enumeration of row values. A distinct
/// enumeration of the joint support gives the usual per-group L1 distance.
pub open spec fn group_distance(
    left: Multiset<Seq<Option<VValue>>>,
    right: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    group: Seq<Option<VValue>>,
) -> nat
    decreases values.len(),
{
    if values.len() == 0 { 0 }
    else {
        let row = values.last();
        let a = left.count(row);
        let b = right.count(row);
        group_distance(left, right, values.drop_last(), group_by, group)
            + if group_by(row) == group {
                (if a >= b { a - b } else { b - a }) as nat
              } else { 0nat }
    }
}

/// Sum of indicators for changed groups. A distinct enumeration of all
/// possible group keys gives the `num_groups` component of FrameDistance.
pub open spec fn changed_groups(
    left: Multiset<Seq<Option<VValue>>>,
    right: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    groups: Seq<Seq<Option<VValue>>>,
) -> nat
    decreases groups.len(),
{
    if groups.len() == 0 { 0 }
    else {
        changed_groups(left, right, values, group_by, groups.drop_last())
            + if group_distance(left, right, values, group_by, groups.last()) > 0 {
                1nat
              } else { 0nat }
    }
}

/// Both insertions and deletions are accounted for by the absolute
/// difference of row multiplicities within each group.
pub proof fn lemma_group_distance_contraction(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    group: Seq<Option<VValue>>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) =>
                group_distance(out_left.to_multiset(), out_right.to_multiset(),
                    values, group_by, group)
                <= group_distance(left.to_multiset(), right.to_multiset(),
                    values, group_by, group),
            _ => false,
        },
    decreases values.len(),
{
    if values.len() > 0 {
        stability::lemma_rows_drop_last_valid(values, schema);
        lemma_group_distance_contraction(plan, left, right, schema,
            values.drop_last(), group_by, group);
        lemma_filter_count_contraction(plan, left, right, schema, values.last());
    } else {
        verified::lemma_apply_prefix_type_sound(plan, left, schema, schema, left.len());
        verified::lemma_apply_prefix_type_sound(plan, right, schema, schema, right.len());
    }
}

/// A filter cannot create a changed group: the support of nonzero group
/// distances is contained in the original support.
pub proof fn lemma_changed_groups_contraction(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    groups: Seq<Seq<Option<VValue>>>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) =>
                changed_groups(out_left.to_multiset(), out_right.to_multiset(),
                    values, group_by, groups)
                <= changed_groups(left.to_multiset(), right.to_multiset(),
                    values, group_by, groups),
            _ => false,
        },
    decreases groups.len(),
{
    if groups.len() > 0 {
        lemma_changed_groups_contraction(plan, left, right, schema,
            values, group_by, groups.drop_last());
        lemma_group_distance_contraction(plan, left, right, schema,
            values, group_by, groups.last());
    } else {
        verified::lemma_apply_prefix_type_sound(plan, left, schema, schema, left.len());
        verified::lemma_apply_prefix_type_sound(plan, right, schema, schema, right.len());
    }
}


/// Logical interpretation of one `Bound<VExpr>` once its grouping expressions
/// are interpreted as a deterministic row-local function. `values` must
/// enumerate the distinct joint row support and `groups` the distinct group
/// keys for this to equal the concrete FrameDistance components.
pub open spec fn within_group_bound(
    left: Multiset<Seq<Option<VValue>>>,
    right: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
) -> bool {
    &&& match per_group {
        Some(limit) => forall|key: Seq<Option<VValue>>|
            group_distance(left, right, values, group_by, key) <= limit as nat,
        None => true,
    }
    &&& match num_groups {
        Some(limit) => changed_groups(left, right, values, group_by, groups) <= limit as nat,
        None => true,
    }
}

/// The identity map on an individual Bound preserves both optional limits.
/// The same argument applies pointwise to every element of Bounds<VExpr>.
pub proof fn lemma_filter_preserves_group_bound(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
        within_group_bound(left.to_multiset(), right.to_multiset(),
            values, group_by, groups, per_group, num_groups),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) =>
                within_group_bound(out_left.to_multiset(), out_right.to_multiset(),
                    values, group_by, groups, per_group, num_groups),
            _ => false,
        },
{
    lemma_changed_groups_contraction(plan, left, right, schema,
        values, group_by, groups);
    if let Some(limit) = per_group {
        assert forall|key: Seq<Option<VValue>>|
            group_distance(
                verified::apply_spec(plan, left).unwrap().to_multiset(),
                verified::apply_spec(plan, right).unwrap().to_multiset(),
                values, group_by, key) <= limit as nat by {
            lemma_group_distance_contraction(plan, left, right, schema,
                values, group_by, key);
        }
    }
}

/// Specialize the grouped-distance certificate to an executable sequence
/// of well-typed VExpr keys. The kernel's verified project::eval_prefix
/// implements exactly project::row_prefix, which supplies this grouping.
pub proof fn lemma_filter_preserves_expr_group_bound(
    plan: &VPlan, left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    keys: Seq<VExpr>, values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(left, schema),
        verified::rows_in_schema(right, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
        verified::project::schema_prefix(keys, schema, keys.len()).is_Some(),
        within_group_bound(left.to_multiset(), right.to_multiset(), values,
            |row: Seq<Option<VValue>>|
                verified::project::row_prefix(keys, row, keys.len()).unwrap(),
            groups, per_group, num_groups),
    ensures
        match (verified::apply_spec(plan, left), verified::apply_spec(plan, right)) {
            (Ok(out_left), Ok(out_right)) =>
                within_group_bound(out_left.to_multiset(), out_right.to_multiset(),
                    values,
                    |row: Seq<Option<VValue>>|
                        verified::project::row_prefix(keys, row, keys.len()).unwrap(),
                    groups, per_group, num_groups),
            _ => false,
        },
{
    // Every typed row has a total grouping key. The same kernel function
    // evaluates keys on both sides of the symmetric distance.
    assert forall|row: Seq<Option<VValue>>| verified::row_in_schema(row, schema)
        implies verified::project::row_prefix(keys, row, keys.len()).is_Ok() by {
        verified::project::lemma_row_prefix_type_sound(
            keys, schema, row, keys.len());
    }
    let by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>> =
        |row: Seq<Option<VValue>>|
            verified::project::row_prefix(keys, row, keys.len()).unwrap();
    lemma_filter_preserves_group_bound(
        plan, left, right, schema, values, by, groups, per_group, num_groups);
}

pub fn construct(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        stability::certified_result(result, source_schema@),
        match result {
            Some((plan, fields)) =>
                preserves_filter_multiset(&plan, fields@)
                && verified::local_schema(&plan, fields@) == Some(fields@)
                && (forall|left: Seq<Seq<Option<VValue>>>,
                    right: Seq<Seq<Option<VValue>>>, value: Seq<Option<VValue>>|
                    verified::rows_in_schema(left, fields@)
                    && verified::rows_in_schema(right, fields@)
                    && verified::row_in_schema(value, fields@)
                    ==> match (verified::apply_spec(&plan, left),
                               verified::apply_spec(&plan, right)) {
                        (Ok(out_left), Ok(out_right)) => {
                            let a = left.to_multiset().count(value) as int;
                            let b = right.to_multiset().count(value) as int;
                            let x = out_left.to_multiset().count(value) as int;
                            let y = out_right.to_multiset().count(value) as int;
                            (if x >= y { x - y } else { y - x })
                                <= (if a >= b { a - b } else { b - a })
                        },
                        _ => false,
                    }),
            None => true,
        },
{
    let plan = VPlan::Filter { input: Box::new(input), predicate };
    match verified::check_plan(&plan, source_schema) {
        None => None,
        Some(fields) => {
            proof {
                stability::lemma_all_typed_edits_stable(&plan, source_schema@);
                assert(verified::local_schema(&plan, fields@) == Some(fields@));
                assert forall|rows: Seq<Seq<Option<VValue>>>|
                    verified::rows_in_schema(rows, fields@)
                    implies match verified::apply_spec(&plan, rows) {
                        Ok(out) => out.to_multiset().subset_of(rows.to_multiset()),
                        Err(_) => false,
                    } by {
                    lemma_filter_submultiset(&plan, rows, fields@);
                }
                assert forall|left: Seq<Seq<Option<VValue>>>,
                    right: Seq<Seq<Option<VValue>>>, value: Seq<Option<VValue>>|
                    verified::rows_in_schema(left, fields@)
                    && verified::rows_in_schema(right, fields@)
                    && verified::row_in_schema(value, fields@)
                    implies match (verified::apply_spec(&plan, left),
                                   verified::apply_spec(&plan, right)) {
                        (Ok(out_left), Ok(out_right)) => {
                            let a = left.to_multiset().count(value) as int;
                            let b = right.to_multiset().count(value) as int;
                            let x = out_left.to_multiset().count(value) as int;
                            let y = out_right.to_multiset().count(value) as int;
                            (if x >= y { x - y } else { y - x })
                                <= (if a >= b { a - b } else { b - a })
                        },
                        _ => false,
                    } by {
                    lemma_filter_count_contraction(&plan, left, right, fields@, value);
                }
            }
            Some((plan, fields))
        },
    }
}

} // verus!

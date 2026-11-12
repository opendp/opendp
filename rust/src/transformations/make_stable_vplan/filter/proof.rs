//! Executable filter node and row-domain preservation certificate.

use super::verified::{self, VExpr, VField, VPlan, VTable, VValue};
use super::verified::domain::{self, VColumnRule};
use super::verified::transformation;
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

/// A filter never introduces additional copies of a row.
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
{
    if rows.len() > 0 {
        assert(verified::row_in_schema(rows[0], schema));
        lemma_filter_exact_count(plan, rows, schema, rows[0]);
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

/// The primary filter certificate: every copy of a row is either
/// retained or discarded, according only to that row.
pub open spec fn preserves_filter_counts(
    plan: &VPlan, schema: Seq<VField>,
) -> bool {
    forall|rows: Seq<Seq<Option<VValue>>>, value: Seq<Option<VValue>>|
        verified::rows_in_schema(rows, schema)
        && verified::row_in_schema(value, schema)
        ==> match verified::apply_spec(plan, rows) {
            Ok(out) =>
                out.to_multiset().count(value) ==
                    if verified::local_row_spec(plan, value) == Ok(Some(value)) {
                        rows.to_multiset().count(value)
                    } else { 0 },
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
            Ok(out) =>
                out.to_multiset().subset_of(rows.to_multiset())
                && (out.to_multiset().count(value) ==
                    if verified::local_row_spec(plan, value) == Ok(Some(value)) {
                        rows.to_multiset().count(value)
                    } else { 0 }),

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
        assert(old_out.to_multiset().subset_of(previous.to_multiset()));
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
                        lemma_subset_insert(old_out.to_multiset(), previous.to_multiset(), last);
                    },
                    Ok(None) => {
                        assert(verified::apply_spec(plan, rows) == Ok(old_out));
                        lemma_subset_insert(old_out.to_multiset(), previous.to_multiset(), last);
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
        assert(verified::apply_spec(plan, rows).unwrap().to_multiset()
            .subset_of(rows.to_multiset()));
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
    lemma_changed_groups_contraction(
        plan, left, right, schema, values, by, groups);
    if let Some(limit) = per_group {
        assert forall|key: Seq<Option<VValue>>|
            group_distance(
                verified::apply_spec(plan, left).unwrap().to_multiset(),
                verified::apply_spec(plan, right).unwrap().to_multiset(),
                values, by, key) <= limit as nat by {
            lemma_group_distance_contraction(
                plan, left, right, schema, values, by, key);
        }
    }
}

/// Row-wise column restrictions can be represented by an allowed set of
/// complete rows. Removing rows preserves every such restriction.
pub open spec fn allowed_rows(
    rows: Multiset<Seq<Option<VValue>>>,
    allowed: Set<Seq<Option<VValue>>>,
) -> bool {
    forall|row: Seq<Option<VValue>>|
        rows.count(row) > 0 ==> allowed.contains(row)
}

pub proof fn lemma_allowed_rows_subset(
    smaller: Multiset<Seq<Option<VValue>>>,
    larger: Multiset<Seq<Option<VValue>>>,
    allowed: Set<Seq<Option<VValue>>>,
)
    requires
        smaller.subset_of(larger),
        allowed_rows(larger, allowed),
    ensures allowed_rows(smaller, allowed),
{
    assert forall|row: Seq<Option<VValue>>|
        smaller.count(row) > 0 implies allowed.contains(row) by {
        assert(smaller.count(row) <= larger.count(row));
    }
}

/// A fixed, distinct enumeration of row values and their grouping keys.
pub open spec fn group_universe(
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
) -> bool {
    &&& forall|i: int, j: int|
        0 <= i < j < values.len() ==> values[i] != values[j]
    &&& forall|i: int| 0 <= i < values.len() ==> exists|j: int|
        0 <= j < groups.len()
        && groups[j] == verified::project::row_prefix(
            keys, #[trigger] values[i], keys.len()).unwrap()
    &&& forall|i: int, j: int|
        0 <= i < j < groups.len() ==> groups[i] != groups[j]
}

/// The table-dependent part of completeness: every present row is enumerated.
pub open spec fn rows_covered(
    rows: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
) -> bool {
    forall|row: Seq<Option<VValue>>|
        rows.count(row) > 0 ==> exists|i: int|
            0 <= i < values.len() && values[i] == row
}

pub open spec fn complete_group_support(
    rows: Multiset<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
) -> bool {
    group_universe(keys, values, groups) && rows_covered(rows, values)
}

pub proof fn lemma_complete_group_support_subset(
    smaller: Multiset<Seq<Option<VValue>>>,
    larger: Multiset<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
)
    requires smaller.subset_of(larger),
        complete_group_support(larger, keys, values, groups),
    ensures complete_group_support(smaller, keys, values, groups),
{
    assert forall|row: Seq<Option<VValue>>|
        smaller.count(row) > 0 implies exists|i: int|
            0 <= i < values.len() && values[i] == row by {
        assert(smaller.count(row) <= larger.count(row));
    }
    assert(rows_covered(smaller, values));
    assert(group_universe(keys, values, groups));
}

/// The portion of concrete FrameDomain<VTable>::member that can constrain
/// a filtered table: row-wise column domains and upper-bound margins.
/// For an empty set of keys, the margin has one group even on empty data.
/// The finite row/key enumerations must cover the support to interpret
/// the numerical bounds as actual group counts.
pub open spec fn bounded_frame_member(
    rows: Multiset<Seq<Option<VValue>>>,
    allowed: Set<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
) -> bool {
    &&& allowed_rows(rows, allowed)
    &&& complete_group_support(rows, keys, values, groups)
    &&& (keys.len() != 0 || num_groups != Some(0u32))
    &&& within_group_bound(rows,
        Seq::<Seq<Option<VValue>>>::empty().to_multiset(),
        values,
        |row: Seq<Option<VValue>>|
            verified::project::row_prefix(keys, row, keys.len()).unwrap(),
        groups, per_group, num_groups)
}

/// Filtering preserves the concrete *kinds* of constraints retained by
/// filter_domain: column-level row restrictions, max_length, max_groups.
/// The source-relative Rust FrameDomain encoding is checked separately.
pub proof fn lemma_filter_transformation_contract(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    allowed: Set<Seq<Option<VValue>>>,
    keys: Seq<VExpr>,
    values: Seq<Seq<Option<VValue>>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(rows, schema),
        verified::rows_in_schema(values, schema),
        verified::local_schema(plan, schema) == Some(schema),
        verified::project::schema_prefix(keys, schema, keys.len()).is_Some(),
        bounded_frame_member(rows.to_multiset(), allowed,
            keys, values, groups, per_group, num_groups),
    ensures
        match verified::apply_spec(plan, rows) {
            Ok(out) => bounded_frame_member(out.to_multiset(), allowed,
                keys, values, groups, per_group, num_groups),
            Err(_) => false,
        },
{
    lemma_filter_submultiset(plan, rows, schema);
    let output = verified::apply_spec(plan, rows).unwrap();
    lemma_allowed_rows_subset(
        output.to_multiset(), rows.to_multiset(), allowed);
    lemma_complete_group_support_subset(
        output.to_multiset(), rows.to_multiset(), keys, values, groups);
    assert(verified::rows_in_schema(Seq::<Seq<Option<VValue>>>::empty(), schema));
    lemma_filter_preserves_expr_group_bound(
        plan, rows, Seq::empty(), schema, keys, values,
        groups, per_group, num_groups);
}

pub fn construct(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        stability::certified_result(result, source_schema@),
        match result {
            Some((plan, fields)) =>
                matches!(plan, VPlan::Filter { .. })
                && preserves_filter_multiset(&plan, fields@)
                && preserves_filter_counts(&plan, fields@)
                && verified::local_schema(&plan, fields@) == Some(fields@),
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
                assert forall|rows: Seq<Seq<Option<VValue>>>,
                    value: Seq<Option<VValue>>|
                    verified::rows_in_schema(rows, fields@)
                    && verified::row_in_schema(value, fields@)
                    implies match verified::apply_spec(&plan, rows) {
                        Ok(out) =>
                            out.to_multiset().count(value) ==
                                if verified::local_row_spec(&plan, value) == Ok(Some(value)) {
                                    rows.to_multiset().count(value)
                                } else { 0 },
                        Err(_) => false,
                    } by {
                    lemma_filter_exact_count(&plan, rows, fields@, value);
                }
            }
            Some((plan, fields))
        },
    }
}


/// A finite support enumeration cannot repeat rows.
pub open spec fn distinct_rows(values: Seq<Seq<Option<VValue>>>) -> bool {
    forall|i: int, j: int|
        0 <= i < j < values.len() ==> values[i] != values[j]
}

/// Global symmetric multiset closeness. The universal quantifier over
/// distinct typed lists represents the sum over finite joint support.
pub open spec fn symmetric_close(
    left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>,
    distance: u32,
    schema: Seq<VField>,
) -> bool {
    forall|values: Seq<Seq<Option<VValue>>>|
        distinct_rows(values) && verified::rows_in_schema(values, schema)
        ==> group_distance(
            left.to_multiset(), right.to_multiset(), values,
            |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
            Seq::<Option<VValue>>::empty(),
        ) <= distance as nat
}

/// The only complete plan admitted by this constructor is Filter(Input).
pub open spec fn single_filter(plan: &VPlan) -> bool {
    match plan {
        VPlan::Filter { input, .. } => matches!(**input, VPlan::Input),
        _ => false,
    }
}

pub proof fn lemma_single_filter_relation(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
)
    requires single_filter(plan),
    ensures verified::relation_spec(plan, rows) == verified::apply_spec(plan, rows),
{
    match plan {
        VPlan::Filter { input, .. } => {
            assert(matches!(**input, VPlan::Input));
            assert(verified::relation_spec(input, rows) == Ok(rows));
        },
        _ => { assert(false); },
    }
}

/// A proof-carrying executable table filter. Unlike a plan certificate,
/// this owns the actual plan passed to the verified evaluator.
pub struct CertifiedTableFilter {
    plan: VPlan,
    schema: Vec<VField>,
    rules: Vec<VColumnRule>,
}

impl CertifiedTableFilter {
    // Public contracts use closed projections, so callers cannot forge a
    // valid certificate by modifying its plan, domain rules or schema.
    pub closed spec fn rules_spec(&self) -> Seq<VColumnRule> { self.rules@ }
    pub closed spec fn schema_spec(&self) -> Seq<VField> { self.schema@ }
    pub closed spec fn output_schema(&self, input: Seq<VField>) -> Option<Seq<VField>> {
        verified::plan_schema(&self.plan, input)
    }
    pub closed spec fn result_spec(&self, rows: Seq<Seq<Option<VValue>>>)
        -> Result<Seq<Seq<Option<VValue>>>, ()>
    {
        verified::relation_spec(&self.plan, rows)
    }

    pub closed spec fn valid(&self) -> bool {
        &&& domain::schema_member(self.schema@, self.rules@)
        &&& single_filter(&self.plan)
        &&& verified::plan_schema(&self.plan, self.schema@) == Some(self.schema@)
        &&& stability::stable_on_typed_edits(&self.plan, self.schema@)
        &&& preserves_filter_multiset(&self.plan, self.schema@)
    }

    pub fn new(predicate: VExpr, rules: &Vec<VColumnRule>) -> (result: Option<Self>)
        ensures match result {
            Some(filter) => filter.valid() && filter.valid_as_transformation()
                && filter.rules_spec() == rules@,
            None => true,
        },
    {
        let schema = domain::rule_schema(rules);
        let fields = match verified::filter::check_schema(&predicate, &schema) {
            Some(fields) => fields,
            None => return None,
        };
        let plan = VPlan::Filter {
            input: Box::new(VPlan::Input), predicate,
        };
        {
                proof {
                    match &plan {
                        VPlan::Filter { input, predicate } => {
                            assert(verified::filter::schema(predicate, schema@)
                                == Some(fields@));
                            assert(matches!(**input, VPlan::Input));
                            assert(verified::plan_schema(input, schema@)
                                == Some(schema@));
                            assert(verified::local_schema(&plan, schema@)
                                == Some(schema@));
                        },
                        _ => { assert(false); },
                    }
                    assert(fields@ == schema@);
                    assert(verified::plan_schema(&plan, schema@) == Some(schema@));
                    stability::lemma_all_typed_edits_stable(&plan, schema@);
                    assert forall|rows: Seq<Seq<Option<VValue>>>|
                        verified::rows_in_schema(rows, schema@)
                        implies match verified::apply_spec(&plan, rows) {
                            Ok(out) => out.to_multiset().subset_of(rows.to_multiset()),
                            Err(_) => false,
                        } by {
                        lemma_filter_submultiset(&plan, rows, schema@);
                    }
                }
                let filter = Self { plan, schema: fields, rules: rules.clone() };
                proof { filter.lemma_valid_as_transformation(); }
                Some(filter)
        }
    }

    /// The implementation used by the OpenDP transformation below.
    /// Every valid input in the certified schema returns a typed submultiset.
    pub fn apply(&self, input: &VTable) -> (result: Result<VTable, ()>)
        ensures
            match result {
                Ok(output) => {
                    &&& verified::table_valid(&output)
                    &&& self.output_schema(input.schema@) == Some(output.schema@)
                    &&& self.result_spec(verified::rows_view(input.rows@))
                        == Ok(verified::rows_view(output.rows@))
                },
                Err(_) => true,
            },
            self.valid()
                && verified::table_valid(input)
                && input.schema@ == self.schema_spec()
                ==> match result {
                    Ok(output) => {
                        &&& output.schema@ == input.schema@
                        &&& verified::rows_view(output.rows@).to_multiset()
                            .subset_of(verified::rows_view(input.rows@).to_multiset())
                    },
                    Err(_) => false,
                },
            self.valid() && domain::table_member(input, self.rules_spec())
                ==> match result {
                    Ok(output) => domain::table_member(&output, self.rules_spec()),
                    Err(_) => false,
                },
    {
        if !verified::check_table(input) {
            proof {
                if self.valid() && domain::table_member(input, self.rules@) {
                    domain::lemma_table_typed(input, self.rules@);
                    assert(verified::table_valid(input));
                }
            }
            return Err(());
        }
        proof {
            if self.valid() && domain::table_member(input, self.rules@) {
                domain::lemma_unique_schema(
                    input.schema@, self.schema@, self.rules@);
            }
        }
        let fields = match verified::check_plan(&self.plan, &input.schema) {
            Some(fields) => fields,
            None => {
                proof {
                    if self.valid() && domain::table_member(input, self.rules@) {
                        assert(input.schema@ == self.schema@);
                        assert(verified::plan_schema(&self.plan, input.schema@)
                            == Some(self.schema@));
                    }
                }
                return Err(());
            },
        };
        let rows = match verified::evaluate(&self.plan, input) {
            Ok(rows) => rows,
            Err(_) => {
                proof {
                    if self.valid() && input.schema@ == self.schema@ {
                        verified::lemma_relation_type_sound(
                            &self.plan, verified::rows_view(input.rows@),
                            input.schema@, self.schema@);
                    }
                    if self.valid() && domain::table_member(input, self.rules@) {
                        assert(input.schema@ == self.schema@);
                        verified::lemma_relation_type_sound(
                            &self.plan, verified::rows_view(input.rows@),
                            input.schema@, self.schema@);
                    }
                }
                return Err(());
            },
        };
        proof {
            verified::lemma_relation_type_sound(
                &self.plan, verified::rows_view(input.rows@),
                input.schema@, fields@);
            if self.valid() && input.schema@ == self.schema@ {
                assert(verified::plan_schema(&self.plan, input.schema@)
                    == Some(self.schema@));
                assert(fields@ == self.schema@);
                assert(verified::relation_spec(
                    &self.plan, verified::rows_view(input.rows@))
                    == Ok(verified::rows_view(rows@)));
                lemma_single_filter_relation(
                    &self.plan, verified::rows_view(input.rows@));
                assert(verified::apply_spec(
                    &self.plan, verified::rows_view(input.rows@))
                    == Ok(verified::rows_view(rows@)));
                assert(verified::rows_view(rows@).to_multiset()
                    .subset_of(verified::rows_view(input.rows@).to_multiset()));
            }
        }
        let output = VTable { schema: fields, rows };
        proof {
            if self.valid() && domain::table_member(input, self.rules@) {
                assert(output.schema@ == input.schema@);
                domain::lemma_table_submultiset(input, &output, self.rules@);
            }
        }
        Ok(output)
    }

    /// The general Result-valued transformation contract. The semantic
    /// carrier is a row sequence; invoke proves its executable realization.
    pub closed spec fn valid_as_transformation(&self) -> bool {
        transformation::valid_transformation::<
            Seq<Seq<Option<VValue>>>, Seq<Seq<Option<VValue>>>,
            u32, u32, (), (),
        >(
            |rows: Seq<Seq<Option<VValue>>>|
                domain::rows_member(rows, self.rules_spec()),
            |rows: Seq<Seq<Option<VValue>>>|
                domain::rows_member(rows, self.rules_spec()),
            |left: Seq<Seq<Option<VValue>>>,
             right: Seq<Seq<Option<VValue>>>, d: u32|
                symmetric_close(left, right, d, self.schema_spec()),
            |left: Seq<Seq<Option<VValue>>>,
             right: Seq<Seq<Option<VValue>>>, d: u32|
                symmetric_close(left, right, d, self.schema_spec()),
            |d: u32| false,
            |rows: Seq<Seq<Option<VValue>>>| self.result_spec(rows),
            |d: u32| Ok(d),
        )
    }

    /// The certified filter is an instance of generic transformation validity.
    pub proof fn lemma_valid_as_transformation(&self)
        requires self.valid(),
        ensures self.valid_as_transformation(),
    {
        assert forall|rows: Seq<Seq<Option<VValue>>>|
            #[trigger] domain::rows_member(rows, self.rules_spec())
            implies match self.result_spec(rows) {
                Ok(output) => domain::rows_member(output, self.rules_spec()),
                Err(_) => true,
            } by {
            self.lemma_preserves_rules(rows);
        }
        assert forall|left: Seq<Seq<Option<VValue>>>,
                      right: Seq<Seq<Option<VValue>>>, d: u32|
            #[trigger] domain::rows_member(left, self.rules_spec())
            && #[trigger] domain::rows_member(right, self.rules_spec())
            && #[trigger] symmetric_close(left, right, d, self.schema_spec())
            implies transformation::result_close(
                self.result_spec(left), self.result_spec(right), d,
                |a: Seq<Seq<Option<VValue>>>,
                 b: Seq<Seq<Option<VValue>>>, bound: u32|
                    symmetric_close(a, b, bound, self.schema_spec()),
                |_: u32| false,
            ) by {
            self.lemma_preserves_rules(left);
            self.lemma_preserves_rules(right);
            self.lemma_symmetric_contraction(left, right, d);
        }
    }

    proof fn lemma_preserves_rules(
        &self, rows: Seq<Seq<Option<VValue>>>,
    )
        requires self.valid(), domain::rows_member(rows, self.rules_spec()),
        ensures match self.result_spec(rows) {
            Ok(out) => domain::rows_member(out, self.rules_spec()),
            Err(_) => false,
        },
    {
        domain::lemma_rows_member_typed(
            rows, self.schema@, self.rules@);
        verified::lemma_relation_type_sound(
            &self.plan, rows, self.schema@, self.schema@);
        lemma_single_filter_relation(&self.plan, rows);
        lemma_filter_submultiset(&self.plan, rows, self.schema@);
        let out = self.result_spec(rows).unwrap();
        rows.to_multiset_ensures();
        out.to_multiset_ensures();
        assert forall|row: Seq<Option<VValue>>|
            rows.to_multiset().count(row) > 0
                implies domain::row_member(row, self.rules@) by {
            assert(rows.contains(row));
            let i = choose|i: int| 0 <= i < rows.len() && rows[i] == row;
            assert(domain::row_member(rows[i], self.rules@));
        }
        domain::lemma_rows_subset(
            out.to_multiset(), rows.to_multiset(), self.rules@);
        assert forall|i: int| 0 <= i < out.len()
            implies domain::row_member(out[i], self.rules@) by {
            assert(out.contains(out[i]));
            assert(out.to_multiset().count(out[i]) > 0);
        }
    }

    proof fn lemma_symmetric_contraction(
        &self,
        left: Seq<Seq<Option<VValue>>>,
        right: Seq<Seq<Option<VValue>>>,
        distance: u32,
    )
        requires
            self.valid(),
            domain::rows_member(left, self.rules_spec()),
            domain::rows_member(right, self.rules_spec()),
            symmetric_close(left, right, distance, self.schema_spec()),
        ensures
            match (self.result_spec(left), self.result_spec(right)) {
                (Ok(a), Ok(b)) =>
                    symmetric_close(a, b, distance, self.schema_spec()),
                _ => false,
            },
    {
        domain::lemma_rows_member_typed(
            left, self.schema@, self.rules@);
        domain::lemma_rows_member_typed(
            right, self.schema@, self.rules@);
        verified::lemma_relation_type_sound(
            &self.plan, left, self.schema@, self.schema@);
        verified::lemma_relation_type_sound(
            &self.plan, right, self.schema@, self.schema@);
        lemma_single_filter_relation(&self.plan, left);
        lemma_single_filter_relation(&self.plan, right);
        let out_left = self.result_spec(left).unwrap();
        let out_right = self.result_spec(right).unwrap();
        assert forall|values: Seq<Seq<Option<VValue>>>|
            distinct_rows(values) && verified::rows_in_schema(
                values, self.schema@)
            implies group_distance(
                out_left.to_multiset(), out_right.to_multiset(), values,
                |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
                Seq::<Option<VValue>>::empty(),
            ) <= distance as nat by {
            lemma_group_distance_contraction(
                &self.plan, left, right, self.schema@, values,
                |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
                Seq::<Option<VValue>>::empty(),
            );
        }
    }

    /// The entrypoint installed in OpenDP's Function. It enforces exactly
    /// the certified input domain; valid inputs always succeed and return a
    /// member of that same domain.
    pub fn invoke(&self, input: &VTable) -> (result: Result<VTable, ()>)
        ensures
            self.valid() ==> match result {
                Ok(output) => {
                    &&& domain::table_member(input, self.rules_spec())
                    &&& domain::table_member(&output, self.rules_spec())
                    &&& self.result_spec(verified::rows_view(input.rows@))
                        == Ok(verified::rows_view(output.rows@))
                },
                Err(_) => !domain::table_member(input, self.rules_spec()),
            },
    {
        if !domain::check_table(input, &self.rules) {
            return Err(());
        }
        self.apply(input)
    }

    /// Full margin-free frame contract: filtering preserves every row-wise
    /// value restriction, and contracts the global multiset L1 distance.
    /// This proves the identity stability map for arbitrary distances, not
    /// just unit insertions and deletions.
    pub proof fn lemma_domain_and_global_stability(
        &self,
        left: Seq<Seq<Option<VValue>>>,
        right: Seq<Seq<Option<VValue>>>,
        values: Seq<Seq<Option<VValue>>>,
        allowed: Set<Seq<Option<VValue>>>,
    )
        requires
            self.valid(),
            verified::rows_in_schema(left, self.schema_spec()),
            verified::rows_in_schema(right, self.schema_spec()),
            verified::rows_in_schema(values, self.schema_spec()),
            allowed_rows(left.to_multiset(), allowed),
            allowed_rows(right.to_multiset(), allowed),
        ensures
            match (
                self.result_spec(left),
                self.result_spec(right),
            ) {
                (Ok(out_left), Ok(out_right)) => {
                    &&& verified::rows_in_schema(out_left, self.schema_spec())
                    &&& verified::rows_in_schema(out_right, self.schema_spec())
                    &&& allowed_rows(out_left.to_multiset(), allowed)
                    &&& allowed_rows(out_right.to_multiset(), allowed)
                    &&& group_distance(
                        out_left.to_multiset(), out_right.to_multiset(), values,
                        |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
                        Seq::<Option<VValue>>::empty(),
                    ) <= group_distance(
                        left.to_multiset(), right.to_multiset(), values,
                        |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
                        Seq::<Option<VValue>>::empty(),
                    )
                },
                _ => false,
            },
    {
        lemma_single_filter_relation(&self.plan, left);
        lemma_single_filter_relation(&self.plan, right);
        verified::lemma_relation_type_sound(&self.plan, left, self.schema@, self.schema@);
        verified::lemma_relation_type_sound(&self.plan, right, self.schema@, self.schema@);
        let out_left = verified::relation_spec(&self.plan, left).unwrap();
        let out_right = verified::relation_spec(&self.plan, right).unwrap();
        assert(out_left.to_multiset().subset_of(left.to_multiset()));
        assert(out_right.to_multiset().subset_of(right.to_multiset()));
        lemma_allowed_rows_subset(out_left.to_multiset(), left.to_multiset(), allowed);
        lemma_allowed_rows_subset(out_right.to_multiset(), right.to_multiset(), allowed);
        assert(verified::local_schema(&self.plan, self.schema@) == Some(self.schema@));
        lemma_group_distance_contraction(
            &self.plan, left, right, self.schema@, values,
            |row: Seq<Option<VValue>>| Seq::<Option<VValue>>::empty(),
            Seq::<Option<VValue>>::empty(),
        );
    }

    pub fn map_distance(distance: u32) -> (output: u32)
        ensures output == distance,
    {
        distance
    }
}

} // verus!

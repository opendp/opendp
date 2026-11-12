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

pub fn construct(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        stability::certified_result(result, source_schema@),
        match result {
            Some((plan, fields)) => preserves_filter_multiset(&plan, fields@),
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
            }
            Some((plan, fields))
        },
    }
}

} // verus!

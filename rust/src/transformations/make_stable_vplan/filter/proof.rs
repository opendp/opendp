//! Executable filter node and row-domain preservation certificate.

use super::verified::{self, VExpr, VField, VPlan, VValue};
use super::stability;
use vstd::prelude::*;

verus! {

/// An arbitrary row-level constraint, including value bounds and null rules.
pub open spec fn rows_allowed(
    rows: Seq<Seq<Option<VValue>>>, allowed: Set<Seq<Option<VValue>>>,
) -> bool {
    forall|i: int| 0 <= i < rows.len() ==> allowed.contains(rows[i])
}

/// Filtering cannot introduce rows that violate any existing row constraint.
/// The length bound supplies the global cardinality portion of the margin.
pub proof fn lemma_filter_prefix_preserves_rows(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>, allowed: Set<Seq<Option<VValue>>>, count: nat,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(rows, schema),
        rows_allowed(rows, allowed),
        verified::local_schema(plan, schema) == Some(schema),
        count <= rows.len(),
    ensures
        match verified::apply_prefix_spec(plan, rows, count) {
            Ok(out) => rows_allowed(out, allowed) && out.len() <= count,
            Err(_) => false,
        },
    decreases count,
{
    verified::lemma_apply_prefix_type_sound(plan, rows, schema, schema, count);
    if count > 0 {
        lemma_filter_prefix_preserves_rows(plan, rows, schema, allowed, (count - 1) as nat);
        let prior = verified::apply_prefix_spec(plan, rows, (count - 1) as nat).unwrap();
        let row = rows[(count - 1) as int];
        assert(allowed.contains(row));
        match plan {
            VPlan::Filter { predicate, .. } => {
                match verified::filter::row_spec(predicate, row) {
                    Ok(Some(retained)) => {
                        assert(retained == row);
                        assert forall|i: int| 0 <= i < prior.push(retained).len()
                            implies allowed.contains(prior.push(retained)[i]) by {
                            if i < prior.len() {
                                assert(prior.push(retained)[i] == prior[i]);
                            } else {
                                assert(i == prior.len());
                                assert(prior.push(retained)[i] == retained);
                            }
                        }
                    },
                    Ok(None) => {},
                    Err(_) => { assert(false); },
                }
            },
            _ => { assert(false); },
        }
    }
}

pub open spec fn preserves_filter_domain(plan: &VPlan, schema: Seq<VField>) -> bool {
    forall|rows: Seq<Seq<Option<VValue>>>, allowed: Set<Seq<Option<VValue>>>|
        verified::rows_in_schema(rows, schema)
        && rows_allowed(rows, allowed)
        ==> match verified::apply_spec(plan, rows) {
            Ok(out) => rows_allowed(out, allowed) && out.len() <= rows.len(),
            Err(_) => false,
        }
}

pub fn construct(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        stability::certified_result(result, source_schema@),
        match result {
            Some((plan, fields)) => preserves_filter_domain(&plan, fields@),
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
                assert forall|rows: Seq<Seq<Option<VValue>>>,
                    allowed: Set<Seq<Option<VValue>>>|
                    verified::rows_in_schema(rows, fields@)
                    && rows_allowed(rows, allowed)
                    implies match verified::apply_spec(&plan, rows) {
                        Ok(out) => rows_allowed(out, allowed) && out.len() <= rows.len(),
                        Err(_) => false,
                    } by {
                    lemma_filter_prefix_preserves_rows(
                        &plan, rows, fields@, allowed, rows.len());
                }
            }
            Some((plan, fields))
        },
    }
}

} // verus!

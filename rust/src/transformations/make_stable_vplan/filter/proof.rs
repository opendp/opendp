//! Executable filter node and row-domain preservation certificate.

use super::verified::{self, VExpr, VField, VPlan, VTable, VValue};
use super::verified::domain::{self, VColumnRule};
use super::verified::frame_distance;
use super::verified::transformation;
use super::stability;
use super::verified::symmetric_distance;
use vstd::prelude::*;
use vstd::multiset::Multiset;

// Grouped-distance guarantees are separate from the completed scalar
// transformation certificate.
#[path = "proof/grouped.rs"]
mod grouped;

verus! {

/// Filtering is a row-local multiset restriction, so it cannot insert rows.
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
    lemma_filter_multiset_equivalent(plan, rows, schema);
    broadcast use vstd::multiset::group_multiset_axioms;
    let out = verified::apply_spec(plan, rows).unwrap();
    let keep = |row: Seq<Option<VValue>>|
        verified::local_row_spec(plan, row) == Ok(Some(row));
    assert forall|row: Seq<Option<VValue>>|
        out.to_multiset().count(row)
            <= rows.to_multiset().count(row) by { }
}

/// Exact retention of duplicates is now a corollary of the generic
/// row-local lift, rather than another operator-specific induction.
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
{
    lemma_filter_multiset_equivalent(plan, rows, schema);
    broadcast use vstd::multiset::group_multiset_axioms;
    let output = verified::apply_spec(plan, rows).unwrap();
    let keep = |row: Seq<Option<VValue>>|
        verified::local_row_spec(plan, row) == Ok(Some(row));
    assert forall|row: Seq<Option<VValue>>|
        output.to_multiset().count(row)
            <= rows.to_multiset().count(row) by { }
}

/// The executable filter is a generic row-local multiset restriction.
pub proof fn lemma_filter_multiset_equivalent(
    plan: &VPlan,
    rows: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>,
)
    requires
        matches!(plan, VPlan::Filter { .. }),
        verified::rows_in_schema(rows, schema),
        verified::local_schema(plan, schema) == Some(schema),
    ensures
        match verified::apply_spec(plan, rows) {
            Ok(out) => out.to_multiset() =~= rows.to_multiset().filter(
                |row: Seq<Option<VValue>>|
                    verified::local_row_spec(plan, row) == Ok(Some(row))
            ),
            Err(_) => false,
        },
{
    stability::lemma_apply_as_lift(plan, rows, schema, schema);
    let f = |row: Seq<Option<VValue>>| verified::local_row_spec(plan, row).unwrap();
    let keep = |row: Seq<Option<VValue>>|
        verified::local_row_spec(plan, row) == Ok(Some(row));
    let g = |row: Seq<Option<VValue>>|
        if keep(row) { Some(row) } else { None };
    assert forall|i: int| 0 <= i < rows.len()
        implies f(rows[i]) == g(rows[i]) by {
        let row = rows[i];
        verified::lemma_local_type_sound(plan, schema, row, schema);
        match plan {
            VPlan::Filter { predicate, .. } => {
                match verified::filter::row_spec(predicate, row) {
                    Ok(Some(retained)) => { assert(retained == row); },
                    Ok(None) => { },
                    Err(_) => { assert(false); },
                }
            },
            _ => { assert(false); },
        }
    }
    stability::lifting::lemma_lift_equal_on_rows(rows, f, g);
    stability::lifting::lemma_filter_lift_multiset(rows, keep);
    let output = verified::apply_spec(plan, rows).unwrap();
    assert(output =~= stability::lifting::lift(rows, f));
    assert(stability::lifting::lift(rows, g)
        =~= stability::lifting::filter_lift(rows, keep));
    assert(output.to_multiset() =~= rows.to_multiset().filter(keep));
}

pub fn construct(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        stability::certified_result(result, source_schema@),
        match result {
            Some((plan, fields)) =>
                matches!(plan, VPlan::Filter { .. })
                && verified::local_schema(&plan, fields@) == Some(fields@),
            None => true,
        },
{
    let plan = VPlan::Filter { input: Box::new(input), predicate };
    match stability::check_certified_plan(&plan, source_schema) {
        None => None,
        Some(fields) => {
            proof {
                assert(verified::local_schema(&plan, fields@) == Some(fields@));
            }
            Some((plan, fields))
        },
    }
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

    pub fn matches_rules(&self, rules: &Vec<VColumnRule>) -> (matches: bool)
        ensures matches == (self.rules_spec() == rules@),
    {
        domain::same_rules(&self.rules, rules)
    }
    pub closed spec fn schema_spec(&self) -> Seq<VField> { self.schema@ }
    pub closed spec fn output_schema(&self, input: Seq<VField>) -> Option<Seq<VField>> {
        verified::plan_schema(&self.plan, input)
    }
    pub closed spec fn result_spec(&self, rows: Seq<Seq<Option<VValue>>>)
        -> Result<Seq<Seq<Option<VValue>>>, ()>
    {
        verified::relation_spec(&self.plan, rows)
    }

    // A certificate is well formed by construction. Its stability theorem is
    // derived from this invariant; no proof field is stored on the value.
    #[verifier::type_invariant]
    spec fn invariant(&self) -> bool { self.valid() }

    pub closed spec fn valid(&self) -> bool {
        &&& domain::schema_member(self.schema@, self.rules@)
        &&& single_filter(&self.plan)
        &&& verified::plan_schema(&self.plan, self.schema@) == Some(self.schema@)
        &&& verified::local_schema(&self.plan, self.schema@) == Some(self.schema@)
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
                }
                let filter = Self { plan, schema: fields, rules: rules.clone() };
                proof { filter.lemma_valid_as_transformation(); }
                Some(filter)
        }
    }

    /// The concrete transformation contract is shared by all table nodes.
    pub closed spec fn valid_as_transformation(&self) -> bool {
        super::table_proof::valid_table_transformation(
            &self.plan, self.rules_spec(), self.rules_spec())
    }

    pub proof fn lemma_valid_as_transformation(tracked &self)
        ensures self.valid_as_transformation(),
    {
        use_type_invariant(self);
        assert forall|rows: Seq<Seq<Option<VValue>>>|
            domain::rows_member(rows, self.rules_spec())
            implies match self.result_spec(rows) {
                Ok(output) => domain::rows_member(output, self.rules_spec()),
                Err(_) => false,
            } by {
            self.lemma_preserves_rules(rows);
            assert(self.result_spec(rows) == verified::relation_spec(&self.plan, rows));
            assert(self.rules_spec() == self.rules@);
        }
        assert forall|rows: Seq<Seq<Option<VValue>>>|
            domain::rows_member(rows, self.rules@)
            implies match verified::relation_spec(&self.plan, rows) {
                Ok(output) => domain::rows_member(output, self.rules@),
                Err(_) => false,
            } by {
            self.lemma_preserves_rules(rows);
            assert(self.result_spec(rows) == verified::relation_spec(&self.plan, rows));
        }
        super::table_proof::lemma_valid_table_transformation(
            &self.plan, self.schema@, self.schema@, self.rules@, self.rules@);
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
            symmetric_distance::close(left.to_multiset(), right.to_multiset(), distance),
        ensures
            match (self.result_spec(left), self.result_spec(right)) {
                (Ok(a), Ok(b)) =>
                    symmetric_distance::close(a.to_multiset(), b.to_multiset(), distance),
                _ => false,
            },
    {
        domain::lemma_rows_member_typed(left, self.schema@, self.rules@);
        domain::lemma_rows_member_typed(right, self.schema@, self.rules@);
        assert(verified::plan_schema(&self.plan, self.schema@) == Some(self.schema@));
        stability::lemma_relation_contraction(
            &self.plan, left, right, self.schema@, self.schema@);

    }

    /// The entrypoint installed in OpenDP's Function. It enforces exactly
    /// the certified input domain; valid inputs always succeed and return a
    /// member of that same domain.
    pub fn invoke(&self, input: &VTable) -> (result: Result<VTable, ()>)
        ensures self.valid() ==> match result {
            Ok(output) =>
                domain::table_member(input, self.rules_spec())
                && domain::table_member(&output, self.rules_spec())
                && self.result_spec(verified::rows_view(input.rows@))
                    == Ok(verified::rows_view(output.rows@)),
            Err(_) => !domain::table_member(input, self.rules_spec()),
        },
    {
        proof {
            if self.valid() {
                assert forall|rows: Seq<Seq<Option<VValue>>>|
                    domain::rows_member(rows, self.rules@)
                    implies match verified::relation_spec(&self.plan, rows) {
                        Ok(output) => domain::rows_member(output, self.rules@),
                        Err(_) => false,
                    } by {
                    self.lemma_preserves_rules(rows);
                }
                assert(super::table_proof::certified_table_plan(
                    &self.plan, self.schema@, self.schema@, self.rules@, self.rules@));
            }
        }
        super::table_proof::invoke_table(
            &self.plan, &self.schema, &self.schema,
            &self.rules, &self.rules, input)
    }
}

} // verus!

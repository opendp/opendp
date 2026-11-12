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
    match verified::check_plan(&plan, source_schema) {
        None => None,
        Some(fields) => {
            proof {
                stability::lemma_all_typed_edits_stable(&plan, source_schema@);
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
                let input_rows = verified::rows_view(input.rows@);
                let output_rows = verified::rows_view(rows@);
                lemma_filter_multiset_equivalent(&self.plan, input_rows, self.schema@);
                let keep = |row: Seq<Option<VValue>>|
                    verified::local_row_spec(&self.plan, row) == Ok(Some(row));
                assert(output_rows.to_multiset() =~= input_rows.to_multiset().filter(keep));
                broadcast use vstd::multiset::group_multiset_axioms;
                assert forall|row: Seq<Option<VValue>>|
                    output_rows.to_multiset().count(row)
                        <= input_rows.to_multiset().count(row) by { }
                assert(output_rows.to_multiset().subset_of(input_rows.to_multiset()));
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
    /// The source of this transformation is the actual VTable carrier, not
    /// a second row-sequence domain that must be trusted as equivalent.
    /// The output carrier is the semantic rows; invoke additionally proves
    /// that its concrete returned VTable has exactly these rows and schema.
    pub closed spec fn valid_as_transformation(&self) -> bool {
        &&& verified::frame_metric_space::symmetric_table_space(self.rules_spec())
        &&& verified::frame_metric_space::symmetric_rows_space(self.rules_spec())
        &&& transformation::valid_transformation::<
            VTable, Seq<Seq<Option<VValue>>>,
            u32, u32, (), (),
        >(
            |table: VTable| domain::table_member(&table, self.rules_spec()),
            |rows: Seq<Seq<Option<VValue>>>|
                domain::rows_member(rows, self.rules_spec()),
            |left: VTable, right: VTable, d: u32|
                symmetric_distance::close(
                    verified::rows_view(left.rows@).to_multiset(),
                    verified::rows_view(right.rows@).to_multiset(), d),
            |left: Seq<Seq<Option<VValue>>>,
             right: Seq<Seq<Option<VValue>>>, d: u32|
                symmetric_distance::close(left.to_multiset(), right.to_multiset(), d),
            |_: u32| false,
            |table: VTable|
                self.result_spec(verified::rows_view(table.rows@)),
            |d: u32| Ok(d),
        )
    }

    /// Membership in the concrete VTable domain now supplies the input
    /// hypotheses to the already proved row-local contraction theorem.
    pub proof fn lemma_valid_as_transformation(&self)
        requires self.valid(),
        ensures self.valid_as_transformation(),
    {
        verified::frame_metric_space::lemma_symmetric_rows_space(self.rules@);
        verified::frame_metric_space::lemma_symmetric_table_space(self.rules@);
        assert forall|table: VTable|
            #[trigger] domain::table_member(&table, self.rules_spec())
            implies match self.result_spec(verified::rows_view(table.rows@)) {
                Ok(output) => domain::rows_member(output, self.rules_spec()),
                Err(_) => true,
            } by {
            domain::lemma_table_rows_domain(&table, self.rules@);
            self.lemma_preserves_rules(verified::rows_view(table.rows@));
        }
        assert forall|left: VTable, right: VTable, d: u32|
            #[trigger] domain::table_member(&left, self.rules_spec())
            && #[trigger] domain::table_member(&right, self.rules_spec())
            && #[trigger] symmetric_distance::close(
                verified::rows_view(left.rows@).to_multiset(),
                verified::rows_view(right.rows@).to_multiset(), d)
            implies transformation::result_close(
                self.result_spec(verified::rows_view(left.rows@)),
                self.result_spec(verified::rows_view(right.rows@)), d,
                |a: Seq<Seq<Option<VValue>>>,
                 b: Seq<Seq<Option<VValue>>>, bound: u32|
                    symmetric_distance::close(a.to_multiset(), b.to_multiset(), bound),
                |_: u32| false,
            ) by {
            domain::lemma_table_rows_domain(&left, self.rules@);
            domain::lemma_table_rows_domain(&right, self.rules@);
            self.lemma_preserves_rules(verified::rows_view(left.rows@));
            self.lemma_preserves_rules(verified::rows_view(right.rows@));
            self.lemma_symmetric_contraction(
                verified::rows_view(left.rows@),
                verified::rows_view(right.rows@), d);
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
            symmetric_distance::close(left.to_multiset(), right.to_multiset(), distance),
        ensures
            match (self.result_spec(left), self.result_spec(right)) {
                (Ok(a), Ok(b)) =>
                    symmetric_distance::close(a.to_multiset(), b.to_multiset(), distance),
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
        lemma_filter_multiset_equivalent(&self.plan, left, self.schema@);
        lemma_filter_multiset_equivalent(&self.plan, right, self.schema@);
        let out_left = self.result_spec(left).unwrap();
        let out_right = self.result_spec(right).unwrap();
        let keep = |row: Seq<Option<VValue>>|
            verified::local_row_spec(&self.plan, row) == Ok(Some(row));
        assert(out_left.to_multiset() =~= left.to_multiset().filter(keep));
        assert(out_right.to_multiset() =~= right.to_multiset().filter(keep));
        symmetric_distance::lemma_filter_contraction(
            left.to_multiset(), right.to_multiset(), keep);
        assert(symmetric_distance::distance(
            out_left.to_multiset(), out_right.to_multiset())
            <= symmetric_distance::distance(
                left.to_multiset(), right.to_multiset()));
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
        proof {
            domain::lemma_table_rows_domain(input, self.rules@);
        }
        self.apply(input)
    }

    pub fn map_distance(distance: u32) -> (output: u32)
        ensures output == distance,
    {
        distance
    }
}

} // verus!

//! Executable projection node, checked in Verus before OpenDP wraps it.

use super::verified::{self, VExpr, VField, VPlan, VTable};
use super::stability;
use super::table_proof;
use super::verified::{domain, symmetric_distance};
use super::verified::domain::VColumnRule;
use vstd::prelude::*;

verus! {

pub fn construct(
    input: VPlan, exprs: Vec<VExpr>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures stability::certified_result(result, source_schema@),
{
    let plan = VPlan::Project { input: Box::new(input), exprs };
    match stability::check_certified_plan(&plan, source_schema) {
        Some(fields) => Some((plan, fields)),
        None => None,
    }
}



/// Aliases do not change an identity column's value.
pub proof fn lemma_identity_value(
    expr: &VExpr, row: Seq<Option<verified::VValue>>, index: usize,
)
    requires
        verified::project::identity_column_spec(expr) == Some(index),
        index < row.len(),
    ensures verified::expr_row_spec(expr, row) == Ok(row[index as int]),
    decreases *expr,
{
    match expr {
        VExpr::Column(i) => { assert(*i == index); },
        VExpr::Alias { expr, .. } =>
            lemma_identity_value(expr, row, index),
        _ => { assert(false); },
    }
}

/// Identity columns retain their original rule; derived expressions must
/// have an unbounded rule, with their dtype/nullability checked separately.
pub open spec fn projection_rule(
    expr: &VExpr,
    input_rules: Seq<VColumnRule>,
    output_rule: VColumnRule,
) -> bool {
    match verified::project::identity_column_spec(expr) {
        Some(index) =>
            index < input_rules.len() && output_rule == input_rules[index as int],
        None => domain::unbounded_rule(output_rule),
    }
}

/// Well-typed expressions always satisfy their declared output column rules.
pub proof fn lemma_project_row_member(
    exprs: Seq<VExpr>,
    input_schema: Seq<VField>, output_schema: Seq<VField>,
    input_rules: Seq<VColumnRule>, output_rules: Seq<VColumnRule>,
    row: Seq<Option<verified::VValue>>,
)
    requires
        domain::row_member(row, input_rules),
        domain::schema_member(input_schema, input_rules),
        domain::schema_member(output_schema, output_rules),
        verified::project::schema_prefix(exprs, input_schema, exprs.len())
            == Some(output_schema),
        output_rules.len() == exprs.len(),
        forall|j: int| #![trigger exprs[j]] 0 <= j < exprs.len()
            ==> projection_rule(&exprs[j], input_rules, output_rules[j]),
    ensures
        match verified::project::row_prefix(exprs, row, exprs.len()) {
            Ok(output) => domain::row_member(output, output_rules),
            Err(_) => false,
        },
{
    domain::lemma_row_typed(row, input_schema, input_rules);
    verified::project::lemma_row_prefix_type_sound(
        exprs, input_schema, row, exprs.len());
    let output = verified::project::row_prefix(exprs, row, exprs.len()).unwrap();
    assert(verified::row_in_schema(output, output_schema));
    assert forall|j: int| 0 <= j < exprs.len()
        implies domain::cell_member(output[j], output_rules[j]) by {
        match verified::project::identity_column_spec(&exprs[j]) {
            Some(index) => {
                verified::project::lemma_row_prefix_cell(
                    exprs, row, exprs.len(), j);
                lemma_identity_value(&exprs[j], row, index);
                assert(output[j] == row[index as int]);
                assert(output_rules[j] == input_rules[index as int]);
            },
            None => {
                assert(verified::cell_matches_field(output[j], output_schema[j]));
                assert(domain::rule_field(output_rules[j]) == output_schema[j]);
                domain::lemma_unbounded_cell(output[j], output_rules[j]);
            },
        }
    }
}

/// Concrete projections preserve exact bounds for pass-through columns and
/// use type/nullability-only domains for arbitrary derived expressions.
pub struct CertifiedTableProject {
    plan: VPlan,
    input_schema: Vec<VField>,
    output_schema: Vec<VField>,
    input_rules: Vec<VColumnRule>,
    output_rules: Vec<VColumnRule>,
}

impl CertifiedTableProject {
    pub closed spec fn input_rules_spec(&self) -> Seq<VColumnRule> {
        self.input_rules@
    }
    pub closed spec fn output_rules_spec(&self) -> Seq<VColumnRule> {
        self.output_rules@
    }
    pub closed spec fn result_spec(&self, rows: Seq<Seq<Option<verified::VValue>>>)
        -> Result<Seq<Seq<Option<verified::VValue>>>, ()>
    {
        verified::relation_spec(&self.plan, rows)
    }

    // A certificate is well formed by construction. Its stability theorem is
    // derived from this invariant; no proof field is stored on the value.
    #[verifier::type_invariant]
    spec fn invariant(&self) -> bool { self.valid() }

    pub closed spec fn valid(&self) -> bool {
        &&& domain::schema_member(self.input_schema@, self.input_rules@)
        &&& domain::schema_member(self.output_schema@, self.output_rules@)
        &&& verified::plan_schema(&self.plan, self.input_schema@)
            == Some(self.output_schema@)
        &&& match &self.plan {
            VPlan::Project { input, exprs } =>
                matches!(**input, VPlan::Input)
                && self.output_rules@.len() == exprs@.len()
                && forall|j: int| #![trigger exprs@[j]] 0 <= j < exprs@.len()
                    ==> projection_rule(
                        &exprs@[j], self.input_rules@, self.output_rules@[j]),
            _ => false,
        }
    }

    pub closed spec fn valid_as_transformation(&self) -> bool {
        table_proof::valid_table_transformation(
            &self.plan, self.input_rules_spec(), self.output_rules_spec())
    }

    pub fn new(
        exprs: Vec<VExpr>,
        input_rules: &Vec<VColumnRule>,
        output_rules: &Vec<VColumnRule>,
    ) -> (result: Option<Self>)
        ensures match result {
            Some(project) =>
                project.valid() && project.valid_as_transformation()
                    && project.input_rules_spec() == input_rules@
                    && project.output_rules_spec() == output_rules@,
            None => true,
        },
    {
        if exprs.len() != output_rules.len() {
            return None;
        }
        let mut i: usize = 0;
        while i < exprs.len()
            invariant
                i <= exprs.len(),
                exprs.len() == output_rules.len(),
                forall|j: int| #![trigger exprs@[j]] 0 <= j < i
                    ==> projection_rule(&exprs@[j], input_rules@, output_rules@[j]),
            decreases exprs.len() - i,
        {
            match verified::project::identity_column(&exprs[i]) {
                Some(index) => {
                    if index >= input_rules.len() || output_rules[i] != input_rules[index] {
                        return None;
                    }
                },
                None => {
                    if !domain::check_unbounded_rule(output_rules[i]) {
                        return None;
                    }
                },
            }
            proof {
                assert(projection_rule(
                    &exprs@[i as int], input_rules@, output_rules@[i as int]));
                assert forall|k: int| #![trigger exprs@[k]] 0 <= k < i + 1
                    implies projection_rule(&exprs@[k], input_rules@, output_rules@[k]) by {
                    if k < i {
                    } else {
                        assert(k == i);
                    }
                }
            }
            i += 1;
        }
        let input_schema = domain::rule_schema(input_rules);
        let output_schema = domain::rule_schema(output_rules);
        let plan = VPlan::Project { input: Box::new(VPlan::Input), exprs };
        let checked = match verified::check_plan(&plan, &input_schema) {
            Some(fields) => fields,
            None => return None,
        };
        if checked.len() != output_schema.len() { return None; }
        let mut j: usize = 0;
        while j < checked.len()
            invariant
                j <= checked.len(),
                checked.len() == output_schema.len(),
                forall|k: int| 0 <= k < j ==> checked@[k] == output_schema@[k],
            decreases checked.len() - j,
        {
            if checked[j] != output_schema[j] { return None; }
            j += 1;
        }
        proof { assert(checked@ =~= output_schema@); }
        let project = Self {
            plan,
            input_schema, output_schema,
            input_rules: input_rules.clone(),
            output_rules: output_rules.clone(),
        };
        proof { project.lemma_valid_as_transformation(); }
        Some(project)
    }

    proof fn lemma_preserves_prefix(
        &self,
        rows: Seq<Seq<Option<verified::VValue>>>, count: nat,
    )
        requires
            self.valid(),
            domain::rows_member(rows, self.input_rules_spec()),
            count <= rows.len(),
        ensures match verified::apply_prefix_spec(&self.plan, rows, count) {
            Ok(output) => domain::rows_member(output, self.output_rules_spec()),
            Err(_) => false,
        },
        decreases count,
    {
        if count > 0 {
            self.lemma_preserves_prefix(rows, (count - 1) as nat);
            let row = rows[(count - 1) as int];
            assert(domain::row_member(row, self.input_rules@));
            match &self.plan {
                VPlan::Project { input, exprs } => {
                    assert(matches!(**input, VPlan::Input));
                    assert(verified::plan_schema(input, self.input_schema@)
                        == Some(self.input_schema@));
                    assert(verified::plan_schema(&self.plan, self.input_schema@)
                        == Some(self.output_schema@));
                    assert(verified::local_schema(&self.plan, self.input_schema@)
                        == Some(self.output_schema@));
                    assert(verified::project::schema_prefix(
                        exprs@, self.input_schema@, exprs@.len())
                        == Some(self.output_schema@));
                    lemma_project_row_member(
                        exprs@, self.input_schema@, self.output_schema@,
                        self.input_rules@, self.output_rules@, row);
                    let projected = verified::project::row_prefix(
                        exprs@, row, exprs@.len()).unwrap();
                    assert(domain::row_member(projected, self.output_rules@));
                    assert(verified::local_row_spec(&self.plan, row)
                        == Ok(Some(projected)));
                    let previous = verified::apply_prefix_spec(
                        &self.plan, rows, (count - 1) as nat).unwrap();
                    let result = previous.push(projected);
                    assert(verified::apply_prefix_spec(&self.plan, rows, count)
                        == Ok(result));
                    assert forall|j: int| 0 <= j < result.len()
                        implies domain::row_member(result[j], self.output_rules@) by {
                        if j < previous.len() {
                        } else {
                            assert(j == previous.len());
                        }
                    }
                },
                _ => { assert(false); },
            }
        }
    }

    proof fn lemma_preserves_rules(&self, rows: Seq<Seq<Option<verified::VValue>>>)
        requires self.valid(), domain::rows_member(rows, self.input_rules@),
        ensures match self.result_spec(rows) {
            Ok(output) => domain::rows_member(output, self.output_rules@),
            Err(_) => false,
        },
    {
        self.lemma_preserves_prefix(rows, rows.len());
        match &self.plan {
            VPlan::Project { input, .. } => {
                assert(matches!(**input, VPlan::Input));
                assert(verified::relation_spec(input, rows) == Ok(rows));
                assert(self.result_spec(rows) == verified::apply_spec(&self.plan, rows));
            },
            _ => { assert(false); },
        }
    }

    pub proof fn lemma_valid_as_transformation(tracked &self)
        ensures self.valid_as_transformation(),
    {
        use_type_invariant(self);
        assert forall|rows: Seq<Seq<Option<verified::VValue>>>|
            domain::rows_member(rows, self.input_rules@)
            implies match verified::relation_spec(&self.plan, rows) {
                Ok(output) => domain::rows_member(output, self.output_rules@),
                Err(_) => false,
            } by {
            self.lemma_preserves_rules(rows);
        }
        table_proof::lemma_valid_table_transformation(
            &self.plan, self.input_schema@, self.output_schema@,
            self.input_rules@, self.output_rules@);
    }

    pub fn invoke(&self, input: &VTable) -> (result: Result<VTable, ()>)
        ensures self.valid() ==> match result {
            Ok(output) =>
                domain::table_member(input, self.input_rules_spec())
                    && domain::table_member(&output, self.output_rules_spec())
                    && self.result_spec(verified::rows_view(input.rows@))
                        == Ok(verified::rows_view(output.rows@)),
            Err(_) => !domain::table_member(input, self.input_rules_spec()),
        },
    {
        proof {
            if self.valid() {
                assert forall|rows: Seq<Seq<Option<verified::VValue>>>|
                    domain::rows_member(rows, self.input_rules@)
                    implies match verified::relation_spec(&self.plan, rows) {
                        Ok(output) => domain::rows_member(output, self.output_rules@),
                        Err(_) => false,
                    } by {
                    self.lemma_preserves_rules(rows);
                }
                assert(table_proof::certified_table_plan(
                    &self.plan, self.input_schema@, self.output_schema@,
                    self.input_rules@, self.output_rules@));
            }
        }
        table_proof::invoke_table(
            &self.plan, &self.input_schema, &self.output_schema,
            &self.input_rules, &self.output_rules, input)
    }
}

} // verus!

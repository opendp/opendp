//! Executable projection node, checked in Verus before OpenDP wraps it.

use super::verified::{VExpr, VField, VPlan};
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

/// A projection composed of selections and aliases preserves exactly the
/// column rules selected by each expression, including duplicate values.
pub proof fn lemma_project_row_member(
    exprs: Seq<VExpr>,
    input_rules: Seq<VColumnRule>, output_rules: Seq<VColumnRule>,
    row: Seq<Option<verified::VValue>>, count: nat,
)
    requires
        domain::row_member(row, input_rules),
        count <= exprs.len(),
        output_rules.len() == exprs.len(),
        forall|j: int| 0 <= j < exprs.len() ==>
            match verified::project::identity_column_spec(&exprs[j]) {
                Some(index) => index < input_rules.len()
                    && output_rules[j] == input_rules[index as int],
                None => false,
            },
    ensures
        match verified::project::row_prefix(exprs, row, count) {
            Ok(output) =>
                output.len() == count
                && forall|j: int| 0 <= j < count
                    ==> domain::cell_member(output[j], output_rules[j]),
            Err(_) => false,
        },
    decreases count,
{
    if count > 0 {
        lemma_project_row_member(
            exprs, input_rules, output_rules, row, (count - 1) as nat);
        let previous = verified::project::row_prefix(exprs, row, (count - 1) as nat).unwrap();
        let expr = &exprs[(count - 1) as int];
        let index = verified::project::identity_column_spec(expr).unwrap();
        lemma_identity_value(expr, row, index);
        assert(domain::cell_member(row[index as int], input_rules[index as int]));
        assert(output_rules[(count - 1) as int] == input_rules[index as int]);
        let output = previous.push(row[index as int]);
        assert(verified::project::row_prefix(exprs, row, count) == Ok(output));
        assert forall|j: int| 0 <= j < count
            implies domain::cell_member(output[j], output_rules[j]) by {
            if j < previous.len() {
            } else {
                assert(j == previous.len());
            }
        }
    }
}

/// A concrete table projection is limited to column references and aliases,
/// retaining their exact value constraints. Other expressions remain
/// available in the symbolic VPlan API until their bounds are certified.
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

    pub closed spec fn valid(&self) -> bool {
        &&& domain::schema_member(self.input_schema@, self.input_rules@)
        &&& domain::schema_member(self.output_schema@, self.output_rules@)
        &&& verified::plan_schema(&self.plan, self.input_schema@)
            == Some(self.output_schema@)
        &&& match &self.plan {
            VPlan::Project { input, exprs } =>
                matches!(**input, VPlan::Input)
                && self.output_rules@.len() == exprs@.len()
                && forall|j: int| 0 <= j < exprs@.len() ==>
                    match verified::project::identity_column_spec(&exprs@[j]) {
                        Some(index) => index < self.input_rules@.len()
                            && self.output_rules@[j]
                                == self.input_rules@[index as int],
                        None => false,
                    },
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
                forall|j: int| 0 <= j < i ==>
                    match verified::project::identity_column_spec(&exprs@[j]) {
                        Some(index) => index < input_rules@.len()
                            && output_rules@[j] == input_rules@[index as int],
                        None => false,
                    },
            decreases exprs.len() - i,
        {
            let index = match verified::project::identity_column(&exprs[i]) {
                Some(index) => index,
                None => return None,
            };
            if index >= input_rules.len() || output_rules[i] != input_rules[index] {
                return None;
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
                VPlan::Project { exprs, .. } => {
                    lemma_project_row_member(
                        exprs@, self.input_rules@, self.output_rules@,
                        row, exprs@.len());
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

    pub proof fn lemma_valid_as_transformation(&self)
        requires self.valid(),
        ensures self.valid_as_transformation(),
    {
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
                domain::table_member(input, self.input_rules@)
                    && domain::table_member(&output, self.output_rules@)
                    && self.result_spec(verified::rows_view(input.rows@))
                        == Ok(verified::rows_view(output.rows@)),
            Err(_) => !domain::table_member(input, self.input_rules@),
        },
    {
        if !domain::check_table(input, &self.input_rules) {
            return Err(());
        }
        proof {
            domain::lemma_table_typed(input, self.input_rules@);
            if self.valid() {
                domain::lemma_unique_schema(
                    input.schema@, self.input_schema@, self.input_rules@);
                verified::lemma_relation_type_sound(
                    &self.plan, verified::rows_view(input.rows@),
                    self.input_schema@, self.output_schema@);
            }
        }
        let rows = match verified::evaluate(&self.plan, input) {
            Ok(rows) => rows,
            Err(_) => {
                proof { if self.valid() { assert(false); } }
                return Err(());
            },
        };
        let output = VTable { schema: self.output_schema.clone(), rows };
        proof {
            if self.valid() {
                let input_rows = verified::rows_view(input.rows@);
                domain::lemma_table_rows_domain(input, self.input_rules@);
                self.lemma_preserves_rules(input_rows);
                domain::lemma_table_rows_domain(&output, self.output_rules@);
            }
        }
        Ok(output)
    }

    pub fn map_distance(distance: u32) -> (output: u32)
        ensures output == distance,
    {
        distance
    }
}

} // verus!

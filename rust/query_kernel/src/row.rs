use vstd::prelude::*;

#[path = "row/project.rs"]
pub mod project;
#[path = "row/margin.rs"]
pub mod margin;
#[path = "row/filter.rs"]
pub mod filter;
#[path = "../../src/domains/verified_frame/verus.rs"]
pub mod domain;
#[path = "../../src/core/verus/transformation.rs"]
pub mod transformation;
#[path = "../../src/core/chain_result.rs"]
pub mod chain_result;
#[path = "../../src/combinators/chain/verus.rs"]
pub mod chain;
#[path = "../../src/core/verus/metric_space.rs"]
pub mod metric_space;
#[path = "../../src/core/verus/row_local.rs"]
pub mod row_local;
#[path = "../../src/domains/polars/frame/verus.rs"]
pub mod frame_metric_space;
#[path = "../../src/metrics/symmetric_distance/verus.rs"]
pub mod symmetric_distance;
#[path = "../../src/metrics/polars/frame_distance/verus.rs"]
pub mod frame_distance;
#[path = "row/scalar.rs"]
pub mod scalar;
use scalar::{binary_field, clip_field, eval_binary, eval_clip};

verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq, StructuralEq, Hash)]
pub enum VDType { Int64, Boolean }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VValue { Int(i64), Bool(bool) }

#[derive(Clone, Copy, Debug, PartialEq, Eq, StructuralEq, Hash)]
pub struct VField {
    pub dtype: VDType,
    pub nullable: bool,
}

/// Typed rows are the sole physical and logical representation in the
/// verified kernel. Arrow conversion is handled by an external adapter.
#[derive(Clone, Debug)]
pub struct VTable {
    pub schema: Vec<VField>,
    pub rows: Vec<Vec<Option<VValue>>>,
}

pub open spec fn table_valid(table: &VTable) -> bool {
    rows_in_schema(rows_view(table.rows@), table.schema@)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VBinaryOp { Lt, LtEq, Gt, GtEq, Eq, NotEq, And }

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum VExpr {
    Column(usize),
    Alias { expr: Box<VExpr>, name: String },
    Literal(VValue),
    Null(VDType),
    Binary { left: Box<VExpr>, op: VBinaryOp, right: Box<VExpr> },
    Clip {
        expr: Box<VExpr>,
        lower: Option<Box<VExpr>>,
        upper: Option<Box<VExpr>>,
    },
}

#[derive(Debug)]
pub enum VPlan {
    Input,
    Project { input: Box<VPlan>, exprs: Vec<VExpr> },
    Filter { input: Box<VPlan>, predicate: VExpr },
}

//// A bound, backend-neutral query. S is not required to be executable.
//// Production backends lower the data-free plan using their own source handle.
pub struct VLazyFrame<S> {
    pub plan: VPlan,
    pub source: S,
}

impl<S> VLazyFrame<S> {
    pub fn new(source: S) -> Self {
        Self { plan: VPlan::Input, source }
    }

    /// Build a lazy filter without inspecting or materializing the source.
    pub fn filter(self, predicate: VExpr) -> Self {
        Self {
            plan: VPlan::Filter {
                input: Box::new(self.plan),
                predicate,
            },
            source: self.source,
        }
    }

    /// Build a lazy projection without inspecting or materializing the source.
    pub fn project(self, exprs: Vec<VExpr>) -> Self {
        Self {
            plan: VPlan::Project {
                input: Box::new(self.plan),
                exprs,
            },
            source: self.source,
        }
    }
}

impl VLazyFrame<VTable> {
    /// The reference interpreter is intentionally only available for VTable.
    /// Other source bindings must supply separate backend semantics.
    pub fn collect(&self) -> (output: Result<VTable, ()>)
        ensures
            match output {
                Ok(table) =>
                    table_valid(&self.source)
                    && plan_schema(&self.plan, self.source.schema@) == Some(table.schema@)
                    && relation_spec(&self.plan, rows_view(self.source.rows@))
                        == Ok(rows_view(table.rows@)),
                Err(_) => true,
            },
    {
        if !check_table(&self.source) {
            return Err(());
        }
        let schema = match check_plan(&self.plan, &self.source.schema) {
            Some(schema) => schema,
            None => return Err(()),
        };
        match evaluate(&self.plan, &self.source) {
            Ok(rows) => Ok(VTable { schema, rows }),
            Err(_) => Err(()),
        }
    }
}

pub open spec fn value_dtype(value: VValue) -> VDType {
    match value {
        VValue::Int(_) => VDType::Int64,
        VValue::Bool(_) => VDType::Boolean,
    }
}

pub open spec fn expr_field(expr: &VExpr, schema: Seq<VField>) -> Option<VField>
    decreases *expr,
{
    match expr {
        VExpr::Column(index) =>
            if *index < schema.len() { Some(schema[*index as int]) } else { None },
        VExpr::Alias { expr, .. } => expr_field(expr, schema),
        VExpr::Literal(value) => Some(VField { dtype: value_dtype(*value), nullable: false }),
        VExpr::Null(dtype) => Some(VField { dtype: *dtype, nullable: true }),
        VExpr::Binary { left, op, right } =>
            binary_field(*op, expr_field(left, schema), expr_field(right, schema)),
        VExpr::Clip { expr, lower, upper } =>
            clip_field(
                expr_field(expr, schema),
                match lower { None => None, Some(e) => Some(expr_field(e, schema)) },
                match upper { None => None, Some(e) => Some(expr_field(e, schema)) },
            ),
    }
}

/// Executable type checking, proved equivalent to the specification.
pub fn check_expr(expr: &VExpr, schema: &Vec<VField>) -> (result: Option<VField>)
    ensures result == expr_field(expr, schema@),
    decreases *expr,
{
    match expr {
        VExpr::Column(index) =>
            if *index < schema.len() { Some(schema[*index]) } else { None },
        VExpr::Alias { expr, .. } => check_expr(expr, schema),
        VExpr::Literal(value) => Some(VField {
            dtype: match value {
                VValue::Int(_) => VDType::Int64,
                VValue::Bool(_) => VDType::Boolean,
            },
            nullable: false,
        }),
        VExpr::Null(dtype) => Some(VField { dtype: *dtype, nullable: true }),
        VExpr::Binary { left, op, right } =>
            binary_field(*op, check_expr(left, schema), check_expr(right, schema)),
        VExpr::Clip { expr, lower, upper } =>
            clip_field(
                check_expr(expr, schema),
                match lower { None => None, Some(e) => Some(check_expr(e, schema)) },
                match upper { None => None, Some(e) => Some(check_expr(e, schema)) },
            ),
    }
}

/// Type checking for a single relational operator.
pub open spec fn local_schema(plan: &VPlan, schema: Seq<VField>) -> Option<Seq<VField>> {
    match plan {
        VPlan::Input => Some(schema),
        VPlan::Project { exprs, .. } =>
            project::schema_prefix(exprs@, schema, exprs@.len()),
        VPlan::Filter { predicate, .. } => filter::schema(predicate, schema),
    }
}

pub open spec fn plan_schema(plan: &VPlan, input_schema: Seq<VField>) -> Option<Seq<VField>>
    decreases *plan,
{
    match plan {
        VPlan::Input => Some(input_schema),
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } =>
            match plan_schema(input, input_schema) {
                Some(child_schema) => local_schema(plan, child_schema),
                None => None,
            },
    }
}

/// Executable whole-plan typing composed from operator-local checkers.
pub fn check_plan(plan: &VPlan, input_schema: &Vec<VField>)
    -> (result: Option<Vec<VField>>)
    ensures
        match result {
            Some(fields) => plan_schema(plan, input_schema@) == Some(fields@),
            None => plan_schema(plan, input_schema@).is_None(),
        },
    decreases *plan,
{
    match plan {
        VPlan::Input => Some(input_schema.clone()),
        VPlan::Project { input, exprs } => {
            match check_plan(input, input_schema) {
                Some(child) => project::check_schema_prefix(exprs, &child, exprs.len()),
                None => None,
            }
        },
        VPlan::Filter { input, predicate } => {
            match check_plan(input, input_schema) {
                Some(child) => filter::check_schema(predicate, &child),
                None => None,
            }
        },
    }
}

pub open spec fn cell_matches_field(value: Option<VValue>, field: VField) -> bool {
    match value {
        Some(value) => value_dtype(value) == field.dtype,
        None => field.nullable,
    }
}

/// A semantic cell view allows generic, order-independent stability arguments.
pub open spec fn expr_row_spec(
    expr: &VExpr, row: Seq<Option<VValue>>,
) -> Result<Option<VValue>, ()>
    decreases *expr,
{
    match expr {
        VExpr::Column(index) =>
            if *index < row.len() { Ok(row[*index as int]) } else { Err(()) },
        VExpr::Alias { expr, .. } => expr_row_spec(expr, row),
        VExpr::Literal(value) => Ok(Some(*value)),
        VExpr::Null(_) => Ok(None),
        VExpr::Binary { left, op, right } =>
            match (expr_row_spec(left, row), expr_row_spec(right, row)) {
                (Ok(lhs), Ok(rhs)) => eval_binary(*op, lhs, rhs),
                _ => Err(()),
            },
        VExpr::Clip { expr, lower, upper } => {
            let value = expr_row_spec(expr, row);
            let low = match lower {
                None => Ok(None),
                Some(bound) => match expr_row_spec(bound, row) {
                    Ok(cell) => Ok(Some(cell)),
                    Err(_) => Err(()),
                },
            };
            let high = match upper {
                None => Ok(None),
                Some(bound) => match expr_row_spec(bound, row) {
                    Ok(cell) => Ok(Some(cell)),
                    Err(_) => Err(()),
                },
            };
            match (value, low, high) {
                (Ok(value), Ok(low), Ok(high)) => eval_clip(value, low, high),
                _ => Err(()),
            }
        },
    }
}


/// Domain membership includes the declared dtype and nullability of every cell.
pub open spec fn row_in_schema(
    row: Seq<Option<VValue>>, schema: Seq<VField>,
) -> bool {
    row.len() == schema.len()
        && forall|i: int| 0 <= i < row.len()
            ==> cell_matches_field(row[i], schema[i])
}

pub open spec fn rows_in_schema(
    rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
) -> bool {
    forall|i: int| 0 <= i < rows.len() ==> row_in_schema(rows[i], schema)
}

/// Executable cell membership, proved equivalent to the typed-cell specification.
pub fn check_cell(cell: Option<VValue>, field: VField) -> (valid: bool)
    ensures valid == cell_matches_field(cell, field),
{
    match cell {
        None => field.nullable,
        Some(VValue::Int(_)) => match field.dtype {
            VDType::Int64 => true,
            VDType::Boolean => false,
        },
        Some(VValue::Bool(_)) => match field.dtype {
            VDType::Boolean => true,
            VDType::Int64 => false,
        },
    }
}

/// Validate one row against its declared schema.
pub fn check_row(row: &Vec<Option<VValue>>, schema: &Vec<VField>) -> (valid: bool)
    ensures valid == row_in_schema(row@, schema@),
{
    if row.len() != schema.len() {
        return false;
    }
    let mut i: usize = 0;
    while i < row.len()
        invariant
            i <= row.len(),
            row.len() == schema.len(),
            forall|j: int| 0 <= j < i ==> cell_matches_field(row@[j], schema@[j]),
        decreases row.len() - i,
    {
        let good = check_cell(row[i], schema[i]);
        if !good {
            proof {
                assert(!cell_matches_field(row@[i as int], schema@[i as int]));
                assert(!row_in_schema(row@, schema@));
            }
            return false;
        }
        proof {
            assert forall|j: int| 0 <= j < i + 1
                implies cell_matches_field(row@[j], schema@[j]) by {
                if j == i {
                    assert(cell_matches_field(row@[j], schema@[j]));
                }
            }
        }
        i += 1;
    }
    true
}

/// Validate a concrete table using the exact predicate used by the Verus proofs.
pub fn check_table(table: &VTable) -> (valid: bool)
    ensures valid == table_valid(table),
{
    let mut i: usize = 0;
    while i < table.rows.len()
        invariant
            i <= table.rows.len(),
            forall|j: int| 0 <= j < i ==>
                row_in_schema(rows_view(table.rows@)[j], table.schema@),
        decreases table.rows.len() - i,
    {
        let good = check_row(&table.rows[i], &table.schema);
        if !good {
            proof {
                assert(rows_view(table.rows@)[i as int] == table.rows@[i as int]@);
                assert(!row_in_schema(rows_view(table.rows@)[i as int], table.schema@));
                assert(!table_valid(table));
            }
            return false;
        }
        proof {
            assert forall|j: int| 0 <= j < i + 1
                implies row_in_schema(rows_view(table.rows@)[j], table.schema@) by {
                if j == i {
                    assert(rows_view(table.rows@)[j] == table.rows@[j]@);
                }
            }
        }
        i += 1;
    }
    true
}

/// Well-typed expressions cannot fail, and nullable outputs are represented
/// explicitly by None instead of conflating null with an evaluation error.
pub proof fn lemma_expr_field_sound(
    expr: &VExpr, schema: Seq<VField>, row: Seq<Option<VValue>>, field: VField,
)
    requires
        row_in_schema(row, schema),
        expr_field(expr, schema) == Some(field),
    ensures
        match expr_row_spec(expr, row) {
            Ok(value) => cell_matches_field(value, field),
            Err(_) => false,
        },
    decreases *expr,
{
    match expr {
        VExpr::Column(index) => {
            assert(*index < schema.len());
            assert(cell_matches_field(row[*index as int], schema[*index as int]));
        },
        VExpr::Alias { expr, .. } => lemma_expr_field_sound(expr, schema, row, field),
        VExpr::Literal(_) | VExpr::Null(_) => {},
        VExpr::Binary { left, op: _, right } => {
            let left_field = expr_field(left, schema).unwrap();
            let right_field = expr_field(right, schema).unwrap();
            lemma_expr_field_sound(left, schema, row, left_field);
            lemma_expr_field_sound(right, schema, row, right_field);
            match (expr_row_spec(left, row), expr_row_spec(right, row)) {
                (Ok(left_cell), Ok(right_cell)) => {
                    assert(cell_matches_field(left_cell, left_field));
                    assert(cell_matches_field(right_cell, right_field));
                },
                _ => { assert(false); },
            }
        },
        VExpr::Clip { expr: child, lower, upper } => {
            let child_field = expr_field(child, schema).unwrap();
            lemma_expr_field_sound(child, schema, row, child_field);
            if let Some(bound) = lower {
                let bound_field = expr_field(bound, schema).unwrap();
                lemma_expr_field_sound(bound, schema, row, bound_field);
            }
            if let Some(bound) = upper {
                let bound_field = expr_field(bound, schema).unwrap();
                lemma_expr_field_sound(bound, schema, row, bound_field);
            }
        },
    }
}

/// Typing soundness for one operator, without evaluating its child plan.
pub proof fn lemma_local_type_sound(
    plan: &VPlan, schema: Seq<VField>, row: Seq<Option<VValue>>, output_schema: Seq<VField>,
)
    requires
        row_in_schema(row, schema),
        local_schema(plan, schema) == Some(output_schema),
    ensures
        match local_row_spec(plan, row) {
            Ok(Some(result)) => row_in_schema(result, output_schema),
            Ok(None) => true,
            Err(_) => false,
        },
{
    match plan {
        VPlan::Input => {},
        VPlan::Project { exprs, .. } =>
            project::lemma_row_prefix_type_sound(exprs@, schema, row, exprs@.len()),
        VPlan::Filter { predicate, .. } =>
            filter::lemma_type_sound(predicate, schema, row),
    }
}

/// A row-local step does not evaluate the plan's child.
pub open spec fn local_row_spec(
    plan: &VPlan, row: Seq<Option<VValue>>,
) -> Result<Option<Seq<Option<VValue>>>, ()> {
    match plan {
        VPlan::Input => Ok(Some(row)),
        VPlan::Project { exprs, .. } =>
            match project::row_prefix(exprs@, row, exprs@.len()) {
                Ok(out) => Ok(Some(out)),
                Err(_) => Err(()),
            },
        VPlan::Filter { predicate, .. } => filter::row_spec(predicate, row),
    }
}

pub open spec fn apply_prefix_spec(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>, count: nat,
) -> Result<Seq<Seq<Option<VValue>>>, ()>
    recommends count <= rows.len(),
    decreases count,
{
    if count == 0 { Ok(Seq::empty()) }
    else {
        match apply_prefix_spec(plan, rows, (count - 1) as nat) {
            Err(_) => Err(()),
            Ok(prefix) => match local_row_spec(plan, rows[(count - 1) as int]) {
                Err(_) => Err(()),
                Ok(None) => Ok(prefix),
                Ok(Some(row)) => Ok(prefix.push(row)),
            },
        }
    }
}

pub open spec fn apply_spec(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
) -> Result<Seq<Seq<Option<VValue>>>, ()> {
    apply_prefix_spec(plan, rows, rows.len())
}

/// A plan node transforms the complete child relation.
pub open spec fn relation_spec(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
) -> Result<Seq<Seq<Option<VValue>>>, ()>
    decreases *plan,
{
    match plan {
        VPlan::Input => Ok(rows),
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } =>
            match relation_spec(input, rows) {
                Err(_) => Err(()),
                Ok(previous) => apply_spec(plan, previous),
            },
    }
}

/// Expressions read the same row type used by the relation.
pub fn eval_expr_row(expr: &VExpr, row: &Vec<Option<VValue>>)
    -> (result: Result<Option<VValue>, ()>)
    ensures result == expr_row_spec(expr, row@),
    decreases *expr,
{
    match expr {
        VExpr::Column(index) =>
            if *index < row.len() { Ok(row[*index]) } else { Err(()) },
        VExpr::Alias { expr, .. } => eval_expr_row(expr, row),
        VExpr::Literal(value) => Ok(Some(*value)),
        VExpr::Null(_) => Ok(None),
        VExpr::Binary { left, op, right } =>
            match (eval_expr_row(left, row), eval_expr_row(right, row)) {
                (Ok(lhs), Ok(rhs)) => eval_binary(*op, lhs, rhs),
                _ => Err(()),
            },
        VExpr::Clip { expr, lower, upper } => {
            let value = eval_expr_row(expr, row);
            let low = match lower {
                None => Ok(None),
                Some(bound) => match eval_expr_row(bound, row) {
                    Ok(cell) => Ok(Some(cell)),
                    Err(_) => Err(()),
                },
            };
            let high = match upper {
                None => Ok(None),
                Some(bound) => match eval_expr_row(bound, row) {
                    Ok(cell) => Ok(Some(cell)),
                    Err(_) => Err(()),
                },
            };
            match (value, low, high) {
                (Ok(value), Ok(low), Ok(high)) => eval_clip(value, low, high),
                _ => Err(()),
            }
        },
    }
}

/// Evaluate just one node's row-local operation.
pub fn eval_local_row(plan: &VPlan, row: &Vec<Option<VValue>>)
    -> (result: Result<Option<Vec<Option<VValue>>>, ()>)
    ensures
        match result {
            Err(_) => local_row_spec(plan, row@).is_Err(),
            Ok(None) => local_row_spec(plan, row@) == Ok(None),
            Ok(Some(out)) => local_row_spec(plan, row@) == Ok(Some(out@)),
        },
{
    match plan {
        VPlan::Input => Ok(Some(row.clone())),
        VPlan::Project { exprs, .. } =>
            match project::eval_prefix(exprs, row, exprs.len()) {
                Ok(output) => Ok(Some(output)),
                Err(_) => Err(()),
            },
        VPlan::Filter { predicate, .. } => filter::eval_row(predicate, row),
    }
}

pub open spec fn rows_view(rows: Seq<Vec<Option<VValue>>>) -> Seq<Seq<Option<VValue>>> {
    Seq::new(rows.len(), |i: int| rows[i]@)
}

pub proof fn lemma_rows_view_push(
    rows: Seq<Vec<Option<VValue>>>, row: Vec<Option<VValue>>,
)
    ensures rows_view(rows.push(row)) =~= rows_view(rows).push(row@),
{
    assert forall|i: int| 0 <= i < rows_view(rows.push(row)).len()
        implies rows_view(rows.push(row))[i] == rows_view(rows).push(row@)[i] by {
        if i < rows.len() {
            assert(rows.push(row)[i] == rows[i]);
        } else {
            assert(i == rows.len());
        }
    }
}

/// Execute a row-local operator on an entire relation, not the whole plan.
pub fn eval_apply_prefix(
    plan: &VPlan, rows: &Vec<Vec<Option<VValue>>>, count: usize,
) -> (output: Result<Vec<Vec<Option<VValue>>>, ()>)
    requires count <= rows.len(),
    ensures
        match output {
            Ok(result) => apply_prefix_spec(plan, rows_view(rows@), count as nat)
                == Ok(rows_view(result@)),
            Err(_) => apply_prefix_spec(plan, rows_view(rows@), count as nat).is_Err(),
        },
    decreases count,
{
    if count == 0 {
        let empty: Vec<Vec<Option<VValue>>> = Vec::new();
        proof { assert(rows_view(empty@) =~= Seq::<Seq<Option<VValue>>>::empty()); }
        Ok(empty)
    } else {
        match eval_apply_prefix(plan, rows, count - 1) {
            Err(_) => Err(()),
            Ok(mut previous) => {
                let row = &rows[count - 1];
                proof {
                    assert(rows_view(rows@)[(count - 1) as int] ==
                        rows@[(count - 1) as int]@);
                }
                match eval_local_row(plan, row) {
                    Err(_) => Err(()),
                    Ok(None) => Ok(previous),
                    Ok(Some(result)) => {
                        let ghost old_rows = previous@;
                        proof { lemma_rows_view_push(old_rows, result); }
                        previous.push(result);
                        Ok(previous)
                    },
                }
            },
        }
    }
}

/// Evaluate child plans once; then apply the current relation operator.
pub fn eval_plan_rows(
    plan: &VPlan, rows: &Vec<Vec<Option<VValue>>>,
) -> (output: Result<Vec<Vec<Option<VValue>>>, ()>)
    ensures
        match output {
            Ok(result) => relation_spec(plan, rows_view(rows@)) == Ok(rows_view(result@)),
            Err(_) => relation_spec(plan, rows_view(rows@)).is_Err(),
        },
    decreases *plan,
{
    match plan {
        VPlan::Input => {
            let copy = rows.clone();
            proof { assert(rows_view(copy@) =~= rows_view(rows@)); }
            Ok(copy)
        },
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } => {
            match eval_plan_rows(input, rows) {
                Err(_) => {
                    proof { assert(relation_spec(input, rows_view(rows@)).is_Err()); }
                    Err(())
                },
                Ok(previous) => {
                    proof {
                        assert(relation_spec(input, rows_view(rows@))
                            == Ok(rows_view(previous@)));
                    }
                    let out = eval_apply_prefix(plan, &previous, previous.len());
                    proof {
                        assert(relation_spec(plan, rows_view(rows@))
                            == apply_spec(plan, rows_view(previous@)));
                    }
                    out
                },
            }
        },
    }
}

pub fn evaluate(plan: &VPlan, input: &VTable)
    -> (output: Result<Vec<Vec<Option<VValue>>>, ()>)
    requires table_valid(input),
    ensures
        match output {
            Ok(result) => relation_spec(plan, rows_view(input.rows@)) == Ok(rows_view(result@)),
            Err(_) => relation_spec(plan, rows_view(input.rows@)).is_Err(),
        },
{
    eval_plan_rows(plan, &input.rows)
}

/// Preserve row schemas when extending a relation.
pub proof fn lemma_rows_push_valid(
    rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>, row: Seq<Option<VValue>>,
)
    requires rows_in_schema(rows, schema), row_in_schema(row, schema),
    ensures rows_in_schema(rows.push(row), schema),
{
    assert forall|i: int| 0 <= i < rows.push(row).len()
        implies row_in_schema(rows.push(row)[i], schema) by {
        if i < rows.len() {
            assert(row_in_schema(rows[i], schema));
        } else {
            assert(i == rows.len());
        }
    }
}

/// One operator preserves well-typed rows and cannot return a runtime error.
pub proof fn lemma_apply_prefix_type_sound(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>, output_schema: Seq<VField>, count: nat,
)
    requires
        rows_in_schema(rows, schema),
        local_schema(plan, schema) == Some(output_schema),
        count <= rows.len(),
    ensures
        match apply_prefix_spec(plan, rows, count) {
            Ok(result) => rows_in_schema(result, output_schema),
            Err(_) => false,
        },
    decreases count,
{
    if count > 0 {
        lemma_apply_prefix_type_sound(plan, rows, schema, output_schema, (count - 1) as nat);
        let previous = apply_prefix_spec(plan, rows, (count - 1) as nat).unwrap();
        assert(row_in_schema(rows[(count - 1) as int], schema));
        lemma_local_type_sound(plan, schema, rows[(count - 1) as int], output_schema);
        if let Ok(Some(result)) = local_row_spec(plan, rows[(count - 1) as int]) {
            lemma_rows_push_valid(previous, output_schema, result);
        }
    }
}

/// Static typing suffices to establish success of complete relational execution.
pub proof fn lemma_relation_type_sound(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>, output_schema: Seq<VField>,
)
    requires
        rows_in_schema(rows, schema),
        plan_schema(plan, schema) == Some(output_schema),
    ensures
        match relation_spec(plan, rows) {
            Ok(result) => rows_in_schema(result, output_schema),
            Err(_) => false,
        },
    decreases *plan,
{
    match plan {
        VPlan::Input => {},
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_relation_type_sound(input, rows, schema, child_schema);
            let child_rows = relation_spec(input, rows).unwrap();
            assert(local_schema(plan, child_schema) == Some(output_schema));
            lemma_apply_prefix_type_sound(
                plan, child_rows, child_schema, output_schema, child_rows.len());
        },
    }
}

} // verus!

// Clone is needed by the OpenDP Frame carrier. Only the reference source
// needs to implement Clone; generic plan construction has no source bound.
impl<S: Clone> Clone for VLazyFrame<S> {
    fn clone(&self) -> Self {
        Self { plan: self.plan.clone(), source: self.source.clone() }
    }
}

// The recursive Clone implementation is ordinary executable Rust; Verus
// currently rejects its automatically derived termination dependencies.
impl Clone for VPlan {
    fn clone(&self) -> Self {
        match self {
            Self::Input => Self::Input,
            Self::Project { input, exprs } => Self::Project {
                input: input.clone(),
                exprs: exprs.clone(),
            },
            Self::Filter { input, predicate } => Self::Filter {
                input: input.clone(),
                predicate: predicate.clone(),
            },
        }
    }
}

impl Clone for VExpr {
    fn clone(&self) -> Self {
        match self {
            Self::Column(index) => Self::Column(*index),
            Self::Alias { expr, name } => Self::Alias {
                expr: expr.clone(),
                name: name.clone(),
            },
            Self::Literal(value) => Self::Literal(*value),
            Self::Null(dtype) => Self::Null(*dtype),
            Self::Binary { left, op, right } => Self::Binary {
                left: left.clone(),
                op: *op,
                right: right.clone(),
            },
            Self::Clip { expr, lower, upper } => Self::Clip {
                expr: expr.clone(),
                lower: lower.clone(),
                upper: upper.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bound_plan_collects_with_verified_reference_evaluator() {
        let table = VTable {
            schema: vec![
                VField { dtype: VDType::Int64, nullable: false },
                VField { dtype: VDType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(VValue::Int(5)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(9)), Some(VValue::Bool(false))],
                vec![Some(VValue::Int(1)), None],
            ],
        };
        let query = VLazyFrame::new(table)
            .filter(VExpr::Column(1))
            .project(vec![VExpr::Column(0)]);
        let collected = query.collect().unwrap();
        assert_eq!(collected.schema, vec![
            VField { dtype: VDType::Int64, nullable: false },
        ]);
        assert_eq!(collected.rows, vec![vec![Some(VValue::Int(5))]]);
    }

    #[test]
    fn opaque_source_can_build_plans_without_execution_contract() {
        struct OpaqueSource { label: &'static str }
        let query = VLazyFrame::new(OpaqueSource { label: "backend-handle" })
            .filter(VExpr::Column(0))
            .project(vec![VExpr::Column(0)]);
        assert_eq!(query.source.label, "backend-handle");
        let VPlan::Project { input, .. } = query.plan else {
            panic!("expected projection");
        };
        assert!(matches!(*input, VPlan::Filter { .. }));
        // An opaque source deliberately has no collect() implementation.
    }

    #[test]
    fn reference_collection_rejects_invalid_source_and_invalid_plan() {
        let malformed = VTable {
            schema: vec![VField { dtype: VDType::Boolean, nullable: false }],
            rows: vec![vec![Some(VValue::Int(5))]],
        };
        assert!(VLazyFrame::new(malformed).collect().is_err());

        let well_formed = VTable {
            schema: vec![VField { dtype: VDType::Boolean, nullable: false }],
            rows: vec![vec![Some(VValue::Bool(true))]],
        };
        assert!(VLazyFrame::new(well_formed)
            .filter(VExpr::Column(3)).collect().is_err());
    }

    #[test]
    fn nested_projection_and_nullable_filter() {
        let input = VTable {
            schema: vec![
                VField { dtype: VDType::Int64, nullable: true },
                VField { dtype: VDType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(VValue::Int(-2)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(2)), Some(VValue::Bool(true))],
                vec![None, Some(VValue::Bool(true))],
                vec![Some(VValue::Int(4)), None],
                vec![Some(VValue::Int(6)), Some(VValue::Bool(false))],
            ],
        };
        let plan = VPlan::Project {
            input: Box::new(VPlan::Filter {
                input: Box::new(VPlan::Project {
                    input: Box::new(VPlan::Input),
                    exprs: vec![
                        VExpr::Clip {
                            expr: Box::new(VExpr::Column(0)),
                            lower: Some(Box::new(VExpr::Literal(VValue::Int(0)))),
                            upper: Some(Box::new(VExpr::Literal(VValue::Int(5)))),
                        },
                        VExpr::Column(1),
                    ],
                }),
                predicate: VExpr::Binary {
                    left: Box::new(VExpr::Column(1)),
                    op: VBinaryOp::And,
                    right: Box::new(VExpr::Binary {
                        left: Box::new(VExpr::Column(0)),
                        op: VBinaryOp::Gt,
                        right: Box::new(VExpr::Literal(VValue::Int(1))),
                    }),
                },
            }),
            exprs: vec![VExpr::Column(0), VExpr::Null(VDType::Boolean)],
        };
        assert!(evaluate(&plan, &input) == Ok(vec![
            vec![Some(VValue::Int(2)), None],
        ]));
    }

    #[test]
    fn empty_tables_and_zero_column_rows() {
        let empty = VTable {
            schema: vec![VField { dtype: VDType::Int64, nullable: false }],
            rows: vec![],
        };
        assert!(evaluate(&VPlan::Input, &empty) == Ok(vec![]));

        let no_columns = VTable {
            schema: vec![],
            rows: vec![vec![], vec![]],
        };
        assert!(evaluate(&VPlan::Input, &no_columns) == Ok(vec![vec![], vec![]]));
    }

    #[test]
    fn null_propagation_and_three_valued_boolean_logic() {
        let t = VTable {
            schema: vec![
                VField { dtype: VDType::Int64, nullable: true },
                VField { dtype: VDType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(VValue::Int(5)), None],
                vec![None, Some(VValue::Bool(false))],
                vec![Some(VValue::Int(1)), Some(VValue::Bool(true))],
            ],
        };
        let less_than = VExpr::Binary {
            left: Box::new(VExpr::Column(0)), op: VBinaryOp::Lt,
            right: Box::new(VExpr::Literal(VValue::Int(3))),
        };
        assert!(eval_expr_row(&less_than, &t.rows[0]) == Ok(Some(VValue::Bool(false))));
        assert!(eval_expr_row(&less_than, &t.rows[1]) == Ok(None));
        assert!(eval_expr_row(&less_than, &t.rows[2]) == Ok(Some(VValue::Bool(true))));

        let and_false = VExpr::Binary {
            left: Box::new(VExpr::Column(1)), op: VBinaryOp::And,
            right: Box::new(VExpr::Literal(VValue::Bool(false))),
        };
        assert!(eval_expr_row(&and_false, &t.rows[0]) == Ok(Some(VValue::Bool(false))));
        assert!(eval_expr_row(&and_false, &t.rows[1]) == Ok(Some(VValue::Bool(false))));

        let and_true = VExpr::Binary {
            left: Box::new(VExpr::Column(1)), op: VBinaryOp::And,
            right: Box::new(VExpr::Literal(VValue::Bool(true))),
        };
        assert!(eval_expr_row(&and_true, &t.rows[0]) == Ok(None));
        assert!(eval_expr_row(&and_true, &t.rows[2]) == Ok(Some(VValue::Bool(true))));

        let clipped = VExpr::Clip {
            expr: Box::new(VExpr::Column(0)),
            lower: Some(Box::new(VExpr::Literal(VValue::Int(2)))),
            upper: Some(Box::new(VExpr::Literal(VValue::Int(4)))),
        };
        assert!(eval_expr_row(&clipped, &t.rows[0]) == Ok(Some(VValue::Int(4))));
        assert!(eval_expr_row(&clipped, &t.rows[1]) == Ok(None));
        assert!(eval_expr_row(&clipped, &t.rows[2]) == Ok(Some(VValue::Int(2))));
    }

    #[test]
    fn clip_expression_bounds_and_null_bounds() {
        let input = VTable {
            schema: vec![
                VField { dtype: VDType::Int64, nullable: true },
                VField { dtype: VDType::Int64, nullable: true },
                VField { dtype: VDType::Int64, nullable: true },
            ],
            rows: vec![
                vec![Some(VValue::Int(0)), Some(VValue::Int(2)), Some(VValue::Int(10))],
                vec![Some(VValue::Int(5)), None, Some(VValue::Int(3))],
                vec![Some(VValue::Int(8)), Some(VValue::Int(9)), Some(VValue::Int(6))],
                vec![None, Some(VValue::Int(0)), Some(VValue::Int(3))],
                vec![Some(VValue::Int(7)), Some(VValue::Int(8)), Some(VValue::Int(3))],
                vec![Some(VValue::Int(5)), None, None],
            ],
        };
        let expression = VExpr::Clip {
            expr: Box::new(VExpr::Column(0)),
            lower: Some(Box::new(VExpr::Column(1))),
            upper: Some(Box::new(VExpr::Column(2))),
        };
        let plan = VPlan::Project {
            input: Box::new(VPlan::Input),
            exprs: vec![expression],
        };
        assert!(evaluate(&plan, &input) == Ok(vec![
            vec![Some(VValue::Int(2))],
            vec![Some(VValue::Int(3))],
            vec![Some(VValue::Int(9))],
            vec![None],
            vec![Some(VValue::Int(8))],
            vec![Some(VValue::Int(5))],
        ]));

        let upper_only = VExpr::Clip {
            expr: Box::new(VExpr::Column(0)),
            lower: None,
            upper: Some(Box::new(VExpr::Column(2))),
        };
        assert!(eval_expr_row(&upper_only, &input.rows[1])
            == Ok(Some(VValue::Int(3))));
        assert!(eval_expr_row(&upper_only, &input.rows[5])
            == Ok(Some(VValue::Int(5))));

        let no_bounds = VExpr::Clip {
            expr: Box::new(VExpr::Column(0)),
            lower: None,
            upper: None,
        };
        assert!(eval_expr_row(&no_bounds, &input.rows[4])
            == Ok(Some(VValue::Int(7))));
    }

    #[test]
    fn alias_preserves_expression_value_and_type() {
        let input = VTable {
            schema: vec![VField { dtype: VDType::Int64, nullable: false }],
            rows: vec![vec![Some(VValue::Int(7))]],
        };
        let expr = VExpr::Alias {
            expr: Box::new(VExpr::Column(0)),
            name: "renamed".to_string(),
        };
        assert!(check_expr(&expr, &input.schema) == Some(input.schema[0]));
        assert!(eval_expr_row(&expr, &input.rows[0]) == Ok(Some(VValue::Int(7))));
    }

    #[test]
    fn duplicate_rows_are_preserved() {
        let input = VTable {
            schema: vec![VField { dtype: VDType::Int64, nullable: true }],
            rows: vec![
                vec![Some(VValue::Int(4))],
                vec![None],
                vec![Some(VValue::Int(4))],
            ],
        };
        assert!(evaluate(&VPlan::Input, &input) == Ok(input.rows));
    }
}

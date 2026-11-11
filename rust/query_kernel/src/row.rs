use vstd::prelude::*;
use vstd::multiset::Multiset;

verus! {

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DType { Int64, Boolean }

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value { Int(i64), Bool(bool) }

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Field {
    pub dtype: DType,
    pub nullable: bool,
}

/// Typed rows are the sole physical and logical representation in the
/// verified kernel. Arrow conversion is handled by an external adapter.
pub struct Table {
    pub schema: Vec<Field>,
    pub rows: Vec<Vec<Option<Value>>>,
}

pub open spec fn table_valid(table: &Table) -> bool {
    rows_in_schema(rows_view(table.rows@), table.schema@)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp { Lt, Gt, Eq, And }

pub enum Expr {
    Column(usize),
    Literal(Value),
    Null(DType),
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    Clip {
        expr: Box<Expr>,
        lower: Option<Box<Expr>>,
        upper: Option<Box<Expr>>,
    },
}

pub enum Plan {
    Input,
    Project { input: Box<Plan>, exprs: Vec<Expr> },
    Filter { input: Box<Plan>, predicate: Expr },
}

pub open spec fn value_dtype(value: Value) -> DType {
    match value {
        Value::Int(_) => DType::Int64,
        Value::Bool(_) => DType::Boolean,
    }
}

pub open spec fn expr_field(expr: &Expr, schema: Seq<Field>) -> Option<Field>
    decreases *expr,
{
    match expr {
        Expr::Column(index) =>
            if *index < schema.len() { Some(schema[*index as int]) } else { None },
        Expr::Literal(value) => Some(Field { dtype: value_dtype(*value), nullable: false }),
        Expr::Null(dtype) => Some(Field { dtype: *dtype, nullable: true }),
        Expr::Binary { left, op, right } => {
            match (expr_field(left, schema), expr_field(right, schema)) {
                (Some(l), Some(r)) => {
                    let valid = match op {
                        BinaryOp::Lt | BinaryOp::Gt =>
                            l.dtype == DType::Int64 && r.dtype == DType::Int64,
                        BinaryOp::Eq => l.dtype == r.dtype,
                        BinaryOp::And =>
                            l.dtype == DType::Boolean && r.dtype == DType::Boolean,
                    };
                    if valid {
                        Some(Field { dtype: DType::Boolean,
                            nullable: l.nullable || r.nullable })
                    } else { None }
                },
                _ => None,
            }
        },
        Expr::Clip { expr, lower, upper } => {
            match expr_field(expr, schema) {
                Some(field) if field.dtype == DType::Int64 => {
                    let lower_ok = match lower {
                        None => true,
                        Some(bound) => match expr_field(bound, schema) {
                            Some(bound_field) => bound_field.dtype == DType::Int64,
                            None => false,
                        },
                    };
                    let upper_ok = match upper {
                        None => true,
                        Some(bound) => match expr_field(bound, schema) {
                            Some(bound_field) => bound_field.dtype == DType::Int64,
                            None => false,
                        },
                    };
                    if lower_ok && upper_ok { Some(field) } else { None }
                },
                _ => None,
            }
        },
    }
}

pub open spec fn project_schema_prefix(
    exprs: Seq<Expr>, schema: Seq<Field>, count: nat,
) -> Option<Seq<Field>>
    recommends count <= exprs.len(),
    decreases count,
{
    if count == 0 { Some(Seq::empty()) }
    else {
        match (project_schema_prefix(exprs, schema, (count - 1) as nat),
               expr_field(&exprs[(count - 1) as int], schema)) {
            (Some(prefix), Some(field)) => Some(prefix.push(field)),
            _ => None,
        }
    }
}

/// Type checking for a single relational operator.
pub open spec fn local_schema(plan: &Plan, schema: Seq<Field>) -> Option<Seq<Field>> {
    match plan {
        Plan::Input => Some(schema),
        Plan::Project { exprs, .. } =>
            project_schema_prefix(exprs@, schema, exprs@.len()),
        Plan::Filter { predicate, .. } => match expr_field(predicate, schema) {
            Some(field) if field.dtype == DType::Boolean => Some(schema),
            _ => None,
        },
    }
}

pub open spec fn plan_schema(plan: &Plan, input_schema: Seq<Field>) -> Option<Seq<Field>>
    decreases *plan,
{
    match plan {
        Plan::Input => Some(input_schema),
        Plan::Project { input, .. } | Plan::Filter { input, .. } =>
            match plan_schema(input, input_schema) {
                Some(child_schema) => local_schema(plan, child_schema),
                None => None,
            },
    }
}

pub open spec fn cell_matches_field(value: Option<Value>, field: Field) -> bool {
    match value {
        Some(value) => value_dtype(value) == field.dtype,
        None => field.nullable,
    }
}

/// A semantic cell view allows generic, order-independent stability arguments.
pub open spec fn expr_row_spec(
    expr: &Expr, row: Seq<Option<Value>>,
) -> Result<Option<Value>, ()>
    decreases *expr,
{
    match expr {
        Expr::Column(index) =>
            if *index < row.len() { Ok(row[*index as int]) } else { Err(()) },
        Expr::Literal(value) => Ok(Some(*value)),
        Expr::Null(_) => Ok(None),
        Expr::Binary { left, op, right } =>
            match (expr_row_spec(left, row), expr_row_spec(right, row)) {
                (Ok(lhs), Ok(rhs)) => eval_binary(*op, lhs, rhs),
                _ => Err(()),
            },
        Expr::Clip { expr, lower, upper } => {
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
    row: Seq<Option<Value>>, schema: Seq<Field>,
) -> bool {
    row.len() == schema.len()
        && forall|i: int| 0 <= i < row.len()
            ==> cell_matches_field(row[i], schema[i])
}

pub open spec fn rows_in_schema(
    rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>,
) -> bool {
    forall|i: int| 0 <= i < rows.len() ==> row_in_schema(rows[i], schema)
}

/// Well-typed expressions cannot fail, and nullable outputs are represented
/// explicitly by None instead of conflating null with an evaluation error.
pub proof fn lemma_expr_field_sound(
    expr: &Expr, schema: Seq<Field>, row: Seq<Option<Value>>, field: Field,
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
        Expr::Column(index) => {
            assert(*index < schema.len());
            assert(cell_matches_field(row[*index as int], schema[*index as int]));
        },
        Expr::Literal(_) | Expr::Null(_) => {},
        Expr::Binary { left, op: _, right } => {
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
        Expr::Clip { expr: child, lower, upper } => {
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

pub proof fn lemma_project_row_prefix_type_sound(
    exprs: Seq<Expr>, schema: Seq<Field>,
    row: Seq<Option<Value>>, count: nat,
)
    requires
        row_in_schema(row, schema),
        count <= exprs.len(),
        project_schema_prefix(exprs, schema, count).is_Some(),
    ensures
        match (
            project_schema_prefix(exprs, schema, count),
            project_row_prefix(exprs, row, count),
        ) {
            (Some(fields), Ok(output)) => row_in_schema(output, fields),
            _ => false,
        },
    decreases count,
{
    if count > 0 {
        lemma_project_row_prefix_type_sound(
            exprs, schema, row, (count - 1) as nat);
        let field = expr_field(&exprs[(count - 1) as int], schema).unwrap();
        lemma_expr_field_sound(
            &exprs[(count - 1) as int], schema, row, field,
        );
        let prior_schema =
            project_schema_prefix(exprs, schema, (count - 1) as nat).unwrap();
        let prior =
            project_row_prefix(exprs, row, (count - 1) as nat).unwrap();
        let value = expr_row_spec(&exprs[(count - 1) as int], row).unwrap();
        assert(row_in_schema(prior, prior_schema));
        assert(cell_matches_field(value, field));
        assert(row_in_schema(prior.push(value), prior_schema.push(field))) by {
            assert forall|i: int| 0 <= i < prior.push(value).len()
                implies cell_matches_field(
                    prior.push(value)[i], prior_schema.push(field)[i]
                ) by {
                if i < prior.len() {
                    assert(cell_matches_field(prior[i], prior_schema[i]));
                } else {
                    assert(i == prior.len());
                }
            }
        }
    }
}

/// Typing soundness for one operator, without evaluating its child plan.
pub proof fn lemma_local_type_sound(
    plan: &Plan, schema: Seq<Field>, row: Seq<Option<Value>>, output_schema: Seq<Field>,
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
        Plan::Input => {},
        Plan::Project { exprs, .. } =>
            lemma_project_row_prefix_type_sound(exprs@, schema, row, exprs@.len()),
        Plan::Filter { predicate, .. } => {
            let field = expr_field(predicate, schema).unwrap();
            lemma_expr_field_sound(predicate, schema, row, field);
        },
    }
}

pub open spec fn project_row_prefix(
    exprs: Seq<Expr>, row: Seq<Option<Value>>, count: nat,
) -> Result<Seq<Option<Value>>, ()>
    recommends count <= exprs.len(),
    decreases count,
{
    if count == 0 { Ok(Seq::empty()) }
    else {
        match (project_row_prefix(exprs, row, (count - 1) as nat),
               expr_row_spec(&exprs[(count - 1) as int], row)) {
            (Ok(prefix), Ok(cell)) => Ok(prefix.push(cell)),
            _ => Err(()),
        }
    }
}

/// A row-local step does not evaluate the plan's child.
pub open spec fn local_row_spec(
    plan: &Plan, row: Seq<Option<Value>>,
) -> Result<Option<Seq<Option<Value>>>, ()> {
    match plan {
        Plan::Input => Ok(Some(row)),
        Plan::Project { exprs, .. } =>
            match project_row_prefix(exprs@, row, exprs@.len()) {
                Ok(out) => Ok(Some(out)),
                Err(_) => Err(()),
            },
        Plan::Filter { predicate, .. } =>
            match expr_row_spec(predicate, row) {
                Ok(Some(Value::Bool(true))) => Ok(Some(row)),
                Ok(Some(Value::Bool(false))) | Ok(None) => Ok(None),
                _ => Err(()),
            },
    }
}

pub open spec fn apply_prefix_spec(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>, count: nat,
) -> Result<Seq<Seq<Option<Value>>>, ()>
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
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
) -> Result<Seq<Seq<Option<Value>>>, ()> {
    apply_prefix_spec(plan, rows, rows.len())
}

/// A plan node transforms the complete child relation.
pub open spec fn relation_spec(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
) -> Result<Seq<Seq<Option<Value>>>, ()>
    decreases *plan,
{
    match plan {
        Plan::Input => Ok(rows),
        Plan::Project { input, .. } | Plan::Filter { input, .. } =>
            match relation_spec(input, rows) {
                Err(_) => Err(()),
                Ok(previous) => apply_spec(plan, previous),
            },
    }
}

/// Expressions read the same row type used by the relation.
pub fn eval_expr_row(expr: &Expr, row: &Vec<Option<Value>>)
    -> (result: Result<Option<Value>, ()>)
    ensures result == expr_row_spec(expr, row@),
    decreases *expr,
{
    match expr {
        Expr::Column(index) =>
            if *index < row.len() { Ok(row[*index]) } else { Err(()) },
        Expr::Literal(value) => Ok(Some(*value)),
        Expr::Null(_) => Ok(None),
        Expr::Binary { left, op, right } =>
            match (eval_expr_row(left, row), eval_expr_row(right, row)) {
                (Ok(lhs), Ok(rhs)) => eval_binary(*op, lhs, rhs),
                _ => Err(()),
            },
        Expr::Clip { expr, lower, upper } => {
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

pub fn eval_project_prefix(
    exprs: &Vec<Expr>, row: &Vec<Option<Value>>, count: usize,
) -> (result: Result<Vec<Option<Value>>, ()>)
    requires count <= exprs.len(),
    ensures
        match result {
            Ok(values) =>
                project_row_prefix(exprs@, row@, count as nat) == Ok(values@),
            Err(_) => project_row_prefix(exprs@, row@, count as nat).is_Err(),
        },
    decreases count,
{
    if count == 0 {
        Ok(Vec::new())
    } else {
        match eval_project_prefix(exprs, row, count - 1) {
            Err(_) => Err(()),
            Ok(mut prefix) => {
                match eval_expr_row(&exprs[count - 1], row) {
                    Err(_) => Err(()),
                    Ok(value) => {
                        prefix.push(value);
                        Ok(prefix)
                    },
                }
            },
        }
    }
}

/// Evaluate just one node's row-local operation.
pub fn eval_local_row(plan: &Plan, row: &Vec<Option<Value>>)
    -> (result: Result<Option<Vec<Option<Value>>>, ()>)
    ensures
        match result {
            Err(_) => local_row_spec(plan, row@).is_Err(),
            Ok(None) => local_row_spec(plan, row@) == Ok(None),
            Ok(Some(out)) => local_row_spec(plan, row@) == Ok(Some(out@)),
        },
{
    match plan {
        Plan::Input => Ok(Some(row.clone())),
        Plan::Project { exprs, .. } =>
            match eval_project_prefix(exprs, row, exprs.len()) {
                Ok(output) => Ok(Some(output)),
                Err(_) => Err(()),
            },
        Plan::Filter { predicate, .. } =>
            match eval_expr_row(predicate, row) {
                Ok(Some(Value::Bool(true))) => Ok(Some(row.clone())),
                Ok(Some(Value::Bool(false))) | Ok(None) => Ok(None),
                _ => Err(()),
            },
    }
}

pub open spec fn rows_view(rows: Seq<Vec<Option<Value>>>) -> Seq<Seq<Option<Value>>> {
    Seq::new(rows.len(), |i: int| rows[i]@)
}

pub proof fn lemma_rows_view_push(
    rows: Seq<Vec<Option<Value>>>, row: Vec<Option<Value>>,
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
    plan: &Plan, rows: &Vec<Vec<Option<Value>>>, count: usize,
) -> (output: Result<Vec<Vec<Option<Value>>>, ()>)
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
        let empty: Vec<Vec<Option<Value>>> = Vec::new();
        proof { assert(rows_view(empty@) =~= Seq::<Seq<Option<Value>>>::empty()); }
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
    plan: &Plan, rows: &Vec<Vec<Option<Value>>>,
) -> (output: Result<Vec<Vec<Option<Value>>>, ()>)
    ensures
        match output {
            Ok(result) => relation_spec(plan, rows_view(rows@)) == Ok(rows_view(result@)),
            Err(_) => relation_spec(plan, rows_view(rows@)).is_Err(),
        },
    decreases *plan,
{
    match plan {
        Plan::Input => Ok(rows.clone()),
        Plan::Project { input, .. } | Plan::Filter { input, .. } =>
            match eval_plan_rows(input, rows) {
                Err(_) => Err(()),
                Ok(previous) => eval_apply_prefix(plan, &previous, previous.len()),
            },
    }
}

pub fn evaluate(plan: &Plan, input: &Table)
    -> (output: Result<Vec<Vec<Option<Value>>>, ()>)
    requires table_valid(input),
    ensures
        match output {
            Ok(result) => relation_spec(plan, rows_view(input.rows@)) == Ok(rows_view(result@)),
            Err(_) => relation_spec(plan, rows_view(input.rows@)).is_Err(),
        },
{
    eval_plan_rows(plan, &input.rows)
}

pub open spec fn dataset_spec(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
) -> Result<Multiset<Seq<Option<Value>>>, ()> {
    match relation_spec(plan, rows) {
        Ok(result) => Ok(result.to_multiset()),
        Err(_) => Err(()),
    }
}

pub open spec fn seq_one_insert(
    before: Seq<Seq<Option<Value>>>, after: Seq<Seq<Option<Value>>>,
) -> bool {
    before =~= after
        || exists|i: int, row: Seq<Option<Value>>|
            0 <= i <= before.len() && after =~= before.insert(i, row)
}

pub open spec fn bag_one_edit(
    left: Multiset<Seq<Option<Value>>>, right: Multiset<Seq<Option<Value>>>,
) -> bool {
    left =~= right
        || exists|row: Seq<Option<Value>>| right =~= left.insert(row)
        || exists|row: Seq<Option<Value>>| left =~= right.insert(row)
}

/// Preserve row schemas when extending a relation.
pub proof fn lemma_rows_push_valid(
    rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>, row: Seq<Option<Value>>,
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

pub proof fn lemma_rows_insert_valid(
    rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>,
    index: int, row: Seq<Option<Value>>,
)
    requires
        rows_in_schema(rows, schema),
        row_in_schema(row, schema),
        0 <= index <= rows.len(),
    ensures rows_in_schema(rows.insert(index, row), schema),
{
    rows.insert_ensures(index, row);
    assert forall|i: int| 0 <= i < rows.insert(index, row).len()
        implies row_in_schema(rows.insert(index, row)[i], schema) by {
        if i == index {
            assert(rows.insert(index, row)[i] == row);
        } else if i < index {
            assert(rows.insert(index, row)[i] == rows[i]);
        } else {
            assert(rows.insert(index, row)[i] == rows[i - 1]);
        }
    }
}

pub proof fn lemma_rows_drop_last_valid(
    rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>,
)
    requires rows_in_schema(rows, schema), rows.len() > 0,
    ensures
        rows_in_schema(rows.drop_last(), schema),
        row_in_schema(rows.last(), schema),
{
    assert forall|i: int| 0 <= i < rows.drop_last().len()
        implies row_in_schema(rows.drop_last()[i], schema) by {
        assert(rows.drop_last()[i] == rows[i]);
    }
    assert(rows.last() == rows[rows.len() as int - 1]);
}

/// One operator preserves well-typed rows and cannot return a runtime error.
pub proof fn lemma_apply_prefix_type_sound(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
    schema: Seq<Field>, output_schema: Seq<Field>, count: nat,
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
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
    schema: Seq<Field>, output_schema: Seq<Field>,
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
        Plan::Input => {},
        Plan::Project { input, .. } | Plan::Filter { input, .. } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_relation_type_sound(input, rows, schema, child_schema);
            let child_rows = relation_spec(input, rows).unwrap();
            assert(local_schema(plan, child_schema) == Some(output_schema));
            lemma_apply_prefix_type_sound(
                plan, child_rows, child_schema, output_schema, child_rows.len());
        },
    }
}

/// A prefix of the input cannot be influenced by later rows.
pub proof fn lemma_apply_prefix_ignores_last(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>, count: nat,
)
    requires count < rows.len(),
    ensures
        apply_prefix_spec(plan, rows, count)
            == apply_prefix_spec(plan, rows.drop_last(), count),
    decreases count,
{
    if count > 0 {
        lemma_apply_prefix_ignores_last(plan, rows, (count - 1) as nat);
        assert(rows.drop_last()[(count - 1) as int] == rows[(count - 1) as int]);
    }
}

/// Inserting before the final element commutes with appending that element.
pub proof fn lemma_seq_insert_push<T>(xs: Seq<T>, index: int, row: T, tail: T)
    requires 0 <= index <= xs.len(),
    ensures xs.insert(index, row).push(tail) =~= xs.push(tail).insert(index, row),
{
    xs.insert_ensures(index, row);
    xs.push(tail).insert_ensures(index, row);
    assert forall|i: int| 0 <= i < xs.insert(index, row).push(tail).len()
        implies xs.insert(index, row).push(tail)[i]
            == xs.push(tail).insert(index, row)[i] by {
        if i < index {
        } else if i == index {
        } else if i <= xs.len() {
        } else {
            assert(i == xs.len() + 1);
        }
    }
}

/// A single local operator preserves one-record insertion.
pub proof fn lemma_apply_insert(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>,
    output_schema: Seq<Field>, index: int, row: Seq<Option<Value>>,
)
    requires
        rows_in_schema(rows, schema),
        row_in_schema(row, schema),
        local_schema(plan, schema) == Some(output_schema),
        0 <= index <= rows.len(),
    ensures
        match (
            apply_spec(plan, rows),
            apply_spec(plan, rows.insert(index, row)),
            local_row_spec(plan, row),
        ) {
            (Ok(before), Ok(after), Ok(Some(added))) =>
                exists|i: int| 0 <= i <= before.len() && after =~= before.insert(i, added),
            (Ok(before), Ok(after), Ok(None)) => after =~= before,
            _ => false,
        },
    decreases rows.len(),
{
    lemma_apply_prefix_type_sound(plan, rows, schema, output_schema, rows.len());
    lemma_rows_insert_valid(rows, schema, index, row);
    lemma_apply_prefix_type_sound(
        plan, rows.insert(index, row), schema, output_schema,
        rows.insert(index, row).len());
    lemma_local_type_sound(plan, schema, row, output_schema);
    if index == rows.len() {
        let appended = rows.push(row);
        assert(rows.insert(index, row) =~= appended);
        assert(appended.drop_last() =~= rows);
        lemma_apply_prefix_ignores_last(plan, appended, rows.len());
        assert(apply_prefix_spec(plan, appended, rows.len()) == apply_spec(plan, rows));
        match local_row_spec(plan, row) {
            Ok(Some(added)) => {
                let before = apply_spec(plan, rows).unwrap();
                assert(before.push(added) =~= before.insert(before.len() as int, added));
                assert(exists|i: int| 0 <= i <= before.len()
                    && before.push(added) =~= before.insert(i, added));
            },
            Ok(None) => {},
            Err(_) => { assert(false); },
        }
    } else {
        let tail = rows.last();
        let prefix = rows.drop_last();
        let inserted = rows.insert(index, row);
        assert(index <= prefix.len());
        assert(inserted.last() == tail);
        assert(inserted.drop_last() =~= prefix.insert(index, row));
        lemma_rows_drop_last_valid(rows, schema);
        lemma_rows_insert_valid(prefix, schema, index, row);
        lemma_apply_insert(plan, prefix, schema, output_schema, index, row);
        lemma_apply_prefix_ignores_last(plan, rows, prefix.len());
        lemma_apply_prefix_ignores_last(plan, inserted, prefix.insert(index, row).len());
        assert(apply_prefix_spec(plan, rows, prefix.len()) == apply_spec(plan, prefix));
        assert(apply_prefix_spec(plan, inserted, prefix.insert(index, row).len())
            == apply_spec(plan, prefix.insert(index, row)));
        let before_prefix = apply_spec(plan, prefix).unwrap();
        let after_prefix = apply_spec(plan, prefix.insert(index, row)).unwrap();
        lemma_local_type_sound(plan, schema, tail, output_schema);
        match (local_row_spec(plan, row), local_row_spec(plan, tail)) {
            (Ok(Some(added)), Ok(Some(tail_result))) => {
                let j = choose|j: int| 0 <= j <= before_prefix.len()
                    && after_prefix =~= before_prefix.insert(j, added);
                lemma_seq_insert_push(before_prefix, j, added, tail_result);
                assert(exists|i: int| 0 <= i <= before_prefix.push(tail_result).len()
                    && after_prefix.push(tail_result)
                        =~= before_prefix.push(tail_result).insert(i, added));
            },
            (Ok(Some(added)), Ok(None)) => {
                let j = choose|j: int| 0 <= j <= before_prefix.len()
                    && after_prefix =~= before_prefix.insert(j, added);
                assert(exists|i: int| 0 <= i <= before_prefix.len()
                    && after_prefix =~= before_prefix.insert(i, added));
            },
            (Ok(None), Ok(Some(_))) | (Ok(None), Ok(None)) => {
                assert(after_prefix =~= before_prefix);
            },
            _ => { assert(false); },
        }
    }
}

/// The relational composition of one-insertion-stable steps is stable.
pub proof fn lemma_relation_insert(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
    schema: Seq<Field>, output_schema: Seq<Field>,
    index: int, row: Seq<Option<Value>>,
)
    requires
        rows_in_schema(rows, schema),
        row_in_schema(row, schema),
        plan_schema(plan, schema) == Some(output_schema),
        0 <= index <= rows.len(),
    ensures
        match (
            relation_spec(plan, rows),
            relation_spec(plan, rows.insert(index, row)),
        ) {
            (Ok(before), Ok(after)) => seq_one_insert(before, after),
            _ => false,
        },
    decreases *plan,
{
    lemma_rows_insert_valid(rows, schema, index, row);
    lemma_relation_type_sound(plan, rows, schema, output_schema);
    lemma_relation_type_sound(
        plan, rows.insert(index, row), schema, output_schema);
    match plan {
        Plan::Input => {
            assert(seq_one_insert(rows, rows.insert(index, row)));
        },
        Plan::Project { input, .. } | Plan::Filter { input, .. } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_relation_insert(input, rows, schema, child_schema, index, row);
            let before_child = relation_spec(input, rows).unwrap();
            let after_child = relation_spec(input, rows.insert(index, row)).unwrap();
            if before_child =~= after_child {
                assert(relation_spec(plan, rows) == relation_spec(plan, rows.insert(index, row)));
            } else {
                let (j, inserted_row) = choose|j: int, inserted_row: Seq<Option<Value>>|
                    0 <= j <= before_child.len()
                        && after_child =~= before_child.insert(j, inserted_row);
                assert(rows_in_schema(after_child, child_schema));
                before_child.insert_ensures(j, inserted_row);
                assert(row_in_schema(inserted_row, child_schema));
                assert(rows_in_schema(before_child, child_schema));
                assert(local_schema(plan, child_schema) == Some(output_schema));
                lemma_apply_insert(
                    plan, before_child, child_schema, output_schema, j, inserted_row);
                match (
                    apply_spec(plan, before_child),
                    apply_spec(plan, after_child),
                    local_row_spec(plan, inserted_row),
                ) {
                    (Ok(before), Ok(after), Ok(Some(added))) => {
                        assert(seq_one_insert(before, after));
                    },
                    (Ok(before), Ok(after), Ok(None)) => {
                        assert(before =~= after);
                    },
                    _ => { assert(false); },
                }
            }
        },
    }
}

/// Moving an inserted row within a sequence does not alter its multiplicity.
pub proof fn lemma_seq_insert_bag<T>(xs: Seq<T>, index: int, value: T)
    requires 0 <= index <= xs.len(),
    ensures xs.insert(index, value).to_multiset() =~= xs.to_multiset().insert(value),
    decreases xs.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if index == xs.len() {
        assert(xs.insert(index, value) =~= xs.push(value));
        xs.to_multiset_ensures();
    } else {
        let tail = xs.last();
        let prefix = xs.drop_last();
        assert(index <= prefix.len());
        lemma_seq_insert_bag(prefix, index, value);
        assert(xs.insert(index, value).last() == tail);
        assert(xs.insert(index, value).drop_last() =~= prefix.insert(index, value));
        assert(xs.insert(index, value) =~= prefix.insert(index, value).push(tail));
        assert(xs =~= prefix.push(tail));
        prefix.to_multiset_ensures();
        prefix.insert(index, value).to_multiset_ensures();
        assert(xs.insert(index, value).to_multiset() =~=
            prefix.to_multiset().insert(value).insert(tail));
        assert(xs.to_multiset().insert(value) =~=
            prefix.to_multiset().insert(tail).insert(value));
        assert(prefix.to_multiset().insert(value).insert(tail) =~=
            prefix.to_multiset().insert(tail).insert(value));
    }
}

pub proof fn lemma_seq_one_insert_bag(
    before: Seq<Seq<Option<Value>>>, after: Seq<Seq<Option<Value>>>,
)
    requires seq_one_insert(before, after),
    ensures bag_one_edit(before.to_multiset(), after.to_multiset()),
{
    if !(before =~= after) {
        let (index, row) = choose|index: int, row: Seq<Option<Value>>|
            0 <= index <= before.len() && after =~= before.insert(index, row);
        lemma_seq_insert_bag(before, index, row);
    }
}

/// Typed row insertion is enough for the symmetric-distance bound.
pub open spec fn table_one_insert(before: &Table, after: &Table, index: int) -> bool {
    let earlier = rows_view(before.rows@);
    let later = rows_view(after.rows@);
    before.schema@ =~= after.schema@
        && 0 <= index <= earlier.len()
        && later.len() == earlier.len() + 1
        && later =~= earlier.insert(index, later[index])
}

pub proof fn lemma_typed_row_insert_stable(
    plan: &Plan, before: &Table, after: &Table, index: int,
)
    requires
        table_valid(before),
        table_valid(after),
        table_one_insert(before, after, index),
        plan_schema(plan, before.schema@).is_Some(),
    ensures
        match (
            dataset_spec(plan, rows_view(before.rows@)),
            dataset_spec(plan, rows_view(after.rows@)),
        ) {
            (Ok(before_bag), Ok(after_bag)) =>
                bag_one_edit(before_bag, after_bag),
            _ => false,
        },
{
    let previous = rows_view(before.rows@);
    let next = rows_view(after.rows@);
    let output_schema = plan_schema(plan, before.schema@).unwrap();
    assert(plan_schema(plan, after.schema@) == Some(output_schema));
    assert(next =~= previous.insert(index, next[index]));
    assert(row_in_schema(next[index], before.schema@));
    lemma_relation_insert(
        plan, previous, before.schema@, output_schema, index, next[index]);
    match (
        relation_spec(plan, previous),
        relation_spec(plan, next),
    ) {
        (Ok(before_rows), Ok(after_rows)) => {
            lemma_seq_one_insert_bag(before_rows, after_rows);
        },
        _ => { assert(false); },
    }
}


} // verus!

/// Single source of truth for scalar semantics: dual_spec generates the
/// open specification used by the row interpreter and typing proofs.
#[verus_verify(dual_spec, open)]
#[verus_spec(returns eval_binary(op, lhs, rhs))]
pub fn eval_binary(
    op: BinaryOp, lhs: Option<Value>, rhs: Option<Value>,
) -> Result<Option<Value>, ()> {
    match op {
        BinaryOp::Lt => match (lhs, rhs) {
            (Some(Value::Int(a)), Some(Value::Int(b))) =>
                Ok(Some(Value::Bool(a < b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        BinaryOp::Gt => match (lhs, rhs) {
            (Some(Value::Int(a)), Some(Value::Int(b))) =>
                Ok(Some(Value::Bool(a > b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        BinaryOp::Eq => match (lhs, rhs) {
            (Some(Value::Int(a)), Some(Value::Int(b))) =>
                Ok(Some(Value::Bool(a == b))),
            (Some(Value::Bool(a)), Some(Value::Bool(b))) =>
                Ok(Some(Value::Bool(a == b))),
            (None, _) | (_, None) => Ok(None),
            _ => Err(()),
        },
        BinaryOp::And => match (lhs, rhs) {
            (Some(Value::Bool(a)), Some(Value::Bool(b))) =>
                Ok(Some(Value::Bool(a && b))),
            (Some(Value::Bool(false)), None) | (None, Some(Value::Bool(false))) =>
                Ok(Some(Value::Bool(false))),
            (Some(Value::Bool(true)), None)
                | (None, Some(Value::Bool(true)))
                | (None, None) => Ok(None),
            _ => Err(()),
        },
    }
}

/// The outer Option indicates a missing bound. The inner Option is the
/// evaluated bound, which can be null. Null bounds are ignored in Polars.
#[verus_verify(dual_spec, open)]
#[verus_spec(returns eval_clip(value, lower, upper))]
pub fn eval_clip(
    value: Option<Value>,
    lower: Option<Option<Value>>,
    upper: Option<Option<Value>>,
) -> Result<Option<Value>, ()> {
    match (value, lower, upper) {
        (None, _, _) => Ok(None),
        (Some(Value::Int(_)), Some(Some(Value::Bool(_))), _)
            | (Some(Value::Int(_)), _, Some(Some(Value::Bool(_)))) => Err(()),
        (Some(Value::Int(v)), lower, upper) => {
            let low = match lower {
                Some(Some(Value::Int(n))) => Some(n),
                _ => None,
            };
            let high = match upper {
                Some(Some(Value::Int(n))) => Some(n),
                _ => None,
            };
            // Use the original input for both comparisons. Polars tests
            // establish that the lower bound takes precedence if both apply.
            let clipped = match low {
                Some(n) if v < n => n,
                _ => match high {
                    Some(n) if v > n => n,
                    _ => v,
                },
            };
            Ok(Some(Value::Int(clipped)))
        },
        _ => Err(()),
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_projection_and_nullable_filter() {
        let input = Table {
            schema: vec![
                Field { dtype: DType::Int64, nullable: true },
                Field { dtype: DType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(Value::Int(-2)), Some(Value::Bool(true))],
                vec![Some(Value::Int(2)), Some(Value::Bool(true))],
                vec![None, Some(Value::Bool(true))],
                vec![Some(Value::Int(4)), None],
                vec![Some(Value::Int(6)), Some(Value::Bool(false))],
            ],
        };
        let plan = Plan::Project {
            input: Box::new(Plan::Filter {
                input: Box::new(Plan::Project {
                    input: Box::new(Plan::Input),
                    exprs: vec![
                        Expr::Clip {
                            expr: Box::new(Expr::Column(0)),
                            lower: Some(Box::new(Expr::Literal(Value::Int(0)))),
                            upper: Some(Box::new(Expr::Literal(Value::Int(5)))),
                        },
                        Expr::Column(1),
                    ],
                }),
                predicate: Expr::Binary {
                    left: Box::new(Expr::Column(1)),
                    op: BinaryOp::And,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Column(0)),
                        op: BinaryOp::Gt,
                        right: Box::new(Expr::Literal(Value::Int(1))),
                    }),
                },
            }),
            exprs: vec![Expr::Column(0), Expr::Null(DType::Boolean)],
        };
        assert!(evaluate(&plan, &input) == Ok(vec![
            vec![Some(Value::Int(2)), None],
        ]));
    }

    #[test]
    fn empty_tables_and_zero_column_rows() {
        let empty = Table {
            schema: vec![Field { dtype: DType::Int64, nullable: false }],
            rows: vec![],
        };
        assert!(evaluate(&Plan::Input, &empty) == Ok(vec![]));

        let no_columns = Table {
            schema: vec![],
            rows: vec![vec![], vec![]],
        };
        assert!(evaluate(&Plan::Input, &no_columns) == Ok(vec![vec![], vec![]]));
    }

    #[test]
    fn null_propagation_and_three_valued_boolean_logic() {
        let t = Table {
            schema: vec![
                Field { dtype: DType::Int64, nullable: true },
                Field { dtype: DType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(Value::Int(5)), None],
                vec![None, Some(Value::Bool(false))],
                vec![Some(Value::Int(1)), Some(Value::Bool(true))],
            ],
        };
        let less_than = Expr::Binary {
            left: Box::new(Expr::Column(0)), op: BinaryOp::Lt,
            right: Box::new(Expr::Literal(Value::Int(3))),
        };
        assert!(eval_expr_row(&less_than, &t.rows[0]) == Ok(Some(Value::Bool(false))));
        assert!(eval_expr_row(&less_than, &t.rows[1]) == Ok(None));
        assert!(eval_expr_row(&less_than, &t.rows[2]) == Ok(Some(Value::Bool(true))));

        let and_false = Expr::Binary {
            left: Box::new(Expr::Column(1)), op: BinaryOp::And,
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert!(eval_expr_row(&and_false, &t.rows[0]) == Ok(Some(Value::Bool(false))));
        assert!(eval_expr_row(&and_false, &t.rows[1]) == Ok(Some(Value::Bool(false))));

        let and_true = Expr::Binary {
            left: Box::new(Expr::Column(1)), op: BinaryOp::And,
            right: Box::new(Expr::Literal(Value::Bool(true))),
        };
        assert!(eval_expr_row(&and_true, &t.rows[0]) == Ok(None));
        assert!(eval_expr_row(&and_true, &t.rows[2]) == Ok(Some(Value::Bool(true))));

        let clipped = Expr::Clip {
            expr: Box::new(Expr::Column(0)),
            lower: Some(Box::new(Expr::Literal(Value::Int(2)))),
            upper: Some(Box::new(Expr::Literal(Value::Int(4)))),
        };
        assert!(eval_expr_row(&clipped, &t.rows[0]) == Ok(Some(Value::Int(4))));
        assert!(eval_expr_row(&clipped, &t.rows[1]) == Ok(None));
        assert!(eval_expr_row(&clipped, &t.rows[2]) == Ok(Some(Value::Int(2))));
    }

    #[test]
    fn clip_expression_bounds_and_null_bounds() {
        let input = Table {
            schema: vec![
                Field { dtype: DType::Int64, nullable: true },
                Field { dtype: DType::Int64, nullable: true },
                Field { dtype: DType::Int64, nullable: true },
            ],
            rows: vec![
                vec![Some(Value::Int(0)), Some(Value::Int(2)), Some(Value::Int(10))],
                vec![Some(Value::Int(5)), None, Some(Value::Int(3))],
                vec![Some(Value::Int(8)), Some(Value::Int(9)), Some(Value::Int(6))],
                vec![None, Some(Value::Int(0)), Some(Value::Int(3))],
                vec![Some(Value::Int(7)), Some(Value::Int(8)), Some(Value::Int(3))],
                vec![Some(Value::Int(5)), None, None],
            ],
        };
        let expression = Expr::Clip {
            expr: Box::new(Expr::Column(0)),
            lower: Some(Box::new(Expr::Column(1))),
            upper: Some(Box::new(Expr::Column(2))),
        };
        let plan = Plan::Project {
            input: Box::new(Plan::Input),
            exprs: vec![expression],
        };
        assert!(evaluate(&plan, &input) == Ok(vec![
            vec![Some(Value::Int(2))],
            vec![Some(Value::Int(3))],
            vec![Some(Value::Int(9))],
            vec![None],
            vec![Some(Value::Int(8))],
            vec![Some(Value::Int(5))],
        ]));

        let upper_only = Expr::Clip {
            expr: Box::new(Expr::Column(0)),
            lower: None,
            upper: Some(Box::new(Expr::Column(2))),
        };
        assert!(eval_expr_row(&upper_only, &input.rows[1])
            == Ok(Some(Value::Int(3))));
        assert!(eval_expr_row(&upper_only, &input.rows[5])
            == Ok(Some(Value::Int(5))));

        let no_bounds = Expr::Clip {
            expr: Box::new(Expr::Column(0)),
            lower: None,
            upper: None,
        };
        assert!(eval_expr_row(&no_bounds, &input.rows[4])
            == Ok(Some(Value::Int(7))));
    }

    #[test]
    fn duplicate_rows_are_preserved() {
        let input = Table {
            schema: vec![Field { dtype: DType::Int64, nullable: true }],
            rows: vec![
                vec![Some(Value::Int(4))],
                vec![None],
                vec![Some(Value::Int(4))],
            ],
        };
        assert!(evaluate(&Plan::Input, &input) == Ok(input.rows));
    }
}

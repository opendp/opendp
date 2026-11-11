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

// Nulls are absent values, not a third physical dtype.
pub enum Column {
    Int64(Vec<Option<i64>>),
    Boolean(Vec<Option<bool>>),
}

pub struct Table {
    pub schema: Vec<Field>,
    pub columns: Vec<Column>,
    pub nrows: usize,
}

pub open spec fn column_len(column: &Column) -> nat {
    match column {
        Column::Int64(values) => values@.len(),
        Column::Boolean(values) => values@.len(),
    }
}

pub open spec fn column_dtype(column: &Column) -> DType {
    match column {
        Column::Int64(_) => DType::Int64,
        Column::Boolean(_) => DType::Boolean,
    }
}

pub open spec fn column_cell(column: &Column, index: int) -> Option<Value>
    recommends 0 <= index < column_len(column),
{
    match column {
        Column::Int64(values) => match values@[index] {
            Some(value) => Some(Value::Int(value)),
            None => None,
        },
        Column::Boolean(values) => match values@[index] {
            Some(value) => Some(Value::Bool(value)),
            None => None,
        },
    }
}

pub open spec fn row_at(table: &Table, index: int) -> Seq<Option<Value>>
    recommends 0 <= index < table.nrows,
{
    Seq::new(table.columns@.len(), |j: int|
        column_cell(&table.columns@[j], index))
}

pub open spec fn column_valid(column: &Column, field: Field, nrows: nat) -> bool {
    column_dtype(column) == field.dtype
        && column_len(column) == nrows
        && (field.nullable || forall|i: int| 0 <= i < nrows
            ==> column_cell(column, i).is_Some())
}

pub open spec fn table_valid(table: &Table) -> bool {
    table.schema@.len() == table.columns@.len()
        && forall|j: int| 0 <= j < table.columns@.len()
            ==> column_valid(&table.columns@[j], table.schema@[j], table.nrows as nat)
}

pub fn read_cell(column: &Column, index: usize) -> (value: Option<Value>)
    requires index < column_len(column),
    ensures value == column_cell(column, index as int),
{
    match column {
        Column::Int64(values) => match values[index] {
            Some(value) => Some(Value::Int(value)),
            None => None,
        },
        Column::Boolean(values) => match values[index] {
            Some(value) => Some(Value::Bool(value)),
            None => None,
        },
    }
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

pub open spec fn plan_schema(plan: &Plan, input_schema: Seq<Field>) -> Option<Seq<Field>>
    decreases *plan,
{
    match plan {
        Plan::Input => Some(input_schema),
        Plan::Project { input, exprs } => {
            match plan_schema(input, input_schema) {
                Some(fields) => project_schema_prefix(exprs@, fields, exprs@.len()),
                None => None,
            }
        },
        Plan::Filter { input, predicate } => {
            match plan_schema(input, input_schema) {
                Some(fields) =>
                    match expr_field(predicate, fields) {
                        Some(field) =>
                            if field.dtype == DType::Boolean { Some(fields) } else { None },
                        None => None,
                    },
                None => None,
            }
        },
    }
}

pub open spec fn cell_matches_field(value: Option<Value>, field: Field) -> bool {
    match value {
        Some(value) => value_dtype(value) == field.dtype,
        None => field.nullable,
    }
}

/// The row interpretation is ghost-only. Executable data is columnar.
pub open spec fn table_rows(table: &Table) -> Seq<Seq<Option<Value>>> {
    Seq::new(table.nrows as nat, |i: int| row_at(table, i))
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

pub proof fn lemma_plan_type_sound(
    plan: &Plan, schema: Seq<Field>,
    row: Seq<Option<Value>>, output_schema: Seq<Field>,
)
    requires
        row_in_schema(row, schema),
        plan_schema(plan, schema) == Some(output_schema),
    ensures
        match row_spec(plan, row) {
            Ok(Some(result)) => row_in_schema(result, output_schema),
            Ok(None) => true,
            Err(_) => false,
        },
    decreases *plan,
{
    match plan {
        Plan::Input => {},
        Plan::Project { input, exprs } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_plan_type_sound(input, schema, row, child_schema);
            if let Ok(Some(previous)) = row_spec(input, row) {
                lemma_project_row_prefix_type_sound(
                    exprs@, child_schema, previous, exprs@.len());
            }
        },
        Plan::Filter { input, predicate } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_plan_type_sound(input, schema, row, child_schema);
            if let Ok(Some(previous)) = row_spec(input, row) {
                let predicate_field = expr_field(predicate, child_schema).unwrap();
                lemma_expr_field_sound(predicate, child_schema, previous, predicate_field);
            }
        },
    }
}

/// A valid columnar table establishes the same row domain used by the
/// stability theorem; no separate schema assumptions are needed per row.
pub proof fn lemma_table_row_valid(table: &Table, index: int)
    requires table_valid(table), 0 <= index < table.nrows,
    ensures row_in_schema(row_at(table, index), table.schema@),
{
    assert(row_at(table, index).len() == table.schema@.len());
    assert forall|j: int| 0 <= j < row_at(table, index).len()
        implies cell_matches_field(
            row_at(table, index)[j], table.schema@[j]
        ) by {
        assert(column_valid(
            &table.columns@[j], table.schema@[j], table.nrows as nat));
        assert(row_at(table, index)[j] ==
            column_cell(&table.columns@[j], index));
        match column_cell(&table.columns@[j], index) {
            Some(_) => {},
            None => {
                assert(table.schema@[j].nullable);
            },
        }
    }
}

pub proof fn lemma_schema_establishes_domain(plan: &Plan, table: &Table)
    requires table_valid(table), plan_schema(plan, table.schema@).is_Some(),
    ensures input_domain(plan, table_rows(table)),
{
    let out_schema = plan_schema(plan, table.schema@).unwrap();
    assert forall|i: int| 0 <= i < table_rows(table).len()
        implies row_spec(plan, table_rows(table)[i]).is_Ok() by {
        lemma_table_row_valid(table, i);
        assert(table_rows(table)[i] == row_at(table, i));
        lemma_plan_type_sound(plan, table.schema@, row_at(table, i), out_schema);
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

pub open spec fn row_spec(
    plan: &Plan, row: Seq<Option<Value>>,
) -> Result<Option<Seq<Option<Value>>>, ()>
    decreases *plan,
{
    match plan {
        Plan::Input => Ok(Some(row)),
        Plan::Project { input, exprs } =>
            match row_spec(input, row) {
                Ok(Some(previous)) => match project_row_prefix(
                    exprs@, previous, exprs@.len()) {
                    Ok(result) => Ok(Some(result)),
                    Err(_) => Err(()),
                },
                Ok(None) => Ok(None),
                Err(_) => Err(()),
            },
        Plan::Filter { input, predicate } =>
            match row_spec(input, row) {
                Ok(Some(previous)) => match expr_row_spec(predicate, previous) {
                    Ok(Some(Value::Bool(true))) => Ok(Some(previous)),
                    Ok(Some(Value::Bool(false))) | Ok(None) => Ok(None),
                    _ => Err(()),
                },
                Ok(None) => Ok(None),
                Err(_) => Err(()),
            },
    }
}


/// The physical source remains columnar. Only an individual record is
/// assembled transiently for a row-local interpreter.
pub fn read_row_prefix(table: &Table, index: usize, count: usize)
    -> (result: Vec<Option<Value>>)
    requires
        table_valid(table),
        index < table.nrows,
        count <= table.columns.len(),
    ensures
        result@ =~= row_at(table, index as int).take(count as int),
    decreases count,
{
    if count == 0 {
        Vec::new()
    } else {
        let mut prior = read_row_prefix(table, index, count - 1);
        proof {
            assert(column_valid(
                &table.columns@[(count - 1) as int],
                table.schema@[(count - 1) as int],
                table.nrows as nat,
            ));
            assert(column_len(&table.columns@[(count - 1) as int]) == table.nrows);
        }
        let value = read_cell(&table.columns[count - 1], index);
        prior.push(value);
        proof {
            assert forall|j: int| 0 <= j < count
                implies prior@[j] == row_at(table, index as int).take(count as int)[j] by {
                if j < count - 1 {
                    assert(prior@[j] ==
                        row_at(table, index as int).take((count - 1) as int)[j]);
                } else {
                    assert(j == count - 1);
                    assert(row_at(table, index as int)[j] ==
                        column_cell(&table.columns@[j], index as int));
                }
            }
        }
        prior
    }
}

pub fn read_row(table: &Table, index: usize) -> (row: Vec<Option<Value>>)
    requires table_valid(table), index < table.nrows,
    ensures row@ =~= row_at(table, index as int),
{
    read_row_prefix(table, index, table.columns.len())
}

/// The same expression evaluator works after projecting onto a temporary row.
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

/// Row-local semantics retains exact null and error behavior, including
/// a nullable filter predicate: only Some(true) keeps the row.
pub fn eval_row(plan: &Plan, row: &Vec<Option<Value>>)
    -> (result: Result<Option<Vec<Option<Value>>>, ()>)
    ensures
        match result {
            Err(_) => row_spec(plan, row@).is_Err(),
            Ok(None) => row_spec(plan, row@) == Ok(None),
            Ok(Some(out)) => row_spec(plan, row@) == Ok(Some(out@)),
        },
    decreases *plan,
{
    match plan {
        Plan::Input => Ok(Some(row.clone())),
        Plan::Project { input, exprs } => {
            match eval_row(input, row) {
                Err(_) => Err(()),
                Ok(None) => Ok(None),
                Ok(Some(previous)) =>
                    match eval_project_prefix(exprs, &previous, exprs.len()) {
                        Ok(output) => Ok(Some(output)),
                        Err(_) => Err(()),
                    },
            }
        },
        Plan::Filter { input, predicate } => {
            match eval_row(input, row) {
                Err(_) => Err(()),
                Ok(None) => Ok(None),
                Ok(Some(previous)) =>
                    match eval_expr_row(predicate, &previous) {
                        Ok(Some(Value::Bool(true))) => Ok(Some(previous)),
                        Ok(Some(Value::Bool(false))) | Ok(None) => Ok(None),
                        _ => Err(()),
                    },
            }
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

/// Whole-dataset semantics evaluated from typed columnar input.
/// The row output is a temporary oracle representation, not an Arrow format.
pub fn eval_dataset_prefix(
    plan: &Plan, input: &Table, count: usize,
) -> (output: Result<Vec<Vec<Option<Value>>>, ()>)
    requires table_valid(input), count <= input.nrows,
    ensures
        match output {
            Ok(rows) => dataset_prefix_spec(plan, table_rows(input), count as nat)
                == Ok(rows_view(rows@).to_multiset()),
            Err(_) => dataset_prefix_spec(plan, table_rows(input), count as nat).is_Err(),
        },
    decreases count,
{
    if count == 0 {
        let empty: Vec<Vec<Option<Value>>> = Vec::new();
        proof {
            let empty_rows = rows_view(empty@);
            empty_rows.to_multiset_ensures();
            vstd::multiset::lemma_multiset_empty_len(empty_rows.to_multiset());
            assert(empty_rows.to_multiset() =~= Multiset::<Seq<Option<Value>>>::empty());
        }
        Ok(empty)
    } else {
        match eval_dataset_prefix(plan, input, count - 1) {
            Err(_) => Err(()),
            Ok(mut previous) => {
                let row = read_row(input, count - 1);
                proof {
                    assert(table_rows(input)[(count - 1) as int] ==
                        row_at(input, (count - 1) as int));
                }
                match eval_row(plan, &row) {
                    Err(_) => Err(()),
                    Ok(None) => Ok(previous),
                    Ok(Some(result)) => {
                        let ghost old_rows = previous@;
                        proof {
                            lemma_rows_view_push(old_rows, result);
                            rows_view(old_rows).to_multiset_ensures();
                            assert(rows_view(old_rows.push(result)).to_multiset() =~=
                                rows_view(old_rows).to_multiset().insert(result@));
                        }
                        previous.push(result);
                        Ok(previous)
                    },
                }
            },
        }
    }
}

pub fn evaluate(plan: &Plan, input: &Table)
    -> (output: Result<Vec<Vec<Option<Value>>>, ()>)
    requires table_valid(input),
    ensures
        match output {
            Ok(rows) => dataset_spec(plan, table_rows(input))
                == Ok(rows_view(rows@).to_multiset()),
            Err(_) => dataset_spec(plan, table_rows(input)).is_Err(),
        },
{
    eval_dataset_prefix(plan, input, input.nrows)
}

pub open spec fn input_domain(plan: &Plan, data: Seq<Seq<Option<Value>>>) -> bool {
    forall|i: int| 0 <= i < data.len() ==> row_spec(plan, data[i]).is_Ok()
}

/// Query semantics are bags. Ordering of realized rows is not observable.
pub open spec fn dataset_prefix_spec(
    plan: &Plan, data: Seq<Seq<Option<Value>>>, count: nat,
) -> Result<Multiset<Seq<Option<Value>>>, ()>
    recommends count <= data.len(),
    decreases count,
{
    if count == 0 { Ok(Multiset::empty()) }
    else {
        match dataset_prefix_spec(plan, data, (count - 1) as nat) {
            Err(_) => Err(()),
            Ok(prefix) => match row_spec(plan, data[(count - 1) as int]) {
                Err(_) => Err(()),
                Ok(None) => Ok(prefix),
                Ok(Some(row)) => Ok(prefix.insert(row)),
            },
        }
    }
}

pub open spec fn dataset_spec(plan: &Plan, data: Seq<Seq<Option<Value>>>)
    -> Result<Multiset<Seq<Option<Value>>>, ()>
{
    dataset_prefix_spec(plan, data, data.len())
}

/// Conformance criterion for an Arrow-backed checker. Check physical
/// dtypes and every nullable value, not equality of Arrow nullable flags:
/// a backend may declare a column nullable even if no rows contain null.
pub open spec fn output_conforms(
    plan: &Plan, input: &Table, actual: &Table,
) -> bool {
    table_valid(input)
        && table_valid(actual)
        && match plan_schema(plan, input.schema@) {
            Some(expected_fields) =>
                expected_fields.len() == actual.schema@.len()
                    && (forall|i: int| 0 <= i < expected_fields.len() ==>
                        expected_fields[i].dtype == actual.schema@[i].dtype)
                    && match dataset_spec(plan, table_rows(input)) {
                        Ok(expected) =>
                            expected =~= table_rows(actual).to_multiset(),
                        Err(_) => false,
                    },
            None => false,
        }
}

pub open spec fn bag_one_edit(
    left: Multiset<Seq<Option<Value>>>, right: Multiset<Seq<Option<Value>>>,
) -> bool {
    left =~= right
        || exists|row: Seq<Option<Value>>| right =~= left.insert(row)
        || exists|row: Seq<Option<Value>>| left =~= right.insert(row)
}

pub proof fn lemma_prefix_ignores_last(
    plan: &Plan, data: Seq<Seq<Option<Value>>>, count: nat,
)
    requires count < data.len(),
    ensures
        dataset_prefix_spec(plan, data, count)
            == dataset_prefix_spec(plan, data.drop_last(), count),
    decreases count,
{
    if count > 0 {
        lemma_prefix_ignores_last(plan, data, (count - 1) as nat);
        assert(data.drop_last()[(count - 1) as int] == data[(count - 1) as int]);
    }
}

pub proof fn lemma_dataset_total(plan: &Plan, data: Seq<Seq<Option<Value>>>)
    requires input_domain(plan, data),
    ensures dataset_spec(plan, data).is_Ok(),
    decreases data.len(),
{
    if data.len() > 0 {
        let prefix = data.drop_last();
        assert(input_domain(plan, prefix)) by {
            assert forall|i: int| 0 <= i < prefix.len()
                implies row_spec(plan, prefix[i]).is_Ok() by {
                assert(prefix[i] == data[i]);
            }
        }
        lemma_dataset_total(plan, prefix);
        lemma_prefix_ignores_last(plan, data, prefix.len());
        assert(dataset_prefix_spec(plan, data, prefix.len()) == dataset_spec(plan, prefix));
        assert(row_spec(plan, data.last()).is_Ok());
    }
}

/// Row-local plans change the complete result bag by at most one output row.
pub proof fn lemma_dataset_insert_stable(
    plan: &Plan, data: Seq<Seq<Option<Value>>>, index: int, row: Seq<Option<Value>>,
)
    requires
        0 <= index <= data.len(),
        input_domain(plan, data),
        input_domain(plan, data.insert(index, row)),
    ensures
        match (
            dataset_spec(plan, data),
            dataset_spec(plan, data.insert(index, row)),
        ) {
            (Ok(before), Ok(after)) =>
                after =~= match row_spec(plan, row) {
                    Ok(Some(added)) => before.insert(added),
                    _ => before,
                },
            _ => false,
        },
    decreases data.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    lemma_dataset_total(plan, data);
    lemma_dataset_total(plan, data.insert(index, row));
    if index == data.len() {
        let appended = data.push(row);
        assert(data.insert(index, row) =~= appended);
        assert(appended.drop_last() =~= data);
        lemma_prefix_ignores_last(plan, appended, data.len());
        assert(dataset_prefix_spec(plan, appended, data.len()) == dataset_spec(plan, data));
        assert(row_spec(plan, row).is_Ok());
    } else {
        let last = data.last();
        let prefix = data.drop_last();
        let inserted = data.insert(index, row);
        assert(index <= prefix.len());
        assert(inserted.last() == last);
        assert(inserted.drop_last() =~= prefix.insert(index, row));
        assert(input_domain(plan, prefix)) by {
            assert forall|i: int| 0 <= i < prefix.len()
                implies row_spec(plan, prefix[i]).is_Ok() by {
                assert(prefix[i] == data[i]);
            }
        }
        assert(input_domain(plan, prefix.insert(index, row))) by {
            assert forall|i: int| 0 <= i < prefix.insert(index, row).len()
                implies row_spec(plan, prefix.insert(index, row)[i]).is_Ok() by {
                assert(prefix.insert(index, row)[i] == inserted[i]);
            }
        }
        lemma_dataset_insert_stable(plan, prefix, index, row);
        lemma_prefix_ignores_last(plan, data, prefix.len());
        lemma_prefix_ignores_last(plan, inserted, prefix.insert(index, row).len());
        assert(dataset_prefix_spec(plan, data, prefix.len()) == dataset_spec(plan, prefix));
        assert(dataset_prefix_spec(plan, inserted, prefix.insert(index, row).len())
            == dataset_spec(plan, prefix.insert(index, row)));
        let before = dataset_spec(plan, prefix).unwrap();
        let after = dataset_spec(plan, prefix.insert(index, row)).unwrap();
        assert(bag_one_edit(before, after));
        match (row_spec(plan, row), row_spec(plan, last)) {
            (Ok(Some(added)), Ok(Some(tail))) => {
                assert(before.insert(added).insert(tail) =~=
                    before.insert(tail).insert(added));
                assert(after =~= before.insert(added));
            },
            _ => {},
        }
    }
}

/// One physical record insertion has a common index in every column.
pub open spec fn column_one_insert(
    before: &Column, after: &Column, index: int,
) -> bool {
    column_dtype(before) == column_dtype(after)
        && column_len(after) == column_len(before) + 1
        && forall|i: int| 0 <= i < column_len(after) ==>
            (i < index ==> column_cell(after, i) == column_cell(before, i))
            && (i > index ==> column_cell(after, i) == column_cell(before, i - 1))
}

pub open spec fn table_one_insert(
    before: &Table, after: &Table, index: int,
) -> bool {
    0 <= index <= before.nrows
        && after.nrows as nat == before.nrows as nat + 1
        && before.schema@ =~= after.schema@
        && before.columns@.len() == after.columns@.len()
        && forall|j: int| 0 <= j < before.columns@.len() ==>
            column_one_insert(
                &before.columns@[j], &after.columns@[j], index)
}

/// Physical column insertion implies insertion in the logical row sequence.
pub proof fn lemma_columnar_insert_rows(
    before: &Table, after: &Table, index: int,
)
    requires
        table_valid(before),
        table_valid(after),
        table_one_insert(before, after, index),
    ensures table_rows(after) =~=
        table_rows(before).insert(index, row_at(after, index)),
{
    assert(table_rows(after).len() == table_rows(before).len() + 1);
    assert forall|i: int| 0 <= i < table_rows(after).len()
        implies table_rows(after)[i] ==
            table_rows(before).insert(index, row_at(after, index))[i] by {
        if i == index {
            assert(table_rows(after)[i] == row_at(after, index));
        } else {
            let old_index = if i < index { i } else { i - 1 };
            assert(0 <= old_index < before.nrows);
            assert forall|j: int| 0 <= j < after.columns@.len()
                implies row_at(after, i)[j] == row_at(before, old_index)[j] by {
                assert(column_one_insert(
                    &before.columns@[j], &after.columns@[j], index));
                assert(row_at(after, i)[j] ==
                    column_cell(&after.columns@[j], i));
                assert(row_at(before, old_index)[j] ==
                    column_cell(&before.columns@[j], old_index));
            }
            assert(row_at(after, i) =~= row_at(before, old_index));
        }
    }
}

/// No row-domain preconditions: validity and static typing suffice.
pub proof fn lemma_typed_columnar_insert_stable(
    plan: &Plan, before: &Table, after: &Table, index: int,
)
    requires
        table_valid(before),
        table_valid(after),
        table_one_insert(before, after, index),
        plan_schema(plan, before.schema@).is_Some(),
    ensures
        match (
            dataset_spec(plan, table_rows(before)),
            dataset_spec(plan, table_rows(after)),
        ) {
            (Ok(before_bag), Ok(after_bag)) =>
                bag_one_edit(before_bag, after_bag),
            _ => false,
        },
{
    lemma_schema_establishes_domain(plan, before);
    assert(plan_schema(plan, after.schema@).is_Some());
    lemma_schema_establishes_domain(plan, after);
    lemma_columnar_insert_rows(before, after, index);
    lemma_dataset_insert_stable(
        plan, table_rows(before), index, row_at(after, index));
    match (
        dataset_spec(plan, table_rows(before)),
        dataset_spec(plan, table_rows(after)),
    ) {
        (Ok(before_bag), Ok(after_bag)) => {
            match row_spec(plan, row_at(after, index)) {
                Ok(Some(added)) => {
                    assert(after_bag =~= before_bag.insert(added));
                    assert(bag_one_edit(before_bag, after_bag));
                },
                _ => {
                    assert(after_bag =~= before_bag);
                    assert(bag_one_edit(before_bag, after_bag));
                },
            }
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
            columns: vec![
                Column::Int64(vec![Some(-2), Some(2), None, Some(4), Some(6)]),
                Column::Boolean(vec![Some(true), Some(true), Some(true), None, Some(false)]),
            ],
            nrows: 5,
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
            columns: vec![Column::Int64(vec![])],
            nrows: 0,
        };
        assert!(evaluate(&Plan::Input, &empty) == Ok(vec![]));

        let no_columns = Table {
            schema: vec![],
            columns: vec![],
            nrows: 2,
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
            columns: vec![
                Column::Int64(vec![Some(5), None, Some(1)]),
                Column::Boolean(vec![None, Some(false), Some(true)]),
            ],
            nrows: 3,
        };
        let less_than = Expr::Binary {
            left: Box::new(Expr::Column(0)), op: BinaryOp::Lt,
            right: Box::new(Expr::Literal(Value::Int(3))),
        };
        assert!(eval_expr_row(&less_than, &read_row(&t, 0)) == Ok(Some(Value::Bool(false))));
        assert!(eval_expr_row(&less_than, &read_row(&t, 1)) == Ok(None));
        assert!(eval_expr_row(&less_than, &read_row(&t, 2)) == Ok(Some(Value::Bool(true))));

        let and_false = Expr::Binary {
            left: Box::new(Expr::Column(1)), op: BinaryOp::And,
            right: Box::new(Expr::Literal(Value::Bool(false))),
        };
        assert!(eval_expr_row(&and_false, &read_row(&t, 0)) == Ok(Some(Value::Bool(false))));
        assert!(eval_expr_row(&and_false, &read_row(&t, 1)) == Ok(Some(Value::Bool(false))));

        let and_true = Expr::Binary {
            left: Box::new(Expr::Column(1)), op: BinaryOp::And,
            right: Box::new(Expr::Literal(Value::Bool(true))),
        };
        assert!(eval_expr_row(&and_true, &read_row(&t, 0)) == Ok(None));
        assert!(eval_expr_row(&and_true, &read_row(&t, 2)) == Ok(Some(Value::Bool(true))));

        let clipped = Expr::Clip {
            expr: Box::new(Expr::Column(0)),
            lower: Some(Box::new(Expr::Literal(Value::Int(2)))),
            upper: Some(Box::new(Expr::Literal(Value::Int(4)))),
        };
        assert!(eval_expr_row(&clipped, &read_row(&t, 0)) == Ok(Some(Value::Int(4))));
        assert!(eval_expr_row(&clipped, &read_row(&t, 1)) == Ok(None));
        assert!(eval_expr_row(&clipped, &read_row(&t, 2)) == Ok(Some(Value::Int(2))));
    }

    #[test]
    fn clip_expression_bounds_and_null_bounds() {
        let input = Table {
            schema: vec![
                Field { dtype: DType::Int64, nullable: true },
                Field { dtype: DType::Int64, nullable: true },
                Field { dtype: DType::Int64, nullable: true },
            ],
            columns: vec![
                Column::Int64(vec![
                    Some(0), Some(5), Some(8), None, Some(7), Some(5),
                ]),
                Column::Int64(vec![
                    Some(2), None, Some(9), Some(0), Some(8), None,
                ]),
                Column::Int64(vec![
                    Some(10), Some(3), Some(6), Some(3), Some(3), None,
                ]),
            ],
            nrows: 6,
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
        assert!(eval_expr_row(&upper_only, &read_row(&input, 1))
            == Ok(Some(Value::Int(3))));
        assert!(eval_expr_row(&upper_only, &read_row(&input, 5))
            == Ok(Some(Value::Int(5))));

        let no_bounds = Expr::Clip {
            expr: Box::new(Expr::Column(0)),
            lower: None,
            upper: None,
        };
        assert!(eval_expr_row(&no_bounds, &read_row(&input, 4))
            == Ok(Some(Value::Int(7))));
    }

    #[test]
    fn nullable_column_cell() {
        let col = Column::Int64(vec![Some(4), None, Some(-1)]);
        assert!(read_cell(&col, 0) == Some(Value::Int(4)));
        assert!(read_cell(&col, 1) == None);
        assert!(read_cell(&col, 2) == Some(Value::Int(-1)));
    }
}

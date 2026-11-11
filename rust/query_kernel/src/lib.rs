use vstd::prelude::*;
use vstd::multiset::Multiset;

verus! {

// This is syntax, not a representation of realized tables. Column references
// are resolved to positions by the frontend; physical schemas belong to adapters.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

impl DeepView for Value {
    type V = Value;

    open spec fn deep_view(&self) -> Value {
        *self
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Lt,
    Gt,
    Eq,
    And,
}

/// A single recursive expression language for values and predicates.
pub enum Expr {
    Column(usize),
    Literal(Value),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Clip {
        expr: Box<Expr>,
        lower: i64,
        upper: i64,
    },
}

/// Extended bag/sequence relational algebra, initially restricted to
/// cardinality-nonincreasing row-wise operators. WithColumns and Select
/// are both projections: WithColumns includes pass-through column expressions.
pub enum Plan {
    Input,
    Project {
        input: Box<Plan>,
        exprs: Vec<Expr>,
    },
    Filter {
        input: Box<Plan>,
        predicate: Expr,
    },
}

// Realized rows appear *only in the semantics*, as abstract sequences
// of values. There is no fixed Row or Column type in the AST.

pub open spec fn binary_spec(op: BinaryOp, lhs: Value, rhs: Value) -> Option<Value> {
    match op {
        BinaryOp::Lt => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a < b)),
            _ => None,
        },
        BinaryOp::Gt => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a > b)),
            _ => None,
        },
        BinaryOp::Eq => match (lhs, rhs) {
            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(a == b)),
            _ => Some(Value::Bool(false)),
        },
        BinaryOp::And => match (lhs, rhs) {
            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(a && b)),
            _ => None,
        },
    }
}

pub open spec fn expr_spec(expr: &Expr, row: Seq<Value>) -> Option<Value>
    decreases *expr,
{
    match expr {
        Expr::Column(index) => {
            if *index < row.len() { Some(row[*index as int]) } else { None }
        },
        Expr::Literal(value) => Some(*value),
        Expr::Binary { left, op, right } => {
            match (expr_spec(left, row), expr_spec(right, row)) {
                (Some(lhs), Some(rhs)) => binary_spec(*op, lhs, rhs),
                _ => None,
            }
        },
        Expr::Clip { expr, lower, upper } => {
            if *lower > *upper {
                None
            } else {
                match expr_spec(expr, row) {
                    Some(Value::Int(v)) => {
                        Some(Value::Int(if v < *lower { *lower }
                            else if v > *upper { *upper } else { v }))
                    },
                    _ => None,
                }
            }
        },
    }
}

/// Types belong to the *analysis*, not to Expr or Plan.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DType {
    Int,
    Bool,
}

pub open spec fn value_dtype(value: Value) -> DType {
    match value {
        Value::Int(_) => DType::Int,
        Value::Bool(_) => DType::Bool,
    }
}

/// Domain membership of a realized row, independent of its logical AST.
pub open spec fn row_in_schema(row: Seq<Value>, schema: Seq<DType>) -> bool {
    row.len() == schema.len()
        && forall|i: int| 0 <= i < row.len()
            ==> value_dtype(row[i]) == schema[i]
}

/// Static type analysis of a recursive expression.
pub open spec fn expr_dtype(expr: &Expr, schema: Seq<DType>) -> Option<DType>
    decreases *expr,
{
    match expr {
        Expr::Column(index) => {
            if *index < schema.len() { Some(schema[*index as int]) } else { None }
        },
        Expr::Literal(value) => Some(value_dtype(*value)),
        Expr::Binary { left, op, right } => {
            let left_dtype = expr_dtype(left, schema);
            let right_dtype = expr_dtype(right, schema);
            match op {
                BinaryOp::Lt | BinaryOp::Gt => {
                    if left_dtype == Some(DType::Int) && right_dtype == Some(DType::Int) {
                        Some(DType::Bool)
                    } else { None }
                },
                BinaryOp::Eq => {
                    if left_dtype.is_Some() && left_dtype == right_dtype {
                        Some(DType::Bool)
                    } else { None }
                },
                BinaryOp::And => {
                    if left_dtype == Some(DType::Bool) && right_dtype == Some(DType::Bool) {
                        Some(DType::Bool)
                    } else { None }
                },
            }
        },
        Expr::Clip { expr, lower, upper } => {
            if *lower <= *upper && expr_dtype(expr, schema) == Some(DType::Int) {
                Some(DType::Int)
            } else { None }
        },
    }
}

pub open spec fn project_prefix_schema(
    exprs: Seq<Expr>, schema: Seq<DType>, count: nat,
) -> Option<Seq<DType>>
    recommends count <= exprs.len(),
    decreases count,
{
    if count == 0 {
        Some(Seq::empty())
    } else {
        match (
            project_prefix_schema(exprs, schema, (count - 1) as nat),
            expr_dtype(&exprs[(count - 1) as int], schema),
        ) {
            (Some(prefix), Some(dtype)) => Some(prefix.push(dtype)),
            _ => None,
        }
    }
}

pub open spec fn project_schema(exprs: Seq<Expr>, schema: Seq<DType>)
    -> Option<Seq<DType>>
{
    project_prefix_schema(exprs, schema, exprs.len())
}

/// Each logical node derives its output schema from its child.
pub open spec fn plan_schema(plan: &Plan, input_schema: Seq<DType>)
    -> Option<Seq<DType>>
    decreases *plan,
{
    match plan {
        Plan::Input => Some(input_schema),
        Plan::Project { input, exprs } => {
            match plan_schema(input, input_schema) {
                Some(child_schema) => project_schema(exprs@, child_schema),
                None => None,
            }
        },
        Plan::Filter { input, predicate } => {
            match plan_schema(input, input_schema) {
                Some(child_schema) => {
                    if expr_dtype(predicate, child_schema) == Some(DType::Bool) {
                        Some(child_schema)
                    } else { None }
                },
                None => None,
            }
        },
    }
}

/// An extensional domain: evaluation must not fail on any input row.
/// Schema checking can establish this property for well-typed input data.
pub open spec fn input_domain(plan: &Plan, data: Seq<Seq<Value>>) -> bool {
    forall|i: int| 0 <= i < data.len()
        ==> row_spec(plan, data[i]).is_Ok()
}

/// Project expressions read the same input row. The indexed recursion
/// aligns directly with the executable reference evaluator.
pub open spec fn project_prefix_spec(
    exprs: Seq<Expr>, row: Seq<Value>, count: nat,
) -> Option<Seq<Value>>
    recommends count <= exprs.len(),
    decreases count,
{
    if count == 0 {
        Some(Seq::empty())
    } else {
        match (
            project_prefix_spec(exprs, row, (count - 1) as nat),
            expr_spec(&exprs[(count - 1) as int], row),
        ) {
            (Some(prefix), Some(value)) => Some(prefix.push(value)),
            _ => None,
        }
    }
}

pub open spec fn project_spec(exprs: Seq<Expr>, row: Seq<Value>) -> Option<Seq<Value>> {
    project_prefix_spec(exprs, row, exprs.len())
}

/// Specification distinguishes filtering (Ok(None)) from invalid expressions
/// (Err). This is essential before a backend conformance checker can use the
/// specification as an oracle.
pub open spec fn row_spec(plan: &Plan, row: Seq<Value>) -> Result<Option<Seq<Value>>, ()>
    decreases *plan,
{
    match plan {
        Plan::Input => Ok(Some(row)),
        Plan::Project { input, exprs } => {
            match row_spec(input, row) {
                Err(_) => Err(()),
                Ok(None) => Ok(None),
                Ok(Some(previous)) => match project_spec(exprs@, previous) {
                    Some(values) => Ok(Some(values)),
                    None => Err(()),
                },
            }
        },
        Plan::Filter { input, predicate } => {
            match row_spec(input, row) {
                Err(_) => Err(()),
                Ok(None) => Ok(None),
                Ok(Some(previous)) => match expr_spec(predicate, previous) {
                    Some(Value::Bool(true)) => Ok(Some(previous)),
                    Some(Value::Bool(false)) => Ok(None),
                    _ => Err(()),
                },
            }
        },
    }
}

/// Complete, finite-relation interpretation of the *same* recursive
/// Expr/Plan AST. Evaluation is strict: a bad expression is an error rather
/// than a dropped record. Physical order is not assumed to be stable.
pub open spec fn dataset_prefix_spec(
    plan: &Plan,
    data: Seq<Seq<Value>>,
    count: nat,
) -> Result<Seq<Seq<Value>>, ()>
    recommends count <= data.len(),
    decreases count,
{
    if count == 0 {
        Ok(Seq::empty())
    } else {
        match dataset_prefix_spec(plan, data, (count - 1) as nat) {
            Err(_) => Err(()),
            Ok(prefix) => match row_spec(plan, data[(count - 1) as int]) {
                Err(_) => Err(()),
                Ok(None) => Ok(prefix),
                Ok(Some(row)) => Ok(prefix.push(row)),
            },
        }
    }
}

pub open spec fn dataset_spec(plan: &Plan, data: Seq<Seq<Value>>)
    -> Result<Seq<Seq<Value>>, ()>
{
    dataset_prefix_spec(plan, data, data.len())
}

/// The bag interpretation used for symmetric-distance analysis, defined
/// only for inputs in input_domain. Invalid rows are ignored by this ghost
/// helper, but no theorem permits using it outside input_domain.
pub open spec fn output_bag(plan: &Plan, data: Seq<Seq<Value>>)
    -> Multiset<Seq<Value>>
    decreases data.len(),
{
    if data.len() == 0 {
        Multiset::empty()
    } else {
        let prior = output_bag(plan, data.drop_last());
        match row_spec(plan, data.last()) {
            Ok(Some(row)) => prior.insert(row),
            _ => prior,
        }
    }
}

pub open spec fn bag_one_edit(left: Multiset<Seq<Value>>, right: Multiset<Seq<Value>>) -> bool {
    left =~= right
        || exists|row: Seq<Value>| right =~= left.insert(row)
        || exists|row: Seq<Value>| left =~= right.insert(row)
}

/// Inserting one row changes the *entire output multiset* by at most one row.
/// Unlike a cardinality bound, this preserves output row identities.
pub proof fn lemma_bag_append(plan: &Plan, data: Seq<Seq<Value>>, row: Seq<Value>)
    ensures
        output_bag(plan, data.push(row))
            == match row_spec(plan, row) {
                Ok(Some(out)) => output_bag(plan, data).insert(out),
                _ => output_bag(plan, data),
            },
        bag_one_edit(output_bag(plan, data), output_bag(plan, data.push(row))),
{
    assert(data.push(row).drop_last() =~= data);
    assert(data.push(row).last() == row);
    match row_spec(plan, row) {
        Ok(Some(out)) => {
            assert(output_bag(plan, data.push(row)) == output_bag(plan, data).insert(out));
        },
        _ => {},
    }
}

/// Extension from appending a row to inserting one at *any* position.
/// Multisets commute, so the inserted row may occur anywhere in the input
/// sequence without affecting the stability bound.
pub proof fn lemma_bag_insert(
    plan: &Plan, data: Seq<Seq<Value>>, index: int, row: Seq<Value>,
)
    requires 0 <= index <= data.len(),
    ensures
        output_bag(plan, data.insert(index, row)) =~=
            match row_spec(plan, row) {
                Ok(Some(out)) => output_bag(plan, data).insert(out),
                _ => output_bag(plan, data),
            },
        bag_one_edit(output_bag(plan, data), output_bag(plan, data.insert(index, row))),
    decreases data.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if index == data.len() {
        lemma_bag_append(plan, data, row);
        assert(data.insert(index, row) =~= data.push(row));
    } else {
        let last = data.last();
        let prefix = data.drop_last();
        assert(index <= prefix.len());
        lemma_bag_insert(plan, prefix, index, row);
        assert(data.insert(index, row).last() == last);
        assert(data.insert(index, row).drop_last() =~= prefix.insert(index, row));
        match (row_spec(plan, row), row_spec(plan, last)) {
            (Ok(Some(added)), Ok(Some(tail))) => {
                let old = output_bag(plan, prefix);
                assert(old.insert(added).insert(tail) =~=
                    old.insert(tail).insert(added));
            },
            _ => {},
        }
    }
}

/// Evaluating the first `count` rows ignores later input rows.
pub proof fn lemma_prefix_ignores_last(
    plan: &Plan, data: Seq<Seq<Value>>, count: nat,
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

/// The multiset used in the stability proof is exactly the bag of rows
/// produced by whole-dataset execution, on the statically validated domain.
pub proof fn lemma_dataset_bag(plan: &Plan, data: Seq<Seq<Value>>)
    requires input_domain(plan, data),
    ensures
        match dataset_spec(plan, data) {
            Ok(rows) => output_bag(plan, data) =~= rows.to_multiset(),
            Err(_) => false,
        },
    decreases data.len(),
{
    if data.len() == 0 {
        let empty = Seq::<Seq<Value>>::empty();
        empty.to_multiset_ensures();
        vstd::multiset::lemma_multiset_empty_len(empty.to_multiset());
        assert(dataset_spec(plan, data) == Ok(Seq::<Seq<Value>>::empty()));
        assert(output_bag(plan, data) =~= Seq::<Seq<Value>>::empty().to_multiset());
    } else {
        let prefix = data.drop_last();
        assert(input_domain(plan, prefix)) by {
            assert forall|i: int| 0 <= i < prefix.len()
                implies row_spec(plan, prefix[i]).is_Ok() by {
                assert(prefix[i] == data[i]);
            }
        }
        lemma_dataset_bag(plan, prefix);
        lemma_prefix_ignores_last(plan, data, prefix.len());
        assert(dataset_prefix_spec(plan, data, prefix.len()) == dataset_spec(plan, prefix));
        assert(data.drop_last() =~= prefix);
        match dataset_spec(plan, prefix) {
            Ok(previous) => {
                assert(output_bag(plan, prefix) =~= previous.to_multiset());
                assert(row_spec(plan, data.last()).is_Ok());
                match row_spec(plan, data.last()) {
                    Ok(Some(row)) => {
                        previous.to_multiset_ensures();
                        assert(previous.push(row).to_multiset() =~=
                            previous.to_multiset().insert(row));
                        assert(dataset_spec(plan, data) == Ok(previous.push(row)));
                        assert(output_bag(plan, data) =~= previous.push(row).to_multiset());
                    },
                    Ok(None) => {
                        assert(dataset_spec(plan, data) == Ok(previous));
                        assert(output_bag(plan, data) =~= previous.to_multiset());
                    },
                    Err(_) => { assert(false); },
                }
            },
            Err(_) => { assert(false); },
        }
    }
}

/// Main stability theorem: one inserted record in a valid input changes
/// the *complete verified executor's output bag* by at most one record.
/// No metric or backend selection is encoded in the query AST.
pub proof fn lemma_dataset_insert_stable(
    plan: &Plan, data: Seq<Seq<Value>>, index: int, row: Seq<Value>,
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
            (Ok(before), Ok(after)) => bag_one_edit(
                before.to_multiset(), after.to_multiset(),
            ),
            _ => false,
        },
{
    lemma_dataset_bag(plan, data);
    lemma_dataset_bag(plan, data.insert(index, row));
    lemma_bag_insert(plan, data, index, row);
    match (
        dataset_spec(plan, data),
        dataset_spec(plan, data.insert(index, row)),
    ) {
        (Ok(before), Ok(after)) => {
            assert(output_bag(plan, data) =~= before.to_multiset());
            assert(output_bag(plan, data.insert(index, row)) =~= after.to_multiset());
            assert(bag_one_edit(output_bag(plan, data), output_bag(plan, data.insert(index, row))));
        },
        _ => { assert(false); },
    }
}

/// A well-typed expression evaluates to a value of its inferred type.
pub proof fn lemma_expr_type_sound(
    expr: &Expr, schema: Seq<DType>, row: Seq<Value>, dtype: DType,
)
    requires
        row_in_schema(row, schema),
        expr_dtype(expr, schema) == Some(dtype),
    ensures
        match expr_spec(expr, row) {
            Some(value) => value_dtype(value) == dtype,
            None => false,
        },
    decreases *expr,
{
    match expr {
        Expr::Column(index) => {
            assert(*index < schema.len());
            assert(0 <= (*index as int) && (*index as int) < row.len());
            assert(value_dtype(row[*index as int]) == schema[*index as int]);
        },
        Expr::Literal(_) => {},
        Expr::Binary { left, op: _, right } => {
            let left_dtype = expr_dtype(left, schema);
            let right_dtype = expr_dtype(right, schema);
            if let Some(left_ty) = left_dtype {
                lemma_expr_type_sound(left, schema, row, left_ty);
            }
            if let Some(right_ty) = right_dtype {
                lemma_expr_type_sound(right, schema, row, right_ty);
            }
        },
        Expr::Clip { expr: child, .. } => {
            lemma_expr_type_sound(child, schema, row, DType::Int);
        },
    }
}

/// Project typing and semantics recurse over exactly the same prefix.
pub proof fn lemma_project_prefix_type_sound(
    exprs: Seq<Expr>, schema: Seq<DType>, row: Seq<Value>, count: nat,
)
    requires
        row_in_schema(row, schema),
        count <= exprs.len(),
        project_prefix_schema(exprs, schema, count).is_Some(),
    ensures
        match (
            project_prefix_schema(exprs, schema, count),
            project_prefix_spec(exprs, row, count),
        ) {
            (Some(types), Some(values)) => row_in_schema(values, types),
            _ => false,
        },
    decreases count,
{
    if count > 0 {
        lemma_project_prefix_type_sound(exprs, schema, row, (count - 1) as nat);
        let dtype = expr_dtype(&exprs[(count - 1) as int], schema).unwrap();
        lemma_expr_type_sound(&exprs[(count - 1) as int], schema, row, dtype);

        let previous_schema =
            project_prefix_schema(exprs, schema, (count - 1) as nat).unwrap();
        let previous_values =
            project_prefix_spec(exprs, row, (count - 1) as nat).unwrap();
        let value = expr_spec(&exprs[(count - 1) as int], row).unwrap();
        assert(row_in_schema(previous_values, previous_schema));
        assert(value_dtype(value) == dtype);
        assert(row_in_schema(previous_values.push(value), previous_schema.push(dtype))) by {
            assert(previous_values.len() == previous_schema.len());
            assert forall|i: int| 0 <= i < previous_values.push(value).len()
                implies value_dtype(previous_values.push(value)[i])
                    == previous_schema.push(dtype)[i] by {
                if i < previous_values.len() {
                    assert(value_dtype(previous_values[i]) == previous_schema[i]);
                } else {
                    assert(i == previous_values.len());
                }
            }
        }
    }
}

pub proof fn lemma_project_type_sound(
    exprs: Seq<Expr>, schema: Seq<DType>, row: Seq<Value>,
)
    requires
        row_in_schema(row, schema),
        project_schema(exprs, schema).is_Some(),
    ensures
        match (project_schema(exprs, schema), project_spec(exprs, row)) {
            (Some(types), Some(values)) => row_in_schema(values, types),
            _ => false,
        },
{
    lemma_project_prefix_type_sound(exprs, schema, row, exprs.len());
}

/// This proof turns the syntactic checker into a semantic totality guarantee.
pub proof fn lemma_plan_type_sound(
    plan: &Plan, schema: Seq<DType>, row: Seq<Value>, output_schema: Seq<DType>,
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
                lemma_project_type_sound(exprs@, child_schema, previous);
            }
        },
        Plan::Filter { input, predicate } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_plan_type_sound(input, schema, row, child_schema);
            if let Ok(Some(previous)) = row_spec(input, row) {
                lemma_expr_type_sound(predicate, child_schema, previous, DType::Bool);
            }
        },
    }
}

pub proof fn lemma_schema_establishes_domain(
    plan: &Plan, schema: Seq<DType>, data: Seq<Seq<Value>>,
)
    requires
        plan_schema(plan, schema).is_Some(),
        forall|i: int| 0 <= i < data.len() ==> row_in_schema(data[i], schema),
    ensures
        input_domain(plan, data),
{
    let output_schema = plan_schema(plan, schema).unwrap();
    assert forall|i: int| 0 <= i < data.len()
        implies row_spec(plan, data[i]).is_Ok() by {
            lemma_plan_type_sound(plan, schema, data[i], output_schema);
        }
}

/// Executable scalar evaluator (with a verified specification).
pub fn eval_expr(expr: &Expr, row: &Vec<Value>) -> (result: Option<Value>)
    ensures result == expr_spec(expr, row@),
    decreases *expr,
{
    match expr {
        Expr::Column(index) => {
            if *index < row.len() { Some(row[*index]) } else { None }
        },
        Expr::Literal(value) => Some(*value),
        Expr::Binary { left, op, right } => {
            match (eval_expr(left, row), eval_expr(right, row)) {
                (Some(lhs), Some(rhs)) => {
                    match op {
                        BinaryOp::Lt => match (lhs, rhs) {
                            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a < b)),
                            _ => None,
                        },
                        BinaryOp::Gt => match (lhs, rhs) {
                            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a > b)),
                            _ => None,
                        },
                        BinaryOp::Eq => match (lhs, rhs) {
                            (Value::Int(a), Value::Int(b)) => Some(Value::Bool(a == b)),
                            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(a == b)),
                            _ => Some(Value::Bool(false)),
                        },
                        BinaryOp::And => match (lhs, rhs) {
                            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(a && b)),
                            _ => None,
                        },
                    }
                },
                _ => None,
            }
        },
        Expr::Clip { expr, lower, upper } => {
            if lower > upper {
                None
            } else {
                match eval_expr(expr, row) {
                    Some(Value::Int(v)) => {
                        Some(Value::Int(if v < *lower { *lower }
                            else if v > *upper { *upper } else { v }))
                    },
                    _ => None,
                }
            }
        },
    }
}

/// The executable interpreter deliberately reports invalid expressions
/// instead of silently treating them as filtered-out rows.
/// Executable implementation of the project specification.
pub fn eval_project_prefix(
    exprs: &Vec<Expr>, row: &Vec<Value>, count: usize,
) -> (result: Option<Vec<Value>>)
    requires count <= exprs.len(),
    ensures
        match result {
            Some(values) =>
                project_prefix_spec(exprs@, row@, count as nat) == Some(values@),
            None =>
                project_prefix_spec(exprs@, row@, count as nat).is_None(),
        },
    decreases count,
{
    if count == 0 {
        Some(Vec::new())
    } else {
        match eval_project_prefix(exprs, row, count - 1) {
            None => None,
            Some(mut prefix) => {
                match eval_expr(&exprs[count - 1], row) {
                    None => None,
                    Some(value) => {
                        prefix.push(value);
                        Some(prefix)
                    },
                }
            },
        }
    }
}

/// Refines the *complete* row specification, including invalid expressions.
pub fn eval_row(plan: &Plan, row: &Vec<Value>) -> (result: Result<Option<Vec<Value>>, ()>)
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
                Ok(Some(previous)) => {
                    match eval_project_prefix(exprs, &previous, exprs.len()) {
                        Some(output) => Ok(Some(output)),
                        None => Err(()),
                    }
                },
            }
        },
        Plan::Filter { input, predicate } => {
            match eval_row(input, row) {
                Err(_) => Err(()),
                Ok(None) => Ok(None),
                Ok(Some(previous)) => {
                    match eval_expr(predicate, &previous) {
                        Some(Value::Bool(true)) => Ok(Some(previous)),
                        Some(Value::Bool(false)) => Ok(None),
                        _ => Err(()),
                    }
                },
            }
        },
    }
}

/// A thin semantic view of the executable nested vectors.
pub open spec fn rows_view(rows: Seq<Vec<Value>>) -> Seq<Seq<Value>> {
    Seq::new(rows.len(), |i: int| rows[i]@)
}

pub proof fn lemma_rows_view_push(rows: Seq<Vec<Value>>, row: Vec<Value>)
    ensures rows_view(rows.push(row)) =~= rows_view(rows).push(row@),
{
    assert(rows_view(rows.push(row)).len() == rows_view(rows).push(row@).len());
    assert forall|i: int| 0 <= i < rows_view(rows.push(row)).len()
        implies rows_view(rows.push(row))[i] == rows_view(rows).push(row@)[i] by {
        if i < rows.len() {
            assert(rows.push(row)[i] == rows[i]);
        } else {
            assert(i == rows.len());
        }
    }
}

/// A deliberately recursive reference evaluator: mirrors dataset_prefix_spec.
/// No loop invariant or special error-propagation proof is necessary.
pub fn eval_dataset_prefix(
    plan: &Plan, input: &Vec<Vec<Value>>, count: usize,
) -> (output: Result<Vec<Vec<Value>>, ()>)
    requires count <= input.len(),
    ensures
        match output {
            Ok(rows) =>
                dataset_prefix_spec(plan, rows_view(input@), count as nat)
                    == Ok(rows_view(rows@)),
            Err(_) =>
                dataset_prefix_spec(plan, rows_view(input@), count as nat).is_Err(),
        },
    decreases count,
{
    if count == 0 {
        let result: Vec<Vec<Value>> = Vec::new();
        proof { assert(rows_view(result@) =~= Seq::<Seq<Value>>::empty()); }
        Ok(result)
    } else {
        match eval_dataset_prefix(plan, input, count - 1) {
            Err(_) => Err(()),
            Ok(mut previous) => {
                proof {
                    assert(rows_view(input@)[(count - 1) as int] == input[(count - 1) as int]@);
                }
                match eval_row(plan, &input[count - 1]) {
                    Err(_) => Err(()),
                    Ok(None) => Ok(previous),
                    Ok(Some(row)) => {
                        let ghost old_rows = previous@;
                        proof { lemma_rows_view_push(old_rows, row); }
                        previous.push(row);
                        Ok(previous)
                    },
                }
            },
        }
    }
}

/// Exact executable reference implementation of whole-dataset semantics.
pub fn evaluate(plan: &Plan, input: &Vec<Vec<Value>>) -> (output: Result<Vec<Vec<Value>>, ()>)
    ensures
        match output {
            Ok(rows) => dataset_spec(plan, rows_view(input@)) == Ok(rows_view(rows@)),
            Err(_) => dataset_spec(plan, rows_view(input@)).is_Err(),
        },
{
    eval_dataset_prefix(plan, input, input.len())
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    fn col(index: usize) -> Expr { Expr::Column(index) }
    fn int(value: i64) -> Expr { Expr::Literal(Value::Int(value)) }
    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
    }

    #[test]
    fn nested_filters_of_projection() {
        // Equivalent to with_columns(z=clip(x, 0, 5))
        //     .filter(z > 1).filter(z < 4).
        let plan = Plan::Filter {
            input: Box::new(Plan::Filter {
                input: Box::new(Plan::Project {
                    input: Box::new(Plan::Input),
                    exprs: vec![
                        col(0),
                        Expr::Clip { expr: Box::new(col(0)), lower: 0, upper: 5 },
                    ],
                }),
                predicate: binary(col(1), BinaryOp::Gt, int(1)),
            }),
            predicate: binary(col(1), BinaryOp::Lt, int(4)),
        };
        let input = vec![-2, 2, 3, 4, 6]
            .into_iter()
            .map(|x| vec![Value::Int(x)])
            .collect();
        let output = evaluate(&plan, &input).unwrap();
        assert!(output == vec![
            vec![Value::Int(2), Value::Int(2)],
            vec![Value::Int(3), Value::Int(3)],
        ]);
    }

    #[test]
    fn invalid_predicate_is_not_silently_a_filter() {
        let plan = Plan::Filter {
            input: Box::new(Plan::Input),
            predicate: int(1),
        };
        let input = vec![vec![Value::Int(2)]];
        assert!(evaluate(&plan, &input).is_err());
    }

    #[test]
    fn empty_input_and_integer_boundaries() {
        let plan = Plan::Project {
            input: Box::new(Plan::Input),
            exprs: vec![Expr::Clip {
                expr: Box::new(col(0)), lower: 0, upper: 5,
            }],
        };
        assert!(evaluate(&plan, &vec![]).unwrap().is_empty());
        let input = vec![
            vec![Value::Int(i64::MIN)],
            vec![Value::Int(i64::MAX)],
        ];
        assert!(evaluate(&plan, &input).unwrap() == vec![
            vec![Value::Int(0)],
            vec![Value::Int(5)],
        ]);
    }
}

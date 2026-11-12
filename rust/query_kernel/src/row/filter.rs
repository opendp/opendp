use super::*;
verus! {

/// Type-check a filter against its input schema.
pub open spec fn schema(predicate: &Expr, input: Seq<Field>) -> Option<Seq<Field>> {
    match expr_field(predicate, input) {
        Some(field) if field.dtype == DType::Boolean => Some(input),
        _ => None,
    }
}

/// A filter returns its input row or drops it.
pub open spec fn row_spec(
    predicate: &Expr, row: Seq<Option<Value>>,
) -> Result<Option<Seq<Option<Value>>>, ()> {
    match expr_row_spec(predicate, row) {
        Ok(Some(Value::Bool(true))) => Ok(Some(row)),
        Ok(Some(Value::Bool(false))) | Ok(None) => Ok(None),
        _ => Err(()),
    }
}

pub proof fn lemma_type_sound(
    predicate: &Expr, input: Seq<Field>, row: Seq<Option<Value>>,
)
    requires
        row_in_schema(row, input),
        schema(predicate, input) == Some(input),
    ensures
        match row_spec(predicate, row) {
            Ok(Some(result)) => row_in_schema(result, input),
            Ok(None) => true,
            Err(_) => false,
        },
{
    let field = expr_field(predicate, input).unwrap();
    lemma_expr_field_sound(predicate, input, row, field);
}

pub fn eval_row(predicate: &Expr, row: &Vec<Option<Value>>)
    -> (result: Result<Option<Vec<Option<Value>>>, ()>)
    ensures
        match result {
            Err(_) => row_spec(predicate, row@).is_Err(),
            Ok(None) => row_spec(predicate, row@) == Ok(None),
            Ok(Some(out)) => row_spec(predicate, row@) == Ok(Some(out@)),
        },
{
    match eval_expr_row(predicate, row) {
        Ok(Some(Value::Bool(true))) => Ok(Some(row.clone())),
        Ok(Some(Value::Bool(false))) | Ok(None) => Ok(None),
        _ => Err(()),
    }
}

} // verus!

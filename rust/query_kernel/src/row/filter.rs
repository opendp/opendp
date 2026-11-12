use super::*;
verus! {

/// Type-check a filter against its input schema.
pub open spec fn schema(predicate: &VExpr, input: Seq<VField>) -> Option<Seq<VField>> {
    match expr_field(predicate, input) {
        Some(field) if field.dtype == VDType::Boolean => Some(input),
        _ => None,
    }
}

/// A filter returns its input row or drops it.
pub open spec fn row_spec(
    predicate: &VExpr, row: Seq<Option<VValue>>,
) -> Result<Option<Seq<Option<VValue>>>, ()> {
    match expr_row_spec(predicate, row) {
        Ok(Some(VValue::Bool(true))) => Ok(Some(row)),
        Ok(Some(VValue::Bool(false))) | Ok(None) => Ok(None),
        _ => Err(()),
    }
}

pub proof fn lemma_type_sound(
    predicate: &VExpr, input: Seq<VField>, row: Seq<Option<VValue>>,
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

pub fn eval_row(predicate: &VExpr, row: &Vec<Option<VValue>>)
    -> (result: Result<Option<Vec<Option<VValue>>>, ()>)
    ensures
        match result {
            Err(_) => row_spec(predicate, row@).is_Err(),
            Ok(None) => row_spec(predicate, row@) == Ok(None),
            Ok(Some(out)) => row_spec(predicate, row@) == Ok(Some(out@)),
        },
{
    match eval_expr_row(predicate, row) {
        Ok(Some(VValue::Bool(true))) => Ok(Some(row.clone())),
        Ok(Some(VValue::Bool(false))) | Ok(None) => Ok(None),
        _ => Err(()),
    }
}

} // verus!

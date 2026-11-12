use super::*;

verus! {

/// Find a column whose value is passed through unchanged by aliases.
pub open spec fn identity_column_spec(expr: &VExpr) -> Option<usize>
    decreases *expr,
{
    match expr {
        VExpr::Column(index) => Some(*index),
        VExpr::Alias { expr, .. } => identity_column_spec(expr),
        _ => None,
    }
}

pub fn identity_column(expr: &VExpr) -> (result: Option<usize>)
    ensures result == identity_column_spec(expr),
    decreases *expr,
{
    match expr {
        VExpr::Column(index) => Some(*index),
        VExpr::Alias { expr, .. } => identity_column(expr),
        _ => None,
    }
}

pub open spec fn schema_prefix(
    exprs: Seq<VExpr>, schema: Seq<VField>, count: nat,
) -> Option<Seq<VField>>
    recommends count <= exprs.len(),
    decreases count,
{
    if count == 0 { Some(Seq::empty()) }
    else {
        match (schema_prefix(exprs, schema, (count - 1) as nat),
               expr_field(&exprs[(count - 1) as int], schema)) {
            (Some(prefix), Some(field)) => Some(prefix.push(field)),
            _ => None,
        }
    }
}

/// Executable projection typing, equivalent to the semantic schema rule.
pub fn check_schema_prefix(
    exprs: &Vec<VExpr>, schema: &Vec<VField>, count: usize,
) -> (result: Option<Vec<VField>>)
    requires count <= exprs.len(),
    ensures
        match result {
            Some(fields) => schema_prefix(exprs@, schema@, count as nat) == Some(fields@),
            None => schema_prefix(exprs@, schema@, count as nat).is_None(),
        },
    decreases count,
{
    if count == 0 {
        Some(Vec::new())
    } else {
        match (check_schema_prefix(exprs, schema, count - 1),
               check_expr(&exprs[count - 1], schema)) {
            (Some(mut prefix), Some(field)) => {
                prefix.push(field);
                Some(prefix)
            },
            _ => None,
        }
    }
}

pub open spec fn row_prefix(
    exprs: Seq<VExpr>, row: Seq<Option<VValue>>, count: nat,
) -> Result<Seq<Option<VValue>>, ()>
    recommends count <= exprs.len(),
    decreases count,
{
    if count == 0 { Ok(Seq::empty()) }
    else {
        match (row_prefix(exprs, row, (count - 1) as nat),
               expr_row_spec(&exprs[(count - 1) as int], row)) {
            (Ok(prefix), Ok(cell)) => Ok(prefix.push(cell)),
            _ => Err(()),
        }
    }
}

pub proof fn lemma_row_prefix_type_sound(
    exprs: Seq<VExpr>, schema: Seq<VField>,
    row: Seq<Option<VValue>>, count: nat,
)
    requires
        row_in_schema(row, schema),
        count <= exprs.len(),
        schema_prefix(exprs, schema, count).is_Some(),
    ensures
        match (
            schema_prefix(exprs, schema, count),
            row_prefix(exprs, row, count),
        ) {
            (Some(fields), Ok(output)) => row_in_schema(output, fields),
            _ => false,
        },
    decreases count,
{
    if count > 0 {
        lemma_row_prefix_type_sound(
            exprs, schema, row, (count - 1) as nat);
        let field = expr_field(&exprs[(count - 1) as int], schema).unwrap();
        lemma_expr_field_sound(
            &exprs[(count - 1) as int], schema, row, field,
        );
        let prior_schema =
            schema_prefix(exprs, schema, (count - 1) as nat).unwrap();
        let prior =
            row_prefix(exprs, row, (count - 1) as nat).unwrap();
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

pub fn eval_prefix(
    exprs: &Vec<VExpr>, row: &Vec<Option<VValue>>, count: usize,
) -> (result: Result<Vec<Option<VValue>>, ()>)
    requires count <= exprs.len(),
    ensures
        match result {
            Ok(values) =>
                row_prefix(exprs@, row@, count as nat) == Ok(values@),
            Err(_) => row_prefix(exprs@, row@, count as nat).is_Err(),
        },
    decreases count,
{
    if count == 0 {
        Ok(Vec::new())
    } else {
        match eval_prefix(exprs, row, count - 1) {
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

} // verus!

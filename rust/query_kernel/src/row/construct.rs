//! Executable verified construction primitives used by OpenDP's make functions.
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

/// Reusing the source column's value-domain constraints is sound for
/// projections made of a column reference and zero or more aliases.
pub proof fn lemma_identity_column_value(
    expr: &VExpr, row: Seq<Option<VValue>>, index: usize,
)
    requires
        identity_column_spec(expr) == Some(index),
        index < row.len(),
    ensures expr_row_spec(expr, row) == Ok(row[index as int]),
    decreases *expr,
{
    match expr {
        VExpr::Column(i) => {
            assert(*i == index);
        },
        VExpr::Alias { expr: inner, .. } => {
            lemma_identity_column_value(inner, row, index);
        },
        _ => {
            assert(false);
        },
    }
}

/// Construct the identity transformation on a well-typed caller plan.
///
/// The result's schema comes from the kernel checker, not a parallel
/// interpretation of its typing rules.
pub fn make_input(
    input: VPlan, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        match result {
            Some((plan, fields)) =>
                plan_schema(&plan, source_schema@) == Some(fields@),
            None => true,
        },
{
    match check_plan(&input, source_schema) {
        Some(fields) => Some((input, fields)),
        None => None,
    }
}

/// Construct one projection and verify the *entire* resulting plan's schema.
pub fn make_project(
    input: VPlan, exprs: Vec<VExpr>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        match result {
            Some((plan, fields)) =>
                plan_schema(&plan, source_schema@) == Some(fields@),
            None => true,
        },
{
    match check_plan(&input, source_schema) {
        None => None,
        Some(child_schema) =>
            match project::check_schema_prefix(&exprs, &child_schema, exprs.len()) {
                None => None,
                Some(fields) => {
                    let plan = VPlan::Project { input: Box::new(input), exprs };
                    Some((plan, fields))
                },
            },
    }
}

/// Construct one filter and verify the *entire* resulting plan's schema.
pub fn make_filter(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        match result {
            Some((plan, fields)) =>
                plan_schema(&plan, source_schema@) == Some(fields@),
            None => true,
        },
{
    match check_plan(&input, source_schema) {
        None => None,
        Some(child_schema) =>
            match filter::check_schema(&predicate, &child_schema) {
                None => None,
                Some(fields) => {
                    let plan = VPlan::Filter { input: Box::new(input), predicate };
                    Some((plan, fields))
                },
            },
    }
}

} // verus!

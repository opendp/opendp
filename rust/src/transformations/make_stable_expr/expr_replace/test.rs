use crate::transformations::make_stable_expr::test_helper::replace::check_replace_expr;

use super::*;

fn check_replace(old: Expr, new: Expr, expected: Option<(Series, bool)>) -> Fallible<()> {
    check_replace_on_input(&[Some(1), Some(2), Some(3)], old, new, expected)
}

fn check_replace_on_input(
    input: &[Option<i32>],
    old: Expr,
    new: Expr,
    expected: Option<(Series, bool)>,
) -> Fallible<()> {
    check_replace_expr(input, col("").replace(old, new), expected)
}

#[test]
fn test_make_expr_replace_standard_scalar() -> Fallible<()> {
    // when replacing A with B, output remains nullable
    check_replace(
        lit(1),
        lit(10),
        Some((Series::new("".into(), [10, 2, 3]), true)),
    )
}

#[test]
fn test_make_expr_replace_standard_series() -> Fallible<()> {
    // when replacing each of old with respective new, output remains nullable
    check_replace(
        lit(Series::new("".into(), vec![1, 2])),
        lit(Series::new("".into(), vec![Some(10), None])),
        Some((Series::new("".into(), [Some(10), None, Some(3)]), true)),
    )
}
#[test]
fn test_make_expr_replace_impute_scalar() -> Fallible<()> {
    // replacing null with a non-null value removes nullability
    check_replace_on_input(
        &[Some(1), None, Some(3)],
        lit(NULL),
        lit(0),
        Some((Series::new("".into(), [1, 0, 3]), false)),
    )
}

#[test]
fn test_make_expr_replace_impute_series() -> Fallible<()> {
    // replacing null with a non-null value removes nullability
    check_replace_on_input(
        &[Some(1), None, Some(3)],
        lit(Series::new("".into(), vec![Some(1), None])),
        lit(Series::new("".into(), vec![10, 0])),
        Some((Series::new("".into(), [10, 0, 3]), false)),
    )
}

#[test]
fn test_make_expr_replace_wrong_dtype() -> Fallible<()> {
    check_replace(lit("A"), lit(45), None)
}

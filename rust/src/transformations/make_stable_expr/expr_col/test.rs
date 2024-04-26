use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_expr::test_helper::get_test_data;
use crate::transformations::StableExpr;

use super::*;

#[test]
fn test_make_col_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.row_by_row();
    let expected = col("const_1f64");
    let t_col = expected
        .clone()
        .make_stable(expr_domain.clone(), SymmetricDistance)?;
    let actual = t_col.invoke(&(lf.logical_plan, all()))?.1;

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_make_col_expr_no_wildcard() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.row_by_row();

    let t_col = col("const_1f64").make_stable(expr_domain.clone(), SymmetricDistance)?;
    let error_res = t_col
        .invoke(&(lf.logical_plan, col("not wildcard")))
        .map(|v| v.1)
        .unwrap_err()
        .variant;
    assert_eq!(error_res, ErrorVariant::FailedFunction);

    Ok(())
}

#[test]
fn test_make_col_expr_wrong_col() -> Fallible<()> {
    let (lf_domain, _) = get_test_data()?;
    let expr_domain = lf_domain.row_by_row();

    let variant = col("nonexistent")
        .make_stable(expr_domain, SymmetricDistance)
        .map(|_: Transformation<_, _, _, SymmetricDistance>| ())
        .unwrap_err()
        .variant;

    assert_eq!(variant, ErrorVariant::MakeTransformation);
    Ok(())
}

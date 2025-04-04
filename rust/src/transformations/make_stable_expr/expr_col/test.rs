use crate::error::ErrorVariant;
use crate::metrics::{FrameDistance, SymmetricDistance};
use crate::transformations::StableExpr;
use crate::transformations::make_stable_expr::test_helper::get_test_data;

use super::*;

#[test]
fn test_make_col_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.row_by_row();
    let expected = col("const_1f64");
    let t_col = expected
        .clone()
        .make_stable(expr_domain.clone(), FrameDistance(SymmetricDistance))?;
    let actual = t_col.invoke(&lf.logical_plan)?.expr;

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_make_col_expr_wrong_col() -> Fallible<()> {
    let (lf_domain, _) = get_test_data()?;
    let expr_domain = lf_domain.row_by_row();

    let variant = col("nonexistent")
        .make_stable(expr_domain, FrameDistance(SymmetricDistance))
        .map(|_: Transformation<_, _, _, FrameDistance<SymmetricDistance>>| ())
        .unwrap_err()
        .variant;

    assert_eq!(variant, ErrorVariant::MakeTransformation);
    Ok(())
}

use crate::{
    metrics::{InsertDeleteDistance, L2Distance, PartitionDistance},
    transformations::test_helper::get_test_data,
};

use polars::prelude::when;

use super::*;

#[test]
fn test_ternary() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    // 1. had problems finding when in Expr:
    // https://docs.rs/polars/latest/polars/prelude/enum.Expr.html
    // 2. searched in their docs for when, found this:
    // https://docs.rs/polars/latest/polars/prelude/fn.when.html
    // 3. updated below to use when function instead of method

    let t_sum: Transformation<_, _, _, InsertDeleteDistance> = when(col("const_1f64").eq(lit(1)))
        .then(lit(1))
        .otherwise(lit(0))
        .make_stable(lf_domain.row_by_row(), InsertDeleteDistance)?;

    // TODO: Make some assertions: This is just copy-paste from sum.

    // let expr_res = t_sum.invoke(&lf.logical_plan)?.expr;
    // // dtype in clip changes
    // assert_eq!(expr_res, col("const_1f64").clip(lit(0.), lit(1.)).sum());

    // let sens = t_sum.map(&(4, 4, 1))?;
    // println!("sens: {:?}", sens);
    // assert!(sens > 2.);
    // assert!(sens < 2.00001);
    Ok(())
}

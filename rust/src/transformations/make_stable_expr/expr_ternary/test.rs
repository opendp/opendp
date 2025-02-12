use crate::{
    metrics::{InsertDeleteDistance, L2Distance, PartitionDistance},
    transformations::test_helper::get_test_data,
};

use super::*;

#[test]
fn test_ternary() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    // TODO: This doesn't work:
    //   no method named `when` found for enum `polars::prelude::Expr` in the current scope
    let t_sum: Transformation<_, _, _, L2Distance<f64>> = col("const_1f64")
        .when(col("const_1f64").eq(lit(1)))
        .then(lit(1))
        .otherwise(lit(0))
        .make_stable(lf_domain, PartitionDistance(InsertDeleteDistance))?;

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

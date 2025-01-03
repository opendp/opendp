use polars::df;
use polars_plan::dsl::len;

use crate::{
    core::Transformation,
    metrics::{InsertDeleteDistance, L2Distance, PartitionDistance},
    transformations::{test_helper::get_test_data, StableExpr},
};

use super::Fallible;

#[test]
fn test_select_make_expr_len() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.aggregate(["chunk_2_bool", "cycle_5_alpha"]);

    let t_sum: Transformation<_, _, _, L2Distance<f64>> =
        len().make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;
    let expr_res = t_sum.invoke(&lf.logical_plan)?.expr;
    assert_eq!(expr_res, len());

    let sensitivity = t_sum.map(&(4, 4, 1))?;
    println!("sens: {:?}", sensitivity);
    assert_eq!(sensitivity, 2.);
    Ok(())
}

#[test]
fn test_grouped_make_len_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.aggregate(["chunk_(..10u32)"]);

    // Get resulting sum (expression result)
    let t_sum: Transformation<_, _, _, L2Distance<f64>> =
        len().make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;
    let expr_res = t_sum.invoke(&lf.logical_plan)?.expr;

    let df_actual = lf
        .group_by(["chunk_(..10u32)"])
        .agg([expr_res])
        .collect()?
        .sort(["chunk_(..10u32)"], Default::default())?;

    let df_expected = df!(
        "chunk_(..10u32)" => [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        "len" => [100u32; 10]
    )?;

    assert_eq!(df_actual, df_expected);

    // assume we're in a grouping context.
    // By the following triple, we know
    // 1. an individual can influence up to 10 partitions (l0)
    // 2. an individual can contribute up to 10 records total (l1)
    // 3. an individual can contribute at most 1 record to any partition (linf)
    let sens = t_sum.map(&(10, 10, 1))?;

    // The sensitivity d_out under the l2 distance in unbounded DP is given by the following formula:
    // = min(sqrt(l0) * map(linf)         , map(l1))
    // = min(sqrt(l0) * linf * max(|L|, U), l1 * max(|L|, U))
    // = min(sqrt(10) * 1, 10)
    // = min(3.16227, 10)
    // = 3.16227

    // that is, in the worst case, we know the sum will differ by at most 1 in 10 partitions,
    // so the l2 distance between any two outputs on neighboring data sets is at most 3.16227

    // The sensitivity is slightly higher to account for potential rounding errors.
    println!("sens: {:?}", sens);
    assert!(sens > (3.16227).into());
    assert!(sens < (3.162278).into());
    Ok(())
}

#[test]
fn test_select_make_expr_len_row_by_row() -> Fallible<()> {
    // this transformation should refuse to build in a row-by-row context like `with_columns`
    let (lf_domain, _) = get_test_data()?;
    let expr_domain = lf_domain.row_by_row();

    assert!(len()
        .make_stable(expr_domain, InsertDeleteDistance)
        .map(|_: Transformation<_, _, _, InsertDeleteDistance>| ())
        .is_err());

    Ok(())
}

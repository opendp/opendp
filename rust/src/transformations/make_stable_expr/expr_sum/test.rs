use crate::{
    metrics::{InsertDeleteDistance, L2Distance, PartitionDistance},
    transformations::test_helper::get_test_data,
};

use super::*;

#[test]
fn test_select_make_sum_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.aggregate(["chunk_2_bool", "cycle_5_alpha"]);

    // Get resulting sum (expression result)
    let t_sum: Transformation<_, _, _, L2Distance<f64>> = col("const_1f64")
        .clip(lit(0), lit(1))
        .sum()
        .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;
    let expr_res = t_sum.invoke(&lf.logical_plan)?.expr;
    // dtype in clip changes
    assert_eq!(expr_res, col("const_1f64").clip(lit(0.), lit(1.)).sum());

    let sens = t_sum.map(&(4, 4, 1))?;
    println!("sens: {:?}", sens);
    assert!(sens > 2.);
    assert!(sens < 2.00001);
    Ok(())
}

#[test]
fn test_grouped_make_sum_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let expr_domain = lf_domain.aggregate(["chunk_(..10u32)"]);

    // Get resulting sum (expression result)
    let t_sum: Transformation<_, _, _, L2Distance<f64>> = col("cycle_(..100i32)")
        .clip(lit(0), lit(1))
        .sum()
        .clone()
        .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))?;
    let expr_res = t_sum.invoke(&lf.logical_plan)?.expr;

    let df_actual = lf
        .group_by(["chunk_(..10u32)"])
        .agg([expr_res])
        .collect()?
        .sort(["chunk_(..10u32)"], Default::default())?;

    let df_expected = df!(
        "chunk_(..10u32)" => [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        "cycle_(..100i32)" => [99i32; 10]
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
fn test_overflow_sum_expr() -> Fallible<()> {
    let (lf_domain, _) = get_test_data()?;
    let expr_domain = lf_domain.aggregate(["chunk_2_bool"]);

    // Get resulting sum (expression result)
    let err = col("chunk_(..10u32)")
        .clip(lit(0), lit(u32::MAX))
        .sum()
        .clone()
        .make_stable(expr_domain, PartitionDistance(InsertDeleteDistance))
        .map(|_: Transformation<_, _, _, L2Distance<f64>>| ())
        .unwrap_err();

    assert_eq!(err.variant, ErrorVariant::MakeTransformation);
    Ok(())
}

#[test]
fn test_polars_sum_types() -> Fallible<()> {
    let lf = df!(
        "i8" => &[1i8, 2, 3],
        "i16" => &[1i16, 2, 3],
        "i32" => &[1i32, 2, 3],
        "i64" => &[1i64, 2, 3],
        "u32" => &[1u32, 2, 3],
        "u64" => &[1u64, 2, 3],
        "f32" => &[1f32, 2.0, 3.0],
        "f64" => &[1f64, 2.0, 3.0],
    )?
    .lazy();

    let schema = lf.select([all().sum()]).collect()?.schema();

    macro_rules! test_dtype {
        ($dtype:ident, $expected:ident) => {
            assert_eq!(
                schema.get_field(stringify!($dtype)).unwrap().dtype(),
                &DataType::$expected
            );
        };
    }

    test_dtype!(i8, Int64);
    test_dtype!(i16, Int64);
    test_dtype!(i32, Int32);
    test_dtype!(i64, Int64);
    test_dtype!(u32, UInt32);
    test_dtype!(u64, UInt64);
    test_dtype!(f32, Float32);
    test_dtype!(f64, Float64);

    Ok(())
}

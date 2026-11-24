use crate::{
    metrics::{L0PInfDistance, L2Distance, SymmetricDistance},
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
        .make_stable(expr_domain, L0PInfDistance(SymmetricDistance))?;
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
        .make_stable(expr_domain, L0PInfDistance(SymmetricDistance))?;
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

    assert_eq!(t_sum.map(&(3, 5, 2))?, 3.);
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
        .make_stable(expr_domain, L0PInfDistance(SymmetricDistance))
        .map(|_: Transformation<_, _, _, L2Distance<f64>>| ())
        .unwrap_err();

    assert_eq!(err.variant, ErrorVariant::MakeTransformation);
    Ok(())
}

#[test]
fn test_float_accumulator() -> Fallible<()> {
    let n = 16;
    assert_eq!(
        <f32 as Accumulator>::relaxation(n, -1., 1.)?,
        crate::accuracy::float_sum_sensitivity::<f32>(0., n - 1, n as f64, n as f64)?
    );
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
    let lf2 = lf.select([all().as_expr().sum()]).collect()?;
    let schema = lf2.schema();

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

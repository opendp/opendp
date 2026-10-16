use polars::df;
use polars::lazy::frame::IntoLazy;
use polars::prelude::{NamedFrom, col, lit};
use polars::series::Series;
use polars_plan::plans::typed_lit;

use crate::core::Domain;
use crate::domains::{AtomDomain, LazyFrameDomain, Margin, OptionDomain, SeriesDomain};
use crate::metrics::{FrameDistance, L0PInfDistance, L2Distance, SymmetricDistance};

use super::*;

#[test]
fn test_make_expr_fill_null() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "i32",
        OptionDomain::new(AtomDomain::<i32>::default()),
    )])?;

    let lf = df!("i32" => [None, Some(1)])?.lazy();

    let t_fill_null = col("i32").fill_null(0).make_stable(
        lf_domain.clone().row_by_row(),
        FrameDistance(SymmetricDistance),
    )?;
    let expr_fill_null = t_fill_null.invoke(&lf.logical_plan)?.expr;
    println!("{:?}", expr_fill_null);
    let actual = lf.with_column(expr_fill_null).collect()?;

    assert_eq!(actual, df!("i32" => [0, 1])?);

    assert!(!t_fill_null.output_domain.column.nullable);

    Ok(())
}

#[test]
fn test_make_expr_fill_null_expr() -> Fallible<()> {
    let f64_null = SeriesDomain::new("f64_null", OptionDomain::new(AtomDomain::<f64>::default()));
    let f64_nonnull = SeriesDomain::new("f64_nonnull", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![f64_null, f64_nonnull])?;

    let lf = df!(
        "f64_null" => [None, Some(1.), Some(f64::NAN)],
        "f64_nonnull" => [0f64; 3]
    )?
    .lazy();

    let t_fill_null = col("f64_null").fill_null(col("f64_nonnull")).make_stable(
        lf_domain.clone().row_by_row(),
        FrameDistance(SymmetricDistance),
    )?;

    let expr_fill_null = t_fill_null.invoke(&lf.logical_plan)?.expr;
    let actual = lf.with_column(expr_fill_null).collect()?;
    let actual_f64_null = actual.column("f64_null")?.as_materialized_series();

    assert_eq!(
        actual_f64_null,
        &Series::new("f64_null".into(), [Some(0.), Some(1.), Some(f64::NAN)])
    );

    assert!(!t_fill_null.output_domain.column.nullable);

    Ok(())
}

#[test]
fn test_make_expr_fill_null_expr_filter_fail() -> Fallible<()> {
    let f64_null = SeriesDomain::new("f64_null", OptionDomain::new(AtomDomain::<f64>::default()));
    let f64_nonnull = SeriesDomain::new("f64_nonnull", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![f64_null, f64_nonnull])?;

    let err = col("f64_null")
        .fill_null(lit(0.0).filter(col("f64_nonnull").gt_eq(0.0)))
        .make_stable(
            lf_domain.clone().row_by_row(),
            FrameDistance(SymmetricDistance),
        )
        .unwrap_err();

    assert_eq!(
        err.message,
        Some("filter is not allowed in a row-by-row context".to_string())
    );
    Ok(())
}

macro_rules! test_fill_null_nan_domain {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() -> Fallible<()> {
            let bounded = AtomDomain::<$ty>::new_closed((0., 1.))?;
            let finite_data = [None, Some(1 as $ty), Some(0 as $ty)];
            let nan_data = [None, Some(1 as $ty), Some(<$ty>::NAN)];

            for (case, data_domain, data, expr, expected, expected_nan) in [
                (
                    "NaN literal",
                    bounded.clone(),
                    finite_data,
                    col("data").fill_null(typed_lit(<$ty>::NAN)),
                    [<$ty>::NAN, 1., 0.],
                    true,
                ),
                (
                    "NaN-capable column",
                    bounded.clone(),
                    finite_data,
                    col("data").fill_null(col("fill")),
                    [<$ty>::NAN, 1., 0.],
                    true,
                ),
                (
                    "fill_nan then fill_null",
                    AtomDomain::<$ty>::default(),
                    nan_data,
                    col("data")
                        .fill_nan(typed_lit(0 as $ty))
                        .fill_null(typed_lit(<$ty>::NAN)),
                    [<$ty>::NAN, 1., 0.],
                    true,
                ),
                (
                    "finite filler",
                    bounded,
                    finite_data,
                    col("data").fill_null(typed_lit(2 as $ty)),
                    [2., 1., 0.],
                    false,
                ),
                (
                    "preserve input NaN",
                    AtomDomain::<$ty>::default(),
                    nan_data,
                    col("data").fill_null(typed_lit(0 as $ty)),
                    [0., 1., <$ty>::NAN],
                    true,
                ),
            ] {
                let lf_domain = LazyFrameDomain::new(vec![
                    SeriesDomain::new("data", OptionDomain::new(data_domain)),
                    SeriesDomain::new("fill", AtomDomain::<$ty>::default()),
                ])?;
                let lf = df!(
                    "data" => data,
                    "fill" => [<$ty>::NAN, 2., 3.]
                )?
                .lazy();
                let input_domain = lf_domain.row_by_row();
                assert!(input_domain.member(&lf.logical_plan)?, "{case}");

                let t_fill_null = expr
                    .make_stable(input_domain.clone(), FrameDistance(SymmetricDistance))?;
                let output_domain = &t_fill_null.output_domain;
                let output_atom_domain = output_domain.column.atom_domain::<$ty>()?;
                assert_eq!(output_atom_domain.nan(), expected_nan, "{case}");
                assert!(output_atom_domain.bounds.is_none(), "{case}");
                assert!(!output_domain.column.nullable, "{case}");
                assert_eq!(output_domain.context, input_domain.context, "{case}");

                let output = t_fill_null.invoke(&lf.logical_plan)?;
                assert!(output_domain.member(&output)?, "{case}");
                let actual = lf.select([output.expr]).collect()?;
                assert_eq!(actual, df!("data" => expected)?, "{case}");
            }
            Ok(())
        }
    };
}

test_fill_null_nan_domain!(test_make_expr_fill_null_nan_domain_f32, f32);
test_fill_null_nan_domain!(test_make_expr_fill_null_nan_domain_f64, f64);

#[test]
fn test_make_expr_fill_null_coercion() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new(
            "data32",
            OptionDomain::new(AtomDomain::<f32>::new_non_nan()),
        ),
        SeriesDomain::new(
            "data64",
            OptionDomain::new(AtomDomain::<f64>::new_non_nan()),
        ),
        SeriesDomain::new("fill32", AtomDomain::<f32>::default()),
        SeriesDomain::new("fill64", AtomDomain::<f64>::default()),
    ])?
    .row_by_row();
    let lf = df!(
        "data32" => [None, Some(1f32)],
        "data64" => [None, Some(1f64)],
        "fill32" => [f32::NAN, 0.],
        "fill64" => [f64::NAN, 0.]
    )?
    .lazy();
    assert!(input_domain.member(&lf.logical_plan)?);

    for (expr, expected_dtype, expected_nan) in [
        (
            col("data32").fill_null(typed_lit(f64::NAN)),
            DataType::Float64,
            true,
        ),
        (
            col("data32").fill_null(col("fill64")),
            DataType::Float64,
            true,
        ),
        (
            col("data64").fill_null(typed_lit(f32::NAN)),
            DataType::Float64,
            true,
        ),
        (
            col("data64").fill_null(col("fill32")),
            DataType::Float64,
            true,
        ),
        (
            col("data32").fill_null(typed_lit(0i32)),
            DataType::Float64,
            false,
        ),
        (col("data64").fill_null(0), DataType::Float64, false),
        // Dynamic literals adopt the column dtype on either side of fill_null.
        (
            col("data32").fill_null(lit(f64::NAN)),
            DataType::Float32,
            true,
        ),
        (lit(0.).fill_null(col("fill32")), DataType::Float32, false),
        (
            lit(0.).alias("constant").fill_null(col("fill32")),
            DataType::Float32,
            false,
        ),
        (
            (lit(0.) + lit(1.)).fill_null(col("fill32")),
            DataType::Float32,
            true,
        ),
    ] {
        let t_fill_null = expr
            .clone()
            .make_stable(input_domain.clone(), FrameDistance(SymmetricDistance))?;
        let output_domain = &t_fill_null.output_domain;
        assert_eq!(output_domain.column.dtype(), expected_dtype, "{expr:?}");
        let nan = match expected_dtype {
            DataType::Float32 => output_domain.column.atom_domain::<f32>()?.nan(),
            DataType::Float64 => output_domain.column.atom_domain::<f64>()?.nan(),
            _ => unreachable!(),
        };
        assert_eq!(nan, expected_nan, "{expr:?}");
        assert!(!output_domain.column.nullable, "{expr:?}");
        assert_eq!(output_domain.context, input_domain.context, "{expr:?}");
        let output = t_fill_null.invoke(&lf.logical_plan)?;
        assert!(output_domain.member(&output)?, "{expr:?}");
    }
    Ok(())
}

#[test]
fn test_make_expr_fill_null_nullable_filler_fail() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "data",
        OptionDomain::new(AtomDomain::<f64>::new_non_nan()),
    )])?
    .row_by_row();
    let err = col("data")
        .fill_null(col("data"))
        .make_stable(input_domain, FrameDistance(SymmetricDistance))
        .unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some("fill expression must not be nullable")
    );
    Ok(())
}

#[test]
fn test_make_expr_fill_null_nonnull_nan_filler() -> Fallible<()> {
    for nullable in [false, true] {
        let mut data_domain = SeriesDomain::new("data", AtomDomain::<f64>::new_non_nan());
        data_domain.nullable = nullable;
        let input_domain = LazyFrameDomain::new(vec![
            data_domain,
            SeriesDomain::new("fill", AtomDomain::<f64>::default()),
        ])?
        .row_by_row();
        let lf = df!(
            "data" => [if nullable { None } else { Some(0.) }, Some(1.)],
            "fill" => [f64::NAN, 0.]
        )?
        .lazy();
        assert!(input_domain.member(&lf.logical_plan)?);
        let t_fill_null = col("data")
            .fill_null(col("fill"))
            .make_stable(input_domain, FrameDistance(SymmetricDistance))?;
        let output_domain = &t_fill_null.output_domain;
        assert_eq!(output_domain.column.atom_domain::<f64>()?.nan(), nullable);
        assert!(output_domain.member(&t_fill_null.invoke(&lf.logical_plan)?)?);
    }
    Ok(())
}

#[test]
fn test_make_expr_fill_null_nonnumeric_coercion_fail() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "data",
        OptionDomain::new(AtomDomain::<f64>::new_non_nan()),
    )])?
    .row_by_row();
    let err = col("data")
        .fill_null(lit("missing"))
        .make_stable(input_domain, FrameDistance(SymmetricDistance))
        .unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some("fill_null with different input and filler types is only supported for numeric data")
    );
    Ok(())
}

#[test]
fn test_make_expr_fill_null_nan_sum_fail() -> Fallible<()> {
    let input_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "data",
        OptionDomain::new(AtomDomain::<f64>::new_non_nan()),
    )])?
    .with_margin(Margin::select().with_max_length(100))?
    .select();
    let err = col("data")
        .fill_null(typed_lit(f64::NAN))
        .clip(typed_lit(0f64), typed_lit(1f64))
        .sum()
        .make_stable(input_domain, L0PInfDistance(SymmetricDistance))
        .map(|_: Transformation<_, _, _, L2Distance<f64>>| ())
        .unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some("input data (data) might contain nans. Preprocess your data with `.fill_nan`.")
    );
    Ok(())
}

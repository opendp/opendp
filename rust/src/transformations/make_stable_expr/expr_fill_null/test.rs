use polars::df;
use polars::lazy::frame::IntoLazy;
use polars::prelude::{DataType, NamedFrom, col, lit};
use polars::series::Series;
use polars_plan::plans::typed_lit;

use crate::core::Domain;

use crate::domains::{AtomDomain, Invariant, LazyFrameDomain, Margin, OptionDomain, SeriesDomain};
use crate::metrics::{FrameDistance, SymmetricDistance};

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

#[test]
fn test_fill_preserves_filter_context() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("x", AtomDomain::<f64>::default()),
        SeriesDomain::new("k", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["k"]).with_invariant_lengths())?
    .aggregate(["k"]);

    for expr in [
        col("x").filter(col("x").gt(0.0)).fill_null(0.0),
        col("x").filter(col("x").gt(0.0)).fill_nan(0.0),
    ] {
        let t_fill = expr.make_stable(domain.clone(), FrameDistance(SymmetricDistance))?;
        assert_eq!(
            t_fill.output_domain.context.aggregation("fill")?.invariant,
            Some(Invariant::Keys)
        );
    }
    Ok(())
}

#[test]
fn test_fill_preserves_aggregation_context_with_column_fill() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("x", OptionDomain::new(AtomDomain::<f64>::default())),
        SeriesDomain::new("y", AtomDomain::<f64>::new_non_nan()),
        SeriesDomain::new("k", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["k"]).with_invariant_lengths())?
    .aggregate(["k"]);

    for expr in [col("x").fill_null(col("y")), col("x").fill_nan(col("y"))] {
        let t_fill = expr.make_stable(domain.clone(), FrameDistance(SymmetricDistance))?;
        assert_eq!(
            t_fill.output_domain.context.aggregation("fill")?.invariant,
            Some(Invariant::Lengths)
        );
    }
    Ok(())
}

#[test]
fn test_fill_null_coercion_and_nan_domain() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new(
            "x",
            OptionDomain::new(AtomDomain::<f32>::new_closed((0.0, 1.0))?),
        ),
        SeriesDomain::new("y", AtomDomain::<f64>::default()),
        SeriesDomain::new("z", AtomDomain::<f64>::new_non_nan()),
    ])?
    .row_by_row();
    let data = df!(
        "x" => [None, Some(1f32)],
        "y" => [f64::NAN, 2.0],
        "z" => [3.0f64, 4.0],
    )?
    .lazy();

    for (expr, expected_dtype, expected_nan) in [
        (col("x").fill_null(lit(f64::NAN)), DataType::Float32, true),
        (
            col("x").fill_null(typed_lit(f64::NAN)),
            DataType::Float64,
            true,
        ),
        (col("x").fill_null(col("y")), DataType::Float64, true),
        (col("x").fill_null(col("z")), DataType::Float64, false),
        (col("x").fill_null(lit(0.0f64)), DataType::Float32, false),
        (
            col("x").fill_null(typed_lit(0i32)),
            DataType::Float64,
            false,
        ),
    ] {
        let transformation = expr
            .clone()
            .make_stable(domain.clone(), FrameDistance(SymmetricDistance))?;
        let output = &transformation.output_domain;
        assert_eq!(output.column.dtype(), expected_dtype, "{expr:?}");
        assert_eq!(
            data.clone().select([expr.clone()]).collect()?.get_columns()[0].dtype(),
            &expected_dtype,
            "{expr:?}"
        );
        let actual_nan = match expected_dtype {
            DataType::Float32 => output.column.atom_domain::<f32>()?.nan(),
            DataType::Float64 => output.column.atom_domain::<f64>()?.nan(),
            _ => unreachable!(),
        };
        assert_eq!(actual_nan, expected_nan, "{expr:?}");
        assert!(!output.column.nullable, "{expr:?}");
        assert!(
            output.member(&transformation.invoke(&data.logical_plan)?)?,
            "{expr:?}"
        );
    }

    let nonnull_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("x", AtomDomain::<f32>::new_non_nan()),
        SeriesDomain::new("y", AtomDomain::<f64>::default()),
    ])?
    .row_by_row();
    let transformation = col("x")
        .fill_null(col("y"))
        .make_stable(nonnull_domain, FrameDistance(SymmetricDistance))?;
    assert_eq!(
        transformation.output_domain.column.dtype(),
        DataType::Float64
    );
    assert!(
        !transformation
            .output_domain
            .column
            .atom_domain::<f64>()?
            .nan()
    );

    let err = col("x")
        .fill_null(lit("missing"))
        .make_stable(domain, FrameDistance(SymmetricDistance))
        .unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some("fill_null with different input and filler types is only supported for numeric data")
    );
    Ok(())
}

#[test]
fn test_fill_null_integer_literal_coercion() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "i64",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?
    .row_by_row();
    let data = df!("i64" => [None, Some(1i64)])?.lazy();

    for (expr, dtype) in [
        (col("i64").fill_null(0), DataType::Int64),
        (col("i64").fill_null(0.0f64), DataType::Float64),
    ] {
        let transformation = expr
            .clone()
            .make_stable(domain.clone(), FrameDistance(SymmetricDistance))?;
        let output = data.clone().select([expr]).collect()?;

        assert_eq!(output.get_columns()[0].dtype(), &dtype);
        assert_eq!(transformation.output_domain.column.dtype(), dtype);
        assert!(
            transformation
                .output_domain
                .member(&transformation.invoke(&data.logical_plan)?)?
        );
    }
    Ok(())
}

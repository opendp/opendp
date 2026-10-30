use polars::df;
use polars::lazy::frame::IntoLazy;
use polars::prelude::{DataType, NamedFrom, col, lit};
use polars::series::Series;
use polars_plan::plans::typed_lit;

use crate::core::Domain;

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::{FrameDistance, SymmetricDistance};

use super::*;

#[test]
fn test_make_expr_fill_nan_literal() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "f64",
        OptionDomain::new(AtomDomain::<f64>::default()),
    )])?;

    let lf = df!("f64" => [None, Some(1.), Some(f64::NAN)])?.lazy();

    let t_fill_nan = col("f64").fill_nan(0.0).make_stable(
        lf_domain.clone().row_by_row(),
        FrameDistance(SymmetricDistance),
    )?;
    let expr_fill_nan = t_fill_nan.invoke(&lf.logical_plan)?.expr;
    let actual = lf.with_column(expr_fill_nan).collect()?;

    assert_eq!(actual, df!("f64" => [None, Some(1.), Some(0.)])?);

    assert!(!t_fill_nan.output_domain.column.atom_domain::<f64>()?.nan());

    Ok(())
}

#[test]
fn test_make_expr_fill_nan_expr() -> Fallible<()> {
    let f64_nan = SeriesDomain::new("f64_nan", OptionDomain::new(AtomDomain::<f64>::default()));
    let f64_nonnan = SeriesDomain::new("f64_nonnan", AtomDomain::<f64>::new_non_nan());
    let lf_domain = LazyFrameDomain::new(vec![f64_nan, f64_nonnan])?;

    let lf = df!(
        "f64_nan" => [None, Some(1.), Some(f64::NAN)],
        "f64_nonnan" => [0f64; 3]
    )?
    .lazy();

    let t_fill_nan = col("f64_nan").fill_nan(col("f64_nonnan")).make_stable(
        lf_domain.clone().row_by_row(),
        FrameDistance(SymmetricDistance),
    )?;

    let expr_fill_nan = t_fill_nan.invoke(&lf.logical_plan)?.expr;
    let actual = lf.with_column(expr_fill_nan).collect()?;
    let actual_f64_nan = actual.column("f64_nan")?.as_materialized_series();

    assert_eq!(
        actual_f64_nan,
        &Series::new("f64_nan".into(), [None, Some(1.), Some(0.)])
    );

    assert!(!t_fill_nan.output_domain.column.atom_domain::<f64>()?.nan());

    Ok(())
}

#[test]
fn test_make_expr_fill_nan_expr_filter_fail() -> Fallible<()> {
    let f64_nan = SeriesDomain::new("f64_nan", OptionDomain::new(AtomDomain::<f64>::default()));
    let f64_nonnan = SeriesDomain::new("f64_nonnan", AtomDomain::<f64>::new_non_nan());
    let lf_domain = LazyFrameDomain::new(vec![f64_nan, f64_nonnan])?;

    let err = col("f64_nan")
        .fill_nan(lit(0.0).filter(col("f64_nonnan").gt_eq(0.0)))
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
fn test_fill_nan_coercion() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("x", OptionDomain::new(AtomDomain::<f32>::default())),
        SeriesDomain::new("y", AtomDomain::<f64>::new_non_nan()),
    ])?
    .row_by_row();
    let data = df!(
        "x" => [None, Some(f32::NAN), Some(1f32)],
        "y" => [0.0f64, 2.0, 3.0],
    )?
    .lazy();

    for (expr, expected_dtype) in [
        (col("x").fill_nan(lit(0.0f64)), DataType::Float32),
        (col("x").fill_nan(typed_lit(0.0f64)), DataType::Float64),
        (col("x").fill_nan(col("y")), DataType::Float64),
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
        assert!(output.column.nullable, "{expr:?}");
        let nan = match expected_dtype {
            DataType::Float32 => output.column.atom_domain::<f32>()?.nan(),
            DataType::Float64 => output.column.atom_domain::<f64>()?.nan(),
            _ => unreachable!(),
        };
        assert!(!nan, "{expr:?}");
        assert!(
            output.member(&transformation.invoke(&data.logical_plan)?)?,
            "{expr:?}"
        );
    }
    Ok(())
}

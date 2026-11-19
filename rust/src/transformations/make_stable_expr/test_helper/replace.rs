use polars::prelude::*;
use polars_plan::plans::{DynLiteralValue, LiteralValue};

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::error::*;
use crate::metrics::{FrameDistance, SymmetricDistance};
use crate::transformations::{StableExpr, make_stable_lazyframe};

pub(crate) fn check_replace_expr(
    input: &[Option<i32>],
    replacement: Expr,
    expected: Option<(Series, bool)>,
) -> Fallible<()> {
    let series_domain = SeriesDomain::new("", OptionDomain::new(AtomDomain::<i32>::default()));
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?;
    let lf = df!("" => input)?.lazy();

    let t_obs = make_stable_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        lf.clone().with_column(replacement),
    );

    let (t_obs, expected) = match (t_obs, expected) {
        (Err(_), None) => return Ok(()),
        (Ok(observed), Some(expected)) => (observed, expected),
        (Err(e), Some(_)) => return Err(e),
        (Ok(_), None) => panic!("expected error, but did not error"),
    };

    let df_observed = t_obs.invoke(&lf)?.collect()?;
    assert_eq!(
        df_observed.column("")?.as_materialized_series(),
        &expected.0
    );

    let output_domain = t_obs.output_domain.series_domain("".into())?;
    assert_eq!(output_domain.nullable, expected.1);
    assert_eq!(
        output_domain.dtype(),
        df_observed.schema().get("").cloned().unwrap()
    );
    Ok(())
}

#[test]
fn test_replace_variants_reject_reserved_names() -> Fallible<()> {
    for name in [
        "__POLARS_REPLACE_OLD",
        "__POLARS_REPLACE_NEW",
        "__POLARS_REPLACE_MASK",
    ] {
        let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
            name,
            OptionDomain::new(AtomDomain::<i32>::default()),
        )])?;
        let old = lit(Series::new("".into(), [Some(1i32), None]));
        let new = lit(Series::new("".into(), [10i32, 0]));
        for expr in [
            col(name).replace(old.clone(), new.clone()),
            col(name).replace_strict(old, new, Some(lit(5i32)), None::<DataType>),
        ] {
            assert!(
                expr.make_stable(
                    domain.clone().row_by_row(),
                    FrameDistance(SymmetricDistance)
                )
                .is_err(),
                "reserved column name {name} must be rejected",
            );
        }
    }
    Ok(())
}

#[test]
fn test_replace_variants_reject_overflowing_dynamic_integer() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new("u", AtomDomain::<u64>::default())])?;
    for value in [i128::MIN, -1i128, (u64::MAX as i128) + 1, i128::MAX] {
        let new = Expr::Literal(LiteralValue::Dyn(DynLiteralValue::Int(value)));
        for expr in [
            col("u").replace(lit(1u64), new.clone()),
            col("u").replace_strict(lit(1u64), new, Some(lit(0u64)), Some(DataType::UInt64)),
        ] {
            assert!(
                expr.make_stable(
                    domain.clone().row_by_row(),
                    FrameDistance(SymmetricDistance)
                )
                .is_err(),
                "out-of-range value {value} must be rejected",
            );
        }
    }
    Ok(())
}

#[test]
fn test_replace_variants_reject_duration_to_string() -> Fallible<()> {
    let duration =
        Series::new("".into(), [i64::MIN]).cast(&DataType::Duration(TimeUnit::Nanoseconds))?;
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "s",
        AtomDomain::<String>::default(),
    )])?;

    for expr in [
        col("s").replace(lit(duration.clone()), lit("value")),
        col("s").replace_strict(
            lit(duration),
            lit("value"),
            Some(lit("missing")),
            None::<DataType>,
        ),
    ] {
        assert!(
            expr.make_stable(
                domain.clone().row_by_row(),
                FrameDistance(SymmetricDistance)
            )
            .is_err()
        );
    }
    Ok(())
}

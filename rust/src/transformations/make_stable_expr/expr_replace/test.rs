use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

use super::*;

fn check_replace(old: Expr, new: Expr, expected: Option<(Series, bool)>) -> Fallible<()> {
    let series_domain = SeriesDomain::new("", OptionDomain::new(AtomDomain::<i32>::default()));
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?;
    let lf = df!("" => &[1, 2, 3])?.lazy();

    let lf_replace = lf.clone().with_column(col("").replace(old, new));
    let t_obs = make_stable_lazyframe(lf_domain.clone(), SymmetricDistance, lf_replace);

    let (t_obs, expected) = match (t_obs, expected) {
        (Err(_), None) => return Ok(()),
        (Ok(observed), Some(expected)) => (observed, expected),
        (Err(e), Some(_)) => return Err(e),
        (Ok(_), None) => panic!("expected error, but did not error"),
    };

    // check that polars behavior is as expected
    let df_observed = t_obs.invoke(&lf)?.collect()?;
    assert_eq!(
        df_observed.column("")?.as_materialized_series(),
        &expected.0
    );

    // since the replacement has nullity, the outcome may have nullity
    let new_series_domain = t_obs.output_domain.series_domain("".into())?;
    assert_eq!(new_series_domain.nullable, expected.1);
    Ok(())
}

#[test]
fn test_make_expr_replace_standard_scalar() -> Fallible<()> {
    // when replacing A with B, output remains nullable
    check_replace(
        lit(1),
        lit(10),
        Some((Series::new("".into(), [10, 2, 3]), true)),
    )
}

#[test]
fn test_make_expr_replace_standard_series() -> Fallible<()> {
    // when replacing each of old with respective new, output remains nullable
    check_replace(
        lit(Series::new("".into(), vec![1, 2])),
        lit(Series::new("".into(), vec![Some(10), None])),
        Some((Series::new("".into(), [Some(10), None, Some(3)]), true)),
    )
}
#[test]
fn test_make_expr_replace_impute_scalar() -> Fallible<()> {
    // when replacing null with "x", output is not nullable
    check_replace(
        lit(NULL),
        lit(0),
        Some((Series::new("".into(), [1, 2, 3]), false)),
    )
}

#[test]
fn test_make_expr_replace_impute_series() -> Fallible<()> {
    // when replacing each of old with respective new, output remains nullable
    check_replace(
        lit(Series::new("".into(), vec![Some(1), None])),
        lit(Series::new("".into(), vec![10, 0])),
        Some((Series::new("".into(), [10, 2, 3]), false)),
    )
}

#[test]
fn test_make_expr_replace_wrong_dtype() -> Fallible<()> {
    check_replace(lit("A"), lit(45), None)
}

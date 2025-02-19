use polars::{
    df,
    lazy::frame::{IntoLazy, LazyFrame},
};
use polars_plan::dsl::col;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, SeriesDomain},
    metrics::SymmetricDistance,
    transformations::make_stable_lazyframe,
};

use super::*;

fn get_alias_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<bool>::default()),
    ])?;
    let ints = [1, 2, 3, 4, 5];
    let bools = [true, false, true, true, false];
    let lf = df!("A" => ints, "B" => bools)?.lazy();
    Ok((lf_domain, lf))
}

macro_rules! test_query {
    ($expr:expr) => {{
        let (lf_domain, lf) = get_alias_data()?;

        let query = lf.clone().with_column($expr);
        let t_lf = make_stable_lazyframe(lf_domain, SymmetricDistance, query)?;

        assert_eq!(
            t_lf.output_domain.schema(),
            t_lf.invoke(&lf)?.collect()?.schema()
        );
        Ok(())
    }};
}

#[test]
fn test_alias_shadow_forward() -> Fallible<()> {
    test_query!(col("A").alias("B"))
}

#[test]
fn test_alias_shadow_backward() -> Fallible<()> {
    test_query!(col("B").alias("A"))
}

#[test]
fn test_alias_new_column() -> Fallible<()> {
    test_query!(col("A").alias("C"))
}

#[test]
fn test_alias_same_column() -> Fallible<()> {
    test_query!(col("A").alias("A"))
}

#[test]
fn test_alias() -> Fallible<()> {
    let ints = [1, 2, 3, 4, 5];
    let bools = [true, false, true, true, false];
    let lf = df!("A" => ints, "B" => bools)?.lazy();

    let expr_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<bool>::default()),
    ])?
    .row_by_row();

    let t_ab = col("A")
        .alias("B")
        .make_stable(expr_domain.clone(), SymmetricDistance)?;
    let t_bc = col("B")
        .alias("C")
        .make_stable(expr_domain, SymmetricDistance)?;

    let expr_ab = t_ab.invoke(&lf.logical_plan)?.expr;
    let expr_bc = t_bc.invoke(&lf.logical_plan)?.expr;

    let actual = lf.with_columns([expr_ab, expr_bc]).collect()?;
    let expect = df!("A" => ints, "B" => ints, "C" => bools)?;
    assert_eq!(actual, expect);

    Ok(())
}

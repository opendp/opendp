use polars::{df, lazy::frame::IntoLazy};
use polars_plan::dsl::{all, col};

use crate::{
    domains::{AtomDomain, LazyFrameDomain, SeriesDomain},
    metrics::SymmetricDistance,
};

use super::*;

#[test]
fn test_behavior() -> Fallible<()> {
    let ints = [1, 2, 3, 4, 5];
    let bools = [true, false, true, true, false];
    let lf = df!("A" => ints, "B" => bools)?.lazy();

    // alias with existing column preserves column order
    let actual = lf.clone().with_column(col("A").alias("B")).collect()?;
    let expect = df!("A" => ints, "B" => ints)?;
    assert_eq!(actual, expect);

    // ...both ways
    let actual = lf.clone().with_column(col("B").alias("A")).collect()?;
    let expect = df!("A" => bools, "B" => bools)?;
    assert_eq!(actual, expect);

    // alias with new column adds to end
    let actual = lf.clone().with_column(col("A").alias("C")).collect()?;
    let expect = df!("A" => ints, "B" => bools, "C" => ints)?;
    assert_eq!(actual, expect);

    // alias with same name maintains position
    let actual = lf.clone().with_column(col("A").alias("A")).collect()?;
    let expect = df!("A" => ints, "B" => bools)?;
    assert_eq!(actual, expect);

    Ok(())
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

    let expr_ab = t_ab.invoke(&(lf.clone().logical_plan, all()))?.1;
    let expr_bc = t_bc.invoke(&(lf.clone().logical_plan, all()))?.1;

    let actual = lf.with_columns([expr_ab, expr_bc]).collect()?;
    let expect = df!("A" => ints, "B" => ints, "C" => bools)?;
    assert_eq!(actual, expect);

    Ok(())
}

use super::*;
use polars::prelude::*;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    error::ErrorVariant,
    measurements::{make_private_expr, make_private_lazyframe, PrivateExpr},
    metrics::{InsertDeleteDistance, PartitionDistance, SymmetricDistance},
    polars::PrivacyNamespace,
    transformations::test_helper::get_test_data,
};

#[test]
fn test_make_expr_puredp() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let scale: f64 = 0.0;

    let m_quant = make_private_expr(
        lf_domain.select(),
        PartitionDistance(InsertDeleteDistance),
        MaxDivergence::default(),
        col("const_1f64").dp().sum((0., 1.), Some(scale)),
        None,
    )?;

    let dp_expr = m_quant.invoke(&lf.logical_plan)?.expr;
    let df_actual = lf.select([dp_expr]).collect()?;

    assert_eq!(df_actual, df!("const_1f64" => [1000.0])?);

    Ok(())
}

#[test]
fn test_make_expr_zcdp() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let scale: f64 = 0.0;

    let m_quant = make_private_expr(
        lf_domain.select(),
        PartitionDistance(InsertDeleteDistance),
        ZeroConcentratedDivergence::default(),
        col("const_1f64").dp().sum((0., 1.), Some(scale)),
        None,
    )?;

    let dp_expr = m_quant.invoke(&lf.logical_plan)?.expr;
    let df_actual = lf.select([dp_expr]).collect()?;

    assert_eq!(df_actual, df!("const_1f64" => [1000.0])?);

    Ok(())
}

#[test]
fn test_fail_make_expr_wrong_distribution() -> Fallible<()> {
    let (lf_domain, _) = get_test_data()?;
    let scale: f64 = 0.0;

    let variant = make_private_expr(
        lf_domain.select(),
        PartitionDistance(InsertDeleteDistance),
        MaxDivergence::default(),
        col("const_1f64")
            .clip(lit(0.), lit(1.))
            .sum()
            .dp()
            .gaussian(Some(scale)),
        None,
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(variant, ErrorVariant::MakeMeasurement);

    Ok(())
}

#[test]
fn test_make_expr_gaussian() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let scale: f64 = 0.0;

    let m_quant = make_private_expr(
        lf_domain.select(),
        PartitionDistance(InsertDeleteDistance),
        ZeroConcentratedDivergence::default(),
        col("const_1f64").dp().sum((0., 1.), Some(scale)),
        None,
    )?;

    let dp_expr = m_quant.invoke(&lf.logical_plan)?.expr;
    let df_actual = lf.select([dp_expr]).collect()?;

    assert_eq!(df_actual, df!("const_1f64" => [1000.0])?);

    Ok(())
}

#[test]
fn test_make_laplace_grouped() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let scale: f64 = 0.0;

    let expr_exp = col("chunk_(..10u32)")
        .clip(lit(0), lit(8))
        .sum()
        .dp()
        .laplace(None);
    let m_lap = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::default(),
        lf.clone().group_by(["chunk_2_bool"]).agg([expr_exp]),
        Some(scale),
        None,
    )?;
    // sum([0, 1, 2, 3, 4]) * 100 = 1000
    // sum([5, 6, 7, 8, 8]) * 100 = 3400

    let df_act = m_lap.invoke(&lf)?.collect()?;
    let df_exp = df!(
        "chunk_2_bool" => [false, true],
        "chunk_(..10u32)" => [1000, 3400]
    )?;

    assert_eq!(
        df_act.sort(["chunk_2_bool"], Default::default())?,
        df_exp.sort(["chunk_2_bool"], Default::default())?
    );
    Ok(())
}

fn check_autocalibration(
    margin: Margin,
    bounds: (u32, u32),
    d_in: (u32, u32, u32),
) -> Fallible<()> {
    let series_domain = SeriesDomain::new("A", AtomDomain::<i32>::default());
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?.with_margin::<&str>(&[], margin)?;
    let expr_domain = lf_domain.select();

    // Get resulting sum (expression result)
    let m_sum = col("A")
        .clip(lit(bounds.0), lit(bounds.1))
        .sum()
        .dp()
        .noise(None, None)
        .make_private(
            expr_domain,
            PartitionDistance(InsertDeleteDistance),
            MaxDivergence,
            Some(1.),
        )?;

    let epsilon = m_sum.map(&d_in)?;
    // autocalibration chooses a noise scale based on the sensitivity.
    // because of this, epsilon will always work out to 1.
    println!("epsilon: {:?}", epsilon);
    assert_eq!(epsilon, 1.0);

    Ok(())
}

#[test]
fn test_sum_unbounded_dp_autocalibration() -> Fallible<()> {
    check_autocalibration(
        Margin::default().with_max_partition_length(100),
        (4, 7),
        (1, 1, 1),
    )
}

#[test]
fn test_sum_bounded_dp_autocalibration() -> Fallible<()> {
    check_autocalibration(
        Margin::default()
            .with_max_partition_length(100)
            .with_public_lengths(),
        (4, 7),
        (1, 2, 2),
    )
}

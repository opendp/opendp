use core::f64;

use polars::{
    df,
    prelude::{IntoLazy, NamedFrom},
    series::Series,
};
use polars_plan::dsl::{col, len};

use crate::{
    accuracy::discrete_laplacian_scale_to_accuracy,
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    error::Fallible,
    measurements::make_private_lazyframe,
    measures::MaxDivergence,
    metrics::SymmetricDistance,
    polars::PrivacyNamespace,
};

use super::summarize_polars_measurement;

#[test]
fn test_summarize_polars_measurement_basic() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin::<&str>(
        &[],
        Margin::default()
            .with_public_keys()
            .with_max_partition_length(10),
    )?;

    let lf = df!("A" => &[3, 4, 5], "B" => &[1., 3., 7.])?.lazy();

    let meas = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::default(),
        lf.select([
            len().dp().noise(None, None),
            col("A").dp().sum((0, 1), None),
        ]),
        Some(1.0),
        None,
    )?;

    let description = summarize_polars_measurement(meas.clone(), None)?;

    let mut expected = df![
        "column" => &["len", "A"],
        "aggregate" => &["Frame Length", "Sum"],
        "distribution" => &["Integer Laplace", "Integer Laplace"],
        "scale" => &[1.0, 1.0]
    ]?;
    println!("{:?}", expected);
    assert_eq!(expected, description);

    let description = summarize_polars_measurement(meas.clone(), Some(0.05))?;

    let accuracy = discrete_laplacian_scale_to_accuracy(1.0, 0.05)?;
    expected.with_column(Series::new("accuracy".into(), &[accuracy, accuracy]))?;
    println!("{:?}", expected);
    assert_eq!(expected, description);

    Ok(())
}

#[test]
fn test_summarize_polars_measurement_mean() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin::<&str>(
        &[],
        Margin::default()
            .with_public_lengths()
            .with_max_partition_length(10),
    )?;

    let lf = df!("A" => &[3, 4, 5], "B" => &[1., 3., 7.])?.lazy();

    let meas = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::default(),
        lf.select([col("A").dp().mean((3, 5), Some((1.0, 0.0)))]),
        None,
        None,
    )?;

    let description = summarize_polars_measurement(meas.clone(), None)?;

    let mut expected = df![
        "column" => &["A", "A"],
        "aggregate" => &["Sum", "Length"],
        "distribution" => &[Some("Integer Laplace"), Some("Integer Laplace")],
        "scale" => &[Some(1.0), Some(0.0)]
    ]?;
    println!("{:?}", expected);
    assert_eq!(expected, description);

    let description = summarize_polars_measurement(meas.clone(), Some(0.05))?;

    let accuracy = discrete_laplacian_scale_to_accuracy(1.0, 0.05)?;
    expected.with_column(Series::new(
        "accuracy".into(),
        &[Some(accuracy), Some(f64::NAN)],
    ))?;
    println!("{:?}", expected);
    assert_eq!(description, expected);

    Ok(())
}

#[test]
fn test_summarize_polars_measurement_quantile() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
            .with_margin::<&str>(
                &[],
                Margin::default()
                    .with_public_lengths()
                    .with_max_partition_length(100),
            )?;

    let lf = df!("A" => (0..=100i32).collect::<Vec<_>>())?.lazy();

    let cands = Series::new(
        "candidates".into(),
        (0..=10).map(|v| v * 10).collect::<Vec<_>>(),
    );
    let meas = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::default(),
        lf.select([
            col("A")
                .dp()
                .quantile(0.25, cands.clone(), Some(1.0))
                .alias("25"),
            col("A")
                .dp()
                .quantile(0.50, cands.clone(), Some(1.0))
                .alias("50"),
            col("A")
                .dp()
                .quantile(0.75, cands.clone(), Some(1.0))
                .alias("75"),
        ]),
        None,
        None,
    )?;

    let description = summarize_polars_measurement(meas.clone(), None)?;

    let expected = df![
        "column" => &["25", "50", "75"],
        "aggregate" => &["0.25-Quantile", "0.5-Quantile", "0.75-Quantile"],
        "distribution" => &[Some("GumbelMin"); 3],
        "scale" => &[Some(1.0); 3]
    ]?;
    println!("{:?}", expected);
    assert_eq!(expected, description);

    Ok(())
}

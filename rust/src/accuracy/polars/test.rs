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

use super::describe_polars_measurement_accuracy;

#[test]
fn test_describe_polars_measurement_accuracy() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin::<&str>(
        &[],
        Margin::new()
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

    let description = describe_polars_measurement_accuracy(meas.clone(), None)?;

    let mut expected = df![
        "column" => &["len", "A"],
        "aggregate" => &["Len", "Sum"],
        "distribution" => &["Integer Laplace", "Integer Laplace"],
        "scale" => &[1.0, 1.0]
    ]?;
    println!("{:?}", expected);
    assert_eq!(expected, description);

    let description = describe_polars_measurement_accuracy(meas.clone(), Some(0.05))?;

    let accuracy = discrete_laplacian_scale_to_accuracy(1.0, 0.05)?;
    expected.with_column(Series::new("accuracy", &[accuracy, accuracy]))?;
    println!("{:?}", expected);
    assert_eq!(expected, description);

    Ok(())
}

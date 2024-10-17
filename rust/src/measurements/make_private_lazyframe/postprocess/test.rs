use polars::{df, lazy::frame::IntoLazy};
use polars_plan::dsl::col;

use crate::{
    domains::{AtomDomain, Margin, SeriesDomain},
    measurements::make_private_lazyframe,
    measures::MaxDivergence,
    metrics::SymmetricDistance,
    polars::PrivacyNamespace,
};

use super::*;

#[test]
fn test_make_private_lazyframe_sort() -> Fallible<()> {
    let s1 = SeriesDomain::new("A", AtomDomain::new_closed((0, 1))?);
    let s2 = SeriesDomain::new("B", AtomDomain::<bool>::default());

    let lf_domain = LazyFrameDomain::new(vec![s1, s2])?.with_margin(
        &["B"],
        Margin::default()
            .with_public_lengths()
            .with_max_partition_length(5),
    )?;
    let lf = df!("A" => [0, 1, 1], "B" => [true, false, true])?.lazy();

    let query = lf
        .group_by([col("B")])
        .agg([col("A").dp().sum((0, 1), Some(1.))])
        .sort(["len"], Default::default());

    make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::default(),
        query,
        None,
        None,
    )?;

    Ok(())
}

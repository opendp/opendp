use crate::{
    domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain},
    metrics::{Binding, FrameDistance, SymmetricDistance, SymmetricIdDistance},
    transformations::make_stable_lazyframe,
};

use super::*;

#[test]
fn test_select_microdata() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?;

    let t_select = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().select([lit(2).gt(col("chunk_2_null"))]),
    )?;

    let observed = t_select.invoke(&lf)?.collect()?;
    let expected = df!(
        "chunk_2_null" => [Some(1), None],
    )?;
    assert_eq!(observed, expected);
    assert!(t_select.output_domain.margins.is_empty());

    Ok(())
}

#[test]
fn test_select_rejects_multisite_identifier_modification() -> Fallible<()> {
    let lf = df!(
        "user_id" => [1i32, 2],
        "edge_src" => [10i32, 11],
        "value" => [5i32, 6]
    )?
    .lazy();

    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("edge_src", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;

    let metric = SymmetricIdDistance {
        protect: "user".to_string(),
        bindings: vec![
            Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            },
            Binding {
                space: "node".to_string(),
                exprs: vec![col("edge_src")],
            },
        ],
    };

    let err = make_stable_lazyframe::<_, FrameDistance<SymmetricIdDistance>>(
        lf_domain,
        FrameDistance(metric),
        lf.select([col("value"), col("user_id").alias("user_id")]),
    )
    .unwrap_err();

    assert!(format!("{err:?}").contains("identifiers"));
    Ok(())
}

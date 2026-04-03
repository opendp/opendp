use super::*;
use crate::{
    domains::{AtomDomain, LazyFrameDomain, SeriesDomain},
    metrics::{Binding, FrameDistance, SymmetricIdDistance},
    transformations::make_stable_lazyframe,
};

#[test]
fn test_check_infallible() -> Fallible<()> {
    // failure to cast causes a data-dependent error
    assert!(check_infallible(&lit("a").strict_cast(DataType::Int32), Resize::Allow).is_err());

    Ok(())
}

#[test]
fn test_check_infallible_resize() -> Fallible<()> {
    // col doesn't resize, so passes the ban
    assert!(check_infallible(&col("A"), Resize::Ban).is_ok());
    // sum results in a broadcastable scalar, so it passes the ban
    assert!(check_infallible(&col("A").sum(), Resize::Ban).is_ok());
    // unique resizes, so fails the ban
    assert!(check_infallible(&col("A").unique(), Resize::Ban).is_err());
    // resizing behind an aggregation is allowed, though, because it can broadcast
    assert!(check_infallible(&col("A").unique().sum(), Resize::Ban).is_ok());
    // while resizing is generally allowed, binary ops ban resizing
    assert!(check_infallible(&(col("A").unique() + col("B")), Resize::Allow).is_err());
    // the sum results in a broadcastable scalar, so it passes the binary op resize ban
    assert!(check_infallible(&(col("A").unique().sum() + col("B")), Resize::Ban).is_ok());

    Ok(())
}

#[test]
fn test_group_by_rebuilds_singleton_owner_claim_from_keys() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let metric = FrameDistance(SymmetricIdDistance {
        protect: "user".to_string(),
        bindings: vec![Binding {
            space: "user".to_string(),
            exprs: vec![col("user_id")],
        }],
        owner_claims: vec![vec![col("user_id")]],
    });

    let plan = df!("user_id" => [1i32, 1, 2], "value" => [10i32, 11, 12])?
        .lazy()
        .group_by([col("user_id")])
        .agg([col("value").sum()])
        .logical_plan;

    let t_group = make_stable_lazyframe::<_, FrameDistance<SymmetricIdDistance>>(
        domain,
        metric,
        LazyFrame::from(plan),
    )?;

    assert_eq!(t_group.output_metric.0.owner_claims, vec![vec![col("user_id")]]);
    assert_eq!(t_group.map(&crate::metrics::Bounds::from(1))?, crate::metrics::Bounds::from(1));
    Ok(())
}

#[test]
fn test_group_by_rejects_when_keys_drop_owner_factor() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("src_user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("dst_user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let metric = FrameDistance(SymmetricIdDistance {
        protect: "user".to_string(),
        bindings: vec![
            Binding {
                space: "user".to_string(),
                exprs: vec![col("src_user_id")],
            },
            Binding {
                space: "user".to_string(),
                exprs: vec![col("dst_user_id")],
            },
        ],
        owner_claims: vec![vec![col("src_user_id"), col("dst_user_id")]],
    });

    let plan = df!(
        "src_user_id" => [1i32, 1, 2],
        "dst_user_id" => [2i32, 3, 3],
        "value" => [10i32, 11, 12]
    )?
    .lazy()
    .group_by([col("src_user_id")])
    .agg([col("value").sum()])
    .logical_plan;

    let err = make_stable_lazyframe::<_, FrameDistance<SymmetricIdDistance>>(
        domain,
        metric,
        LazyFrame::from(plan),
    )
    .unwrap_err();
    assert!(
        err.message
            .unwrap_or_default()
            .contains("complete protected owner claim")
    );
    Ok(())
}

#[test]
fn test_group_by_rebuilds_singleton_owner_claim_from_exact_one_derived_key() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("merchant_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let metric = FrameDistance(SymmetricIdDistance {
        protect: "user".to_string(),
        bindings: vec![
            Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            },
            Binding {
                space: "user".to_string(),
                exprs: vec![col("merchant_id")],
            },
        ],
        owner_claims: vec![vec![col("user_id")]],
    });

    let plan = df!(
        "user_id" => [1i32, 1, 2],
        "merchant_id" => [7i32, 7, 8],
        "value" => [10i32, 11, 12]
    )?
    .lazy()
    .group_by([col("merchant_id")])
    .agg([col("value").sum()])
    .logical_plan;

    let t_group = make_stable_lazyframe::<_, FrameDistance<SymmetricIdDistance>>(
        domain,
        metric,
        LazyFrame::from(plan),
    )?;

    assert_eq!(t_group.output_metric.0.owner_claims, vec![vec![col("merchant_id")]]);
    Ok(())
}

#[test]
fn test_group_by_currently_rejects_multi_owner_rebuild_from_derived_keys() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("src_user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("dst_user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("src_merchant_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("dst_merchant_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let metric = FrameDistance(SymmetricIdDistance {
        protect: "user".to_string(),
        bindings: vec![
            Binding {
                space: "user".to_string(),
                exprs: vec![col("src_user_id"), col("src_merchant_id")],
            },
            Binding {
                space: "user".to_string(),
                exprs: vec![col("dst_user_id"), col("dst_merchant_id")],
            },
        ],
        owner_claims: vec![vec![col("src_user_id"), col("dst_user_id")]],
    });

    let plan = df!(
        "src_user_id" => [1i32, 1, 2],
        "dst_user_id" => [2i32, 3, 3],
        "src_merchant_id" => [10i32, 10, 20],
        "dst_merchant_id" => [30i32, 40, 40],
        "value" => [10i32, 11, 12]
    )?
    .lazy()
    .group_by([col("src_merchant_id"), col("dst_merchant_id")])
    .agg([col("value").sum()])
    .logical_plan;

    let err = make_stable_lazyframe::<_, FrameDistance<SymmetricIdDistance>>(
        domain,
        metric,
        LazyFrame::from(plan),
    )
    .unwrap_err();
    assert!(
        err.message
            .unwrap_or_default()
            .contains("complete protected owner claim")
    );
    Ok(())
}

use std::collections::HashMap;

use crate::{
    domains::{AtomDomain, DatabaseDomain, LazyFrameDomain, SeriesDomain},
    metrics::{Bound, Bounds, DatabaseIdDistance, IdSite, PolarsMetric},
};
use polars::prelude::*;

use super::*;

#[test]
fn test_database_source_stability_map_for_private_table() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("age", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("events".to_string(), events_domain),
        ("users".to_string(), users_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protected_label: "user".to_string(),
        id_sites: HashMap::from([(
            "events".to_string(),
            vec![IdSite {
                label: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
    };

    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events", "events"],
        "user_id" => [1i32, 2],
        "value" => [10i32, 11]
    )?
    .lazy();

    let t_source =
        make_stable_database_source(database_domain, database_metric, events.logical_plan)?;

    assert_eq!(
        t_source.map(&2)?,
        Bounds::from(2).with_bound(
            Bound::by([col("user_id")])
                .with_num_groups(2)
                .with_per_group(1),
        )
    );
    Ok(())
}

#[test]
fn test_database_source_stability_map_for_public_table_is_identity() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("age", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("events".to_string(), events_domain),
        ("users".to_string(), users_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protected_label: "user".to_string(),
        id_sites: HashMap::from([(
            "events".to_string(),
            vec![IdSite {
                label: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
    };

    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users", "users"],
        "user_id" => [1i32, 2],
        "age" => [30i32, 31]
    )?
    .lazy();

    let t_source =
        make_stable_database_source(database_domain, database_metric, users.logical_plan)?;

    assert!(t_source.output_metric.0.active_id_sites().is_empty());
    assert_eq!(t_source.map(&3)?, Bounds::from(3));
    Ok(())
}

#[test]
fn test_database_source_preserves_non_protected_sites_but_only_activates_protected_label() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("household_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([(
        "events".to_string(),
        events_domain,
    )]));
    let database_metric = DatabaseIdDistance {
        protected_label: "user".to_string(),
        id_sites: HashMap::from([(
            "events".to_string(),
            vec![
                IdSite {
                    label: "user".to_string(),
                    exprs: vec![col("user_id")],
                },
                IdSite {
                    label: "household".to_string(),
                    exprs: vec![col("household_id")],
                },
            ],
        )]),
    };

    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events", "events"],
        "user_id" => [1i32, 2],
        "household_id" => [10i32, 11],
        "value" => [10i32, 11]
    )?
    .lazy();

    let t_source =
        make_stable_database_source(database_domain, database_metric, events.logical_plan)?;

    assert_eq!(t_source.output_metric.0.id_sites.len(), 2);
    assert_eq!(t_source.output_metric.0.active_id_sites().len(), 1);
    assert_eq!(
        t_source.output_metric.0.active_id_sites()[0].exprs,
        vec![col("user_id")]
    );
    Ok(())
}

#[test]
fn test_database_source_requires_marker_for_multitable_scan() -> Fallible<()> {
    let database_domain = DatabaseDomain::new(HashMap::from([
        (
            "events".to_string(),
            LazyFrameDomain::new(vec![
                SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
                SeriesDomain::new("value", AtomDomain::<i32>::default()),
            ])?,
        ),
        (
            "users".to_string(),
            LazyFrameDomain::new(vec![
                SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
                SeriesDomain::new("age", AtomDomain::<i32>::default()),
            ])?,
        ),
    ]));
    let database_metric = DatabaseIdDistance {
        protected_label: "user".to_string(),
        id_sites: HashMap::new(),
    };

    let unmarked_scan = df!("user_id" => [1i32], "value" => [10i32])?.lazy().logical_plan;
    let err = make_stable_database_source(database_domain, database_metric, unmarked_scan)
        .unwrap_err();
    assert!(
        err.message
            .unwrap_or_default()
            .contains("expected exactly one source table marker column")
    );
    Ok(())
}

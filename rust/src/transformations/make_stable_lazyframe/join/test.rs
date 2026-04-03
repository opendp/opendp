use std::collections::HashMap;

use crate::{
    domains::{AtomDomain, DatabaseDomain, LazyFrameDomain, Margin, SeriesDomain},
    metrics::{DatabaseIdDistance, IdSite},
    transformations::make_stable_database_lazyframe,
};
use polars::prelude::*;

use super::*;

#[test]
fn test_database_join_augments_id_sites() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("id", AtomDomain::<i32>::default()),
        SeriesDomain::new("age", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["id"]).with_max_length(1))?;

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
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users", "users"],
        "id" => [1i32, 2],
        "age" => [30i32, 31]
    )?
    .lazy();

    let plan = events.join(users, [col("user_id")], [col("id")], JoinType::Left.into());

    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.id_sites.len(), 1);
    assert_eq!(
        t_join.output_metric.0.id_sites[0].exprs,
        vec![col("user_id"), col("id")]
    );

    Ok(())
}

#[test]
fn test_database_join_allows_non_unique_public_keys() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("id", AtomDomain::<i32>::default()),
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
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "user_id" => [1i32],
        "value" => [10i32]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users", "users"],
        "id" => [1i32, 1],
        "age" => [30i32, 31]
    )?
    .lazy();

    let plan = events.join(users, [col("user_id")], [col("id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.id_sites.len(), 1);
    assert_eq!(
        t_join.output_metric.0.id_sites[0].exprs,
        vec![col("user_id"), col("id")]
    );
    Ok(())
}

#[test]
fn test_database_join_drops_output_margins() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["user_id"]).with_max_length(2))?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("id", AtomDomain::<i32>::default()),
        SeriesDomain::new("age", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["id"]).with_max_length(1))?;

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
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users", "users"],
        "id" => [1i32, 2],
        "age" => [30i32, 31]
    )?
    .lazy();

    let plan = events.join(users, [col("user_id")], [col("id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert!(t_join.output_domain.margins.is_empty());
    Ok(())
}

#[test]
fn test_database_join_allows_private_private_join_on_protected_identifier() -> Fallible<()> {
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
        id_sites: HashMap::from([
            (
                "events".to_string(),
                vec![IdSite {
                    label: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "users".to_string(),
                vec![IdSite {
                    label: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
        ]),
    };

    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "user_id" => [1i32],
        "value" => [10i32]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users"],
        "user_id" => [1i32],
        "age" => [30i32]
    )?
    .lazy();

    let plan = events.join(users, [col("user_id")], [col("user_id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.active_id_sites().len(), 1);
    assert_eq!(t_join.output_metric.0.active_id_sites()[0].exprs, vec![col("user_id")]);
    Ok(())
}

#[test]
fn test_database_join_rejects_private_private_join_on_non_identifier_key() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("events".to_string(), events_domain),
        ("users".to_string(), users_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protected_label: "user".to_string(),
        id_sites: HashMap::from([
            (
                "events".to_string(),
                vec![IdSite {
                    label: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "users".to_string(),
                vec![IdSite {
                    label: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
        ]),
    };

    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "user_id" => [1i32],
        "value" => [10i32]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users"],
        "user_id" => [1i32],
        "value" => [10i32]
    )?
    .lazy();

    let plan = events.join(users, [col("value")], [col("value")], JoinType::Left.into());
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(
        err.message
            .unwrap_or_default()
            .contains("active protected identifier")
    );
    Ok(())
}

#[test]
fn test_database_join_after_truncation_is_rejected() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("age", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["user_id"]).with_max_length(1))?;

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
        "__OPENDP_TABLE_NAME__[events]" => ["events", "events", "events"],
        "user_id" => [1i32, 1, 2],
        "value" => [10i32, 11, 12]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users", "users"],
        "user_id" => [1i32, 2],
        "age" => [30i32, 31]
    )?
    .lazy();

    let truncation = int_range(lit(0), len(), 1, DataType::Int64)
        .over([col("user_id")])
        .lt(lit(1u32));
    let plan = events
        .filter(truncation)
        .join(users, [col("user_id")], [col("user_id")], JoinType::Left.into());
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(
        err.message
            .unwrap_or_default()
            .contains("joins are only supported before truncation")
    );
    Ok(())
}

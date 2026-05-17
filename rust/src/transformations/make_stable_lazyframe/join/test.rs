use std::collections::HashMap;

use crate::{
    domains::{AtomDomain, DatabaseDomain, LazyFrameDomain, Margin, SeriesDomain},
    metrics::{Binding, DatabaseIdDistance},
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
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
        base_owner_claims: HashMap::from([("events".to_string(), vec![vec![col("user_id")]])]),
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

    assert_eq!(t_join.output_metric.0.bindings.len(), 1);
    assert_eq!(t_join.output_metric.0.bindings[0].exprs, vec![col("user_id")]);

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
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
        base_owner_claims: HashMap::from([("events".to_string(), vec![vec![col("user_id")]])]),
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

    assert_eq!(t_join.output_metric.0.bindings.len(), 1);
    assert_eq!(t_join.output_metric.0.bindings[0].exprs, vec![col("user_id")]);
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
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
        base_owner_claims: HashMap::from([("events".to_string(), vec![vec![col("user_id")]])]),
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
fn test_database_join_output_domain_excludes_table_markers() -> Fallible<()> {
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
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
        base_owner_claims: HashMap::from([("events".to_string(), vec![vec![col("user_id")]])]),
    };

    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "user_id" => [1i32],
        "value" => [10i32]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users"],
        "id" => [1i32],
        "age" => [30i32]
    )?
    .lazy();

    let plan = events.join(users, [col("user_id")], [col("id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    let schema = t_join.output_domain.schema();
    let observed = schema
        .iter_names()
        .map(|name| name.as_str())
        .collect::<Vec<_>>();
    assert!(!observed
        .iter()
        .any(|name| name.starts_with("__OPENDP_TABLE_NAME__")));
    assert!(observed.contains(&"user_id"));
    assert!(observed.contains(&"value"));
    assert!(observed.contains(&"age"));
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
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "events".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "users".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
        ]),
        base_owner_claims: HashMap::from([
            ("events".to_string(), vec![vec![col("user_id")]]),
            ("users".to_string(), vec![vec![col("user_id")]]),
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

    let plan = events.join(
        users,
        [col("user_id")],
        [col("user_id")],
        JoinType::Left.into(),
    );
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.owner_claims.len(), 1);
    assert_eq!(t_join.output_metric.0.owner_claims[0], vec![col("user_id")]);
    Ok(())
}

#[test]
fn test_database_join_deduplicates_equivalent_protected_owners() -> Fallible<()> {
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
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "events".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "users".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("id")],
                }],
            ),
        ]),
        base_owner_claims: HashMap::from([
            ("events".to_string(), vec![vec![col("user_id")]]),
            ("users".to_string(), vec![vec![col("id")]]),
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
        "id" => [1i32],
        "age" => [30i32]
    )?
    .lazy();

    let plan = events.join(users, [col("user_id")], [col("id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.owner_claims.len(), 1);
    assert_eq!(t_join.output_metric.0.owner_claims[0].len(), 1);
    Ok(())
}

#[test]
fn test_database_join_private_public_preserves_private_owner_claim() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
    ])?;
    let products_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("price", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("events".to_string(), events_domain),
        ("products".to_string(), products_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
        base_owner_claims: HashMap::from([
            ("events".to_string(), vec![vec![col("user_id")]]),
            ("products".to_string(), vec![vec![]]),
        ]),
    };

    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "user_id" => [1i32],
        "product_id" => [10i32]
    )?
    .lazy();
    let products = df!(
        "__OPENDP_TABLE_NAME__[products]" => ["products"],
        "product_id" => [10i32],
        "price" => [99i32]
    )?
    .lazy();

    let plan = events.join(products, [col("product_id")], [col("product_id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.owner_claims, vec![vec![col("user_id")]]);
    Ok(())
}

#[test]
fn test_database_join_allows_public_lookup_with_protected_binding_but_empty_claim() -> Fallible<()> {
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
        ("users_metadata".to_string(), users_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "events".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "users_metadata".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
        ]),
        base_owner_claims: HashMap::from([
            ("events".to_string(), vec![vec![col("user_id")]]),
            ("users_metadata".to_string(), vec![vec![]]),
        ]),
    };

    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "user_id" => [1i32],
        "value" => [10i32]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users_metadata]" => ["users_metadata"],
        "user_id" => [1i32],
        "age" => [30i32]
    )?
    .lazy();

    let plan = events.join(users, [col("user_id")], [col("user_id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.owner_claims, vec![vec![col("user_id")]]);
    Ok(())
}

#[test]
fn test_database_join_rewrites_same_named_right_owner_column_to_output_suffix() -> Fallible<()> {
    let products_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
    ])?;
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("products".to_string(), products_domain),
        ("events".to_string(), events_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
        base_owner_claims: HashMap::from([
            ("products".to_string(), vec![vec![]]),
            ("events".to_string(), vec![vec![col("user_id")]]),
        ]),
    };

    let products = df!(
        "__OPENDP_TABLE_NAME__[products]" => ["products"],
        "product_id" => [10i32],
        "user_id" => [999i32]
    )?
    .lazy();
    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "product_id" => [10i32],
        "user_id" => [1i32]
    )?
    .lazy();

    let plan = products.join(events, [col("product_id")], [col("product_id")], JoinType::Left.into());
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(
        t_join.output_metric.0.owner_claims,
        vec![vec![col("user_id_right")]]
    );
    Ok(())
}

#[test]
fn test_database_join_rejects_structured_right_owner_factor_under_suffix_collision() -> Fallible<()> {
    let products_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
    ])?;
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
    ])?;

    let owner_expr = col("user_id").cast(DataType::Int64);
    let database_domain = DatabaseDomain::new(HashMap::from([
        ("products".to_string(), products_domain),
        ("events".to_string(), events_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![owner_expr.clone()],
            }],
        )]),
        base_owner_claims: HashMap::from([
            ("products".to_string(), vec![vec![]]),
            ("events".to_string(), vec![vec![owner_expr]]),
        ]),
    };

    let products = df!(
        "__OPENDP_TABLE_NAME__[products]" => ["products"],
        "product_id" => [10i32],
        "user_id" => [999i32]
    )?
    .lazy();
    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "product_id" => [10i32],
        "user_id" => [1i32]
    )?
    .lazy();

    let plan = products.join(events, [col("product_id")], [col("product_id")], JoinType::Left.into());
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(err
        .message
        .unwrap_or_default()
        .contains("could not safely rewrite structured right-branch expression into join output"));
    Ok(())
}

#[test]
fn test_database_join_rejects_structured_right_binding_expression_under_collision() -> Fallible<()> {
    let products_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
    ])?;
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_key", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("products".to_string(), products_domain),
        ("events".to_string(), events_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id").cast(DataType::Int64), col("user_key")],
            }],
        )]),
        base_owner_claims: HashMap::from([
            ("products".to_string(), vec![vec![]]),
            ("events".to_string(), vec![vec![col("user_key")]]),
        ]),
    };

    let products = df!(
        "__OPENDP_TABLE_NAME__[products]" => ["products"],
        "product_id" => [10i32],
        "user_id" => [999i32]
    )?
    .lazy();
    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "product_id" => [10i32],
        "user_id" => [1i32],
        "user_key" => [1i32]
    )?
    .lazy();

    let plan = products.join(events, [col("product_id")], [col("product_id")], JoinType::Left.into());
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(err
        .message
        .unwrap_or_default()
        .contains("could not safely rewrite structured right-branch expression into join output"));
    Ok(())
}

#[test]
fn test_database_join_rejects_structured_owner_factor_transport_without_collision() -> Fallible<()> {
    let products_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("product_id", AtomDomain::<i32>::default())])?;
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("product_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
    ])?;

    let owner_expr = col("user_id").cast(DataType::Int64);
    let database_domain = DatabaseDomain::new(HashMap::from([
        ("products".to_string(), products_domain),
        ("events".to_string(), events_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![owner_expr.clone()],
            }],
        )]),
        base_owner_claims: HashMap::from([
            ("products".to_string(), vec![vec![]]),
            ("events".to_string(), vec![vec![owner_expr]]),
        ]),
    };

    let products = df!(
        "__OPENDP_TABLE_NAME__[products]" => ["products"],
        "product_id" => [10i32]
    )?
    .lazy();
    let events = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events"],
        "product_id" => [10i32],
        "user_id" => [1i32]
    )?
    .lazy();

    let plan =
        products.join(events, [col("product_id")], [col("product_id")], JoinType::Left.into());
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(err
        .message
        .unwrap_or_default()
        .contains("owner claim factors transported through joins must resolve to simple output columns"));
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
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "events".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "users".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
        ]),
        base_owner_claims: HashMap::from([
            ("events".to_string(), vec![vec![col("user_id")]]),
            ("users".to_string(), vec![vec![col("user_id")]]),
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
            .contains("join must align the active owner claim")
    );
    Ok(())
}

#[test]
fn test_database_join_many_to_many_on_protected_key_remains_singleton_claim() -> Fallible<()> {
    let clicks_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("clicks", AtomDomain::<i32>::default()),
    ])?;
    let purchases_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("amount", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("clicks".to_string(), clicks_domain),
        ("purchases".to_string(), purchases_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "clicks".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "purchases".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
        ]),
        base_owner_claims: HashMap::from([
            ("clicks".to_string(), vec![vec![col("user_id")]]),
            ("purchases".to_string(), vec![vec![col("user_id")]]),
        ]),
    };

    let clicks = df!(
        "__OPENDP_TABLE_NAME__[clicks]" => ["clicks", "clicks"],
        "user_id" => [1i32, 1],
        "clicks" => [10i32, 11]
    )?
    .lazy();
    let purchases = df!(
        "__OPENDP_TABLE_NAME__[purchases]" => ["purchases", "purchases"],
        "user_id" => [1i32, 1],
        "amount" => [20i32, 21]
    )?
    .lazy();

    let plan = clicks.join(
        purchases,
        [col("user_id")],
        [col("user_id")],
        JoinType::Inner.into(),
    );
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.owner_claims.len(), 1);
    assert_eq!(t_join.output_metric.0.owner_claims[0].len(), 1);
    Ok(())
}

#[test]
fn test_database_join_rewrites_right_owner_aliases_into_output_frame() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;
    let users_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("id", AtomDomain::<i32>::default()),
        SeriesDomain::new("age", AtomDomain::<i32>::default()),
    ])?;
    let audits_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("owner_user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("flag", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("events".to_string(), events_domain),
        ("users".to_string(), users_domain),
        ("audits".to_string(), audits_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "events".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                }],
            ),
            (
                "users".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("id")],
                }],
            ),
            (
                "audits".to_string(),
                vec![Binding {
                    space: "user".to_string(),
                    exprs: vec![col("owner_user_id")],
                }],
            ),
        ]),
        base_owner_claims: HashMap::from([
            ("events".to_string(), vec![vec![col("user_id")]]),
            ("users".to_string(), vec![vec![col("id")]]),
            ("audits".to_string(), vec![vec![col("owner_user_id")]]),
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
        "id" => [1i32],
        "age" => [30i32]
    )?
    .lazy();
    let audits = df!(
        "__OPENDP_TABLE_NAME__[audits]" => ["audits"],
        "owner_user_id" => [1i32],
        "flag" => [7i32]
    )?
    .lazy();

    let plan = events
        .join(users, [col("user_id")], [col("id")], JoinType::Inner.into())
        .join(
            audits,
            [col("user_id")],
            [col("owner_user_id")],
            JoinType::Inner.into(),
        );
    let t_join = make_stable_database_lazyframe(database_domain, database_metric, plan)?;

    assert_eq!(t_join.output_metric.0.bindings.len(), 1);
    assert_eq!(t_join.output_metric.0.bindings[0].exprs, vec![col("user_id")]);
    assert_eq!(t_join.output_metric.0.owner_claims, vec![vec![col("user_id")]]);
    Ok(())
}

#[test]
fn test_database_join_rejects_non_owner_protected_identifier_alignment() -> Fallible<()> {
    let users_left_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("id", AtomDomain::<i32>::default()),
        SeriesDomain::new("referred_by_user_id", AtomDomain::<i32>::default()),
    ])?;
    let users_right_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("id_right", AtomDomain::<i32>::default()),
        SeriesDomain::new("referred_by_user_id_right", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("users_left".to_string(), users_left_domain),
        ("users_right".to_string(), users_right_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "users_left".to_string(),
                vec![
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("id")],
                    },
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("referred_by_user_id")],
                    },
                ],
            ),
            (
                "users_right".to_string(),
                vec![
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("id_right")],
                    },
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("referred_by_user_id_right")],
                    },
                ],
            ),
        ]),
        base_owner_claims: HashMap::from([
            ("users_left".to_string(), vec![vec![col("id")]]),
            ("users_right".to_string(), vec![vec![col("id_right")]]),
        ]),
    };

    let users_left = df!(
        "__OPENDP_TABLE_NAME__[users_left]" => ["users_left"],
        "id" => [1i32],
        "referred_by_user_id" => [7i32]
    )?
    .lazy();
    let users_right = df!(
        "__OPENDP_TABLE_NAME__[users_right]" => ["users_right"],
        "id_right" => [2i32],
        "referred_by_user_id_right" => [7i32]
    )?
    .lazy();

    let plan = users_left.join(
        users_right,
        [col("referred_by_user_id")],
        [col("referred_by_user_id_right")],
        JoinType::Inner.into(),
    );
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(
        err.message
            .unwrap_or_default()
            .contains("join must align the active owner claim")
    );
    Ok(())
}

#[test]
fn test_database_join_rejects_partial_multi_owner_alignment() -> Fallible<()> {
    let left_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("src_user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("dst_user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("topic", AtomDomain::<i32>::default()),
    ])?;
    let right_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("src_user_id_right", AtomDomain::<i32>::default()),
        SeriesDomain::new("dst_user_id_right", AtomDomain::<i32>::default()),
        SeriesDomain::new("topic_right", AtomDomain::<i32>::default()),
    ])?;

    let database_domain = DatabaseDomain::new(HashMap::from([
        ("left".to_string(), left_domain),
        ("right".to_string(), right_domain),
    ]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([
            (
                "left".to_string(),
                vec![
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("src_user_id")],
                    },
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("dst_user_id")],
                    },
                ],
            ),
            (
                "right".to_string(),
                vec![
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("src_user_id_right")],
                    },
                    Binding {
                        space: "user".to_string(),
                        exprs: vec![col("dst_user_id_right")],
                    },
                ],
            ),
        ]),
        base_owner_claims: HashMap::from([
            (
                "left".to_string(),
                vec![vec![col("src_user_id"), col("dst_user_id")]],
            ),
            (
                "right".to_string(),
                vec![vec![col("src_user_id_right"), col("dst_user_id_right")]],
            ),
        ]),
    };

    let left = df!(
        "__OPENDP_TABLE_NAME__[left]" => ["left"],
        "src_user_id" => [1i32],
        "dst_user_id" => [2i32],
        "topic" => [9i32]
    )?
    .lazy();
    let right = df!(
        "__OPENDP_TABLE_NAME__[right]" => ["right"],
        "src_user_id_right" => [1i32],
        "dst_user_id_right" => [3i32],
        "topic_right" => [9i32]
    )?
    .lazy();

    let plan = left.join(
        right,
        [col("src_user_id")],
        [col("src_user_id_right")],
        JoinType::Inner.into(),
    );
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(
        err.message
            .unwrap_or_default()
            .contains("join must align the active owner claim")
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
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("user_id")],
            }],
        )]),
        base_owner_claims: HashMap::from([("events".to_string(), vec![vec![col("user_id")]])]),
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
    let plan = events.filter(truncation).join(
        users,
        [col("user_id")],
        [col("user_id")],
        JoinType::Left.into(),
    );
    let err = make_stable_database_lazyframe(database_domain, database_metric, plan).unwrap_err();

    assert!(
        err.message
            .unwrap_or_default()
            .contains("joins are only supported before truncation")
    );
    Ok(())
}

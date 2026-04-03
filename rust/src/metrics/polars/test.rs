use std::collections::HashMap;

use polars::lazy::dsl::{as_struct, col};

use super::{
    choose_owner_claim, compile_database_id_distance, expr_identifies_protected_id,
    normalize_claim_with_bindings, normalize_claims_with_bindings, Binding, ForeignKey,
    FunctionalDependency, Ownership, UniqueKey,
};
use crate::error::Fallible;

#[test]
fn test_compile_database_id_distance_propagates_across_foreign_keys() -> Fallible<()> {
    let metric = compile_database_id_distance(
        "user".to_string(),
        HashMap::from([(
            "users".to_string(),
            vec![Binding {
                exprs: vec![col("user_id")],
                space: "user".to_string(),
            }],
        )]),
        vec![UniqueKey {
            table: "users".to_string(),
            key: col("user_id"),
        }],
        vec![ForeignKey {
            from_table: "events".to_string(),
            from: col("user_id"),
            to_table: "users".to_string(),
            to: col("user_id"),
        }],
        vec![],
        vec![],
    )?;

    let events = metric.bindings.get("events").unwrap();
    assert!(expr_identifies_protected_id(events, "user", &col("user_id")));
    assert_eq!(metric.base_owner_claims["events"], vec![vec![col("user_id")]]);
    Ok(())
}

#[test]
fn test_compile_database_id_distance_propagates_across_functional_dependencies() -> Fallible<()> {
    let metric = compile_database_id_distance(
        "user".to_string(),
        HashMap::from([(
            "users".to_string(),
            vec![Binding {
                exprs: vec![col("user_id")],
                space: "user".to_string(),
            }],
        )]),
        vec![],
        vec![],
        vec![FunctionalDependency {
            table: "users".to_string(),
            from: col("merchant_id"),
            to: col("user_id"),
        }],
        vec![],
    )?;

    let users = metric.bindings.get("users").unwrap();
    assert!(expr_identifies_protected_id(users, "user", &col("merchant_id")));
    Ok(())
}

#[test]
fn test_compile_database_id_distance_accepts_explicit_owner_declarations() -> Fallible<()> {
    let metric = compile_database_id_distance(
        "user".to_string(),
        HashMap::from([(
            "users".to_string(),
            vec![Binding {
                exprs: vec![col("user_id")],
                space: "user".to_string(),
            }],
        )]),
        vec![],
        vec![],
        vec![],
        vec![Ownership {
            table: "users".to_string(),
            claims: vec![vec![col("user_id")], vec![]],
        }],
    )?;

    assert_eq!(metric.base_owner_claims["users"], vec![vec![col("user_id")], vec![]]);
    Ok(())
}

#[test]
fn test_compile_database_id_distance_rejects_invalid_owner_declarations() {
    let err = compile_database_id_distance(
        "user".to_string(),
        HashMap::from([(
            "users".to_string(),
            vec![Binding {
                exprs: vec![col("user_id")],
                space: "user".to_string(),
            }],
        )]),
        vec![],
        vec![],
        vec![],
        vec![Ownership {
            table: "users".to_string(),
            claims: vec![vec![col("household_id")]],
        }],
    )
    .unwrap_err();

    assert!(format!("{err:?}").contains("ownership declaration"));
}

#[test]
fn test_compile_database_id_distance_supports_composite_metadata_keys() -> Fallible<()> {
    let user_key = as_struct(vec![col("country"), col("user_num")]);
    let event_key = as_struct(vec![col("country"), col("user_num")]);
    let metric = compile_database_id_distance(
        "user".to_string(),
        HashMap::from([(
            "users".to_string(),
            vec![Binding {
                exprs: vec![user_key.clone()],
                space: "user".to_string(),
            }],
        )]),
        vec![UniqueKey {
            table: "users".to_string(),
            key: user_key.clone(),
        }],
        vec![ForeignKey {
            from_table: "events".to_string(),
            from: event_key.clone(),
            to_table: "users".to_string(),
            to: user_key,
        }],
        vec![],
        vec![],
    )?;

    let events = metric.bindings.get("events").unwrap();
    assert!(expr_identifies_protected_id(events, "user", &event_key));
    Ok(())
}

#[test]
fn test_compile_database_id_distance_propagates_across_transitive_foreign_keys() -> Fallible<()> {
    let metric = compile_database_id_distance(
        "household".to_string(),
        HashMap::from([(
            "households".to_string(),
            vec![Binding {
                exprs: vec![col("household_id")],
                space: "household".to_string(),
            }],
        )]),
        vec![UniqueKey {
            table: "households".to_string(),
            key: col("household_id"),
        },
        UniqueKey {
            table: "users".to_string(),
            key: col("user_id"),
        }],
        vec![
            ForeignKey {
                from_table: "users".to_string(),
                from: col("household_id"),
                to_table: "households".to_string(),
                to: col("household_id"),
            },
            ForeignKey {
                from_table: "events".to_string(),
                from: col("user_id"),
                to_table: "users".to_string(),
                to: col("user_id"),
            },
        ],
        vec![FunctionalDependency {
            table: "users".to_string(),
            from: col("user_id"),
            to: col("household_id"),
        }],
        vec![],
    )?;

    let users = metric.bindings.get("users").unwrap();
    assert!(expr_identifies_protected_id(users, "household", &col("household_id")));

    let events = metric.bindings.get("events").unwrap();
    assert!(expr_identifies_protected_id(events, "household", &col("user_id")));
    Ok(())
}

#[test]
fn test_compile_database_id_distance_does_not_propagate_transitively_without_intermediate_uniqueness(
) -> Fallible<()> {
    let metric = compile_database_id_distance(
        "household".to_string(),
        HashMap::from([(
            "households".to_string(),
            vec![Binding {
                exprs: vec![col("household_id")],
                space: "household".to_string(),
            }],
        )]),
        vec![UniqueKey {
            table: "households".to_string(),
            key: col("household_id"),
        }],
        vec![
            ForeignKey {
                from_table: "users".to_string(),
                from: col("household_id"),
                to_table: "households".to_string(),
                to: col("household_id"),
            },
            ForeignKey {
                from_table: "events".to_string(),
                from: col("user_id"),
                to_table: "users".to_string(),
                to: col("user_id"),
            },
        ],
        vec![],
        vec![],
    )?;

    let users = metric.bindings.get("users").unwrap();
    assert!(expr_identifies_protected_id(users, "household", &col("household_id")));

    let events = metric.bindings.get("events").unwrap();
    assert!(!expr_identifies_protected_id(events, "household", &col("user_id")));
    Ok(())
}

#[test]
fn test_normalize_claim_with_bindings_collapses_equivalent_protected_aliases() {
    let normalized = normalize_claim_with_bindings(
        &vec![col("user_id"), col("id")],
        &[Binding {
            exprs: vec![col("user_id"), col("id")],
            space: "user".to_string(),
        }],
        "user",
    );

    assert_eq!(normalized.len(), 1);
}

#[test]
fn test_normalize_claim_with_bindings_merges_overlapping_binding_classes_transitively() {
    let normalized = normalize_claim_with_bindings(
        &vec![col("a"), col("c")],
        &[
            Binding {
                exprs: vec![col("a"), col("b")],
                space: "user".to_string(),
            },
            Binding {
                exprs: vec![col("b"), col("c")],
                space: "user".to_string(),
            },
        ],
        "user",
    );

    assert_eq!(normalized.len(), 1);
}

#[test]
fn test_normalize_claims_with_bindings_preserves_known_empty_claim() {
    let normalized = normalize_claims_with_bindings(
        &[vec![]],
        &[Binding {
            exprs: vec![col("user_id")],
            space: "user".to_string(),
        }],
        "user",
    );

    assert_eq!(normalized, vec![vec![]]);
}

#[test]
fn test_choose_owner_claim_preserves_multi_owner_source_claim() {
    let claim = choose_owner_claim(&[vec![col("buyer_id"), col("seller_id")]]).unwrap();
    assert_eq!(claim, vec![col("buyer_id"), col("seller_id")]);
}

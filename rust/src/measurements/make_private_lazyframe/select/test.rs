use std::collections::HashMap;

use crate::domains::{
    AtomDomain, DatabaseDomain, LazyFrameDomain, Margin, SeriesDomain,
};
use crate::error::ErrorVariant::MakeMeasurement;
use crate::error::*;
use crate::measurements::{make_private_database_lazyframe, make_private_lazyframe};
use crate::measures::MaxDivergence;
use crate::polars::PrivacyNamespace;
use polars::prelude::*;

use crate::metrics::{DatabaseIdDistance, IdSite, SymmetricDistance};

use super::*;

#[test]
fn test_select_no_margin() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => &[1i32, 2, 2])?.lazy();

    let m_select = make_private_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        lf.clone().select(&[len().dp().noise(Some(0.))]),
        Some(1.),
        None,
    )?;

    let actual = m_select.invoke(&lf)?.collect()?;
    let expect = df!("len" => [3])?;

    assert_eq!(actual, expect);
    Ok(())
}

#[test]
fn test_select() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
            .with_margin(Margin::select().with_max_length(10))?;

    let lf = df!("A" => &[1i32, 2, 2])?.lazy();

    let m_select = make_private_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        lf.clone().select(&[
            col("A").dp().sum((lit(0), lit(3)), Some(0.)),
            len().dp().noise(Some(0.)),
        ]),
        Some(1.),
        None,
    )?;

    let actual = m_select.invoke(&lf)?.collect()?;
    let expect = df!("A" => [5], "len" => [3])?;

    assert_eq!(actual, expect);
    Ok(())
}

#[test]
fn test_fail_select_invalid_expression() -> Fallible<()> {
    let lf_domain = DslPlanDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => &[1i32, 2, 2])?.lazy();

    let error_variant_res = make_private_select::<_, _>(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        // this expression cannot be parsed into a measurement
        lf.select(&[col("A").sum()]).logical_plan,
        Some(1.),
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(MakeMeasurement, error_variant_res);

    Ok(())
}

#[test]
fn test_database_select_requires_truncation() -> Fallible<()> {
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
        "user_id" => [1i32, 1],
        "value" => [10i32, 11]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users"],
        "id" => [1i32],
        "age" => [30i32]
    )?
    .lazy();

    let plan = events
        .clone()
        .join(users.clone(), [col("user_id")], [col("id")], JoinType::Left.into())
        .select([len().dp().noise(Some(0.))]);

    let err = make_private_database_lazyframe(
        database_domain,
        database_metric,
        MaxDivergence,
        plan,
        Some(1.),
        None,
    )
    .unwrap_err();

    let message = err.message.unwrap_or_default();
    assert!(
        message.contains("truncation")
            || message.contains("per_group contributions is unknown")
            || message.contains("not recognized")
    );
    Ok(())
}

#[test]
fn test_database_select_with_explicit_truncation_constructs() -> Fallible<()> {
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
        .clone()
        .join(
            users.clone(),
            [col("user_id")],
            [col("user_id")],
            JoinType::Left.into(),
        )
        .filter(truncation)
        .select([len().dp().noise(Some(1.0))]);

    let meas = make_private_database_lazyframe(
        database_domain,
        database_metric,
        MaxDivergence,
        plan,
        Some(1.0),
        None,
    )?;

    let release = meas
        .invoke(&std::collections::HashMap::from([
            ("events".to_string(), events),
            ("users".to_string(), users),
        ]))?
        .collect()?;
    assert_eq!(release.height(), 1);
    assert_eq!(release.get_column_names_str(), vec!["len"]);

    Ok(())
}

#[test]
fn test_database_select_with_explicit_truncation_still_requires_positive_noise() -> Fallible<()> {
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
        .join(
            users,
            [col("user_id")],
            [col("user_id")],
            JoinType::Left.into(),
        )
        .filter(truncation)
        .select([len().dp().noise(Some(0.0))]);

    let meas = make_private_database_lazyframe(
        database_domain,
        database_metric,
        MaxDivergence,
        plan,
        Some(0.0),
        None,
    )?;
    assert!(meas.check(&1, &1.0).is_err());
    Ok(())
}

#[test]
#[ignore = "multi-table private selection currently requires an explicit truncation transform"]
fn test_database_select_with_bounded_contributions() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["user_id"]).with_max_length(1))?;
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
        "value" => [10i32, 12]
    )?
    .lazy();
    let users = df!(
        "__OPENDP_TABLE_NAME__[users]" => ["users", "users"],
        "id" => [1i32, 2],
        "age" => [30i32, 31]
    )?
    .lazy();

    let plan = events
        .clone()
        .join(users.clone(), [col("user_id")], [col("id")], JoinType::Left.into())
        .select([len().dp().noise(Some(0.))]);

    let meas = make_private_database_lazyframe(
        database_domain,
        database_metric,
        MaxDivergence,
        plan.clone(),
        Some(1.),
        None,
    )?;

    let release = meas
        .invoke(&std::collections::HashMap::from([
            ("events".to_string(), events),
            ("users".to_string(), users),
        ]))?
        .collect()?;
    assert_eq!(release, df!("len" => [2u32])?);
    Ok(())
}

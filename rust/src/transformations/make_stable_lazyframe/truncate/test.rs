use std::collections::HashMap;

use crate::{
    core::Transformation,
    domains::{
        AtomDomain, DatabaseDomain, DslPlanDomain, LazyFrameDomain, Margin, OptionDomain,
        SeriesDomain,
    },
    metrics::{
        Binding, Bounds, DatabaseIdDistance, FrameDistance, SymmetricDistance, SymmetricIdDistance,
    },
    transformations::{StableDslPlan, make_stable_lazyframe},
};
use polars::prelude::{DataType, int_range, len, lit};

use super::*;

#[test]
fn test_filter() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?
    .with_margin(Margin::by(["chunk_2_null"]).with_invariant_keys())?;

    let t_filter = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().filter(col("chunk_2_null").is_not_null()),
    )?;

    let actual = t_filter.invoke(&lf)?.collect()?;
    assert_eq!(actual, df!("chunk_2_null" => [Some(1)])?);

    assert!(
        t_filter
            .output_domain
            .margins
            .iter()
            .all(|m| { m.invariant.is_none() })
    );

    Ok(())
}

#[test]
fn test_filter_fail_with_non_bool_predicate() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?
    .with_margin(Margin::by(["chunk_2_null"]).with_invariant_keys())?;

    let variant = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().filter(col("chunk_2_null").fill_nan(0)),
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(variant, ErrorVariant::MakeTransformation);

    Ok(())
}

#[test]
fn test_database_truncation_matches_single_table_stability_map() -> Fallible<()> {
    let frame_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("alpha", AtomDomain::<i32>::default()),
        SeriesDomain::new("id", AtomDomain::<i32>::default()),
    ])?;

    let frame_metric = FrameDistance(SymmetricIdDistance {
        protect: "user".to_string(),
        bindings: vec![Binding {
            space: "user".to_string(),
            exprs: vec![col("id")],
        }],
    });

    let database_domain = DatabaseDomain::new(HashMap::from([(
        "events".to_string(),
        frame_domain.clone(),
    )]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![Binding {
                space: "user".to_string(),
                exprs: vec![col("id")],
            }],
        )]),
    };

    let truncation = int_range(lit(0), len(), 1, DataType::Int64)
        .over([col("id")])
        .lt(lit(2u32));

    let frame_plan = df!("alpha" => [1i32, 2], "id" => [1i32, 1])?
        .lazy()
        .filter(truncation.clone())
        .logical_plan;

    let database_plan = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events", "events"],
        "alpha" => [1i32, 2],
        "id" => [1i32, 1]
    )?
    .lazy()
    .filter(truncation)
    .logical_plan;

    let t_frame: Transformation<
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
        DslPlanDomain,
        FrameDistance<SymmetricDistance>,
    > = frame_plan.make_stable(frame_domain.cast_carrier(), frame_metric)?;
    let t_database: Transformation<
        DatabaseDomain,
        DatabaseIdDistance,
        DslPlanDomain,
        FrameDistance<SymmetricDistance>,
    > = database_plan.make_stable(database_domain, database_metric)?;

    assert_eq!(t_frame.output_domain, t_database.output_domain);
    assert_eq!(
        t_frame.stability_map.eval(&Bounds::from(1))?,
        t_database.stability_map.eval(&1)?
    );

    Ok(())
}

#[test]
fn test_database_truncation_rejects_multiple_protected_identifier_exprs() -> Fallible<()> {
    let events_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("user_id", AtomDomain::<i32>::default()),
        SeriesDomain::new("user_id_dup", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?;

    let database_domain =
        DatabaseDomain::new(HashMap::from([("events".to_string(), events_domain)]));
    let database_metric = DatabaseIdDistance {
        protect: "user".to_string(),
        bindings: HashMap::from([(
            "events".to_string(),
            vec![
                Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id")],
                },
                Binding {
                    space: "user".to_string(),
                    exprs: vec![col("user_id_dup")],
                },
            ],
        )]),
    };

    let truncation = int_range(lit(0), len(), 1, DataType::Int64)
        .over([col("user_id")])
        .lt(lit(1u32));
    let plan = df!(
        "__OPENDP_TABLE_NAME__[events]" => ["events", "events", "events"],
        "user_id" => [1i32, 1, 2],
        "user_id_dup" => [1i32, 1, 2],
        "value" => [10i32, 11, 12]
    )?
    .lazy()
    .filter(truncation)
    .logical_plan;

    let err = <DslPlan as StableDslPlan<
        DatabaseDomain,
        DatabaseIdDistance,
        FrameDistance<SymmetricDistance>,
    >>::make_stable(plan, database_domain, database_metric)
    .unwrap_err();

    assert!(
        err.message
            .unwrap_or_default()
            .contains("supports at most one identifier site")
    );
    Ok(())
}

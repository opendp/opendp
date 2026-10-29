use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, OptionDomain, SeriesDomain},
    metrics::SymmetricDistance,
    transformations::make_stable_lazyframe,
};

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
fn test_truncate_domain_computed_group_keys() -> Fallible<()> {
    let key = col("A") + col("B");
    let domain = DslPlanDomain::new(vec![
        SeriesDomain::new("ID", AtomDomain::<i32>::default()),
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        Margin::by([key.clone()])
            .with_max_groups(4)
            .with_invariant_keys(),
    )?;

    let output = truncate_domain(
        domain,
        &Truncation::GroupBy {
            keys: vec![col("ID"), key.clone()],
            aggs: vec![col("B").sum()],
        },
    )?;

    // A now contains the former A + B, while B contains sum(B). Evaluating
    // the same expression on those new columns describes different groups.
    assert!(output.margins.iter().all(|m| !m.by.contains(&key)));
    assert_eq!(
        output.margins,
        vec![Margin::by([col("A")]).with_max_groups(4)]
    );
    assert!(output.margins.iter().all(|m| m.invariant.is_none()));
    Ok(())
}

#[test]
fn test_truncate_domain_aliased_group_keys() -> Fallible<()> {
    let key = col("A").alias("key");
    let domain = DslPlanDomain::new(vec![
        SeriesDomain::new("ID", AtomDomain::<i32>::default()),
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by([key.clone()]).with_max_groups(4))?;

    let output = truncate_domain(
        domain,
        &Truncation::GroupBy {
            keys: vec![col("ID"), key.clone()],
            aggs: vec![],
        },
    )?;

    // An alias names an output column; it is not a request to evaluate the
    // original input-column expression again against the output schema.
    assert!(output.margins.iter().all(|m| !m.by.contains(&key)));
    assert_eq!(
        output.margins,
        vec![Margin::by([col("key")]).with_max_groups(4)]
    );
    Ok(())
}

#[test]
fn test_truncate_domain_plain_group_keys() -> Fallible<()> {
    let margin = Margin::by([col("A")])
        .with_max_groups(4)
        .with_max_length(3)
        .with_invariant_lengths();
    let domain = DslPlanDomain::new(vec![
        SeriesDomain::new("ID", AtomDomain::<i32>::default()),
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
    ])?
    .with_margin(margin.clone())?;

    let output = truncate_domain(
        domain,
        &Truncation::GroupBy {
            keys: vec![col("ID"), col("A")],
            aggs: vec![],
        },
    )?;

    assert_eq!(
        output.margins,
        vec![Margin {
            invariant: None,
            ..margin
        }]
    );
    Ok(())
}

#[test]
fn test_truncate_domain_rejects_duplicate_output_key_names() -> Fallible<()> {
    let domain = DslPlanDomain::new(vec![
        SeriesDomain::new("ID", AtomDomain::<i32>::default()),
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?;
    let error = truncate_domain(
        domain,
        &Truncation::GroupBy {
            keys: vec![col("ID"), col("A"), col("A") + col("B")],
            aggs: vec![],
        },
    )
    .unwrap_err();

    // Both A and A + B emit column A, so no unambiguous output schema or
    // partition certificates can be constructed for this query.
    assert_eq!(error.variant, ErrorVariant::MakeTransformation);
    Ok(())
}

#[test]
fn test_truncate_computed_key_bound_after_group_by() -> Fallible<()> {
    let lf = df!(
        "ID" => [7i32; 4],
        "A" => [20i32, 18, 16, 14],
        "B" => [0i32, 1, 2, 3],
    )?
    .lazy();
    let domain = LazyFrameDomain::new_from_schema((*lf.clone().collect_schema()?).clone())?;
    let key = col("A") + col("B");
    let predicate = key
        .clone()
        .rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
        .over([col("ID")])
        .lt_eq(lit(4));
    let plan = lf
        .clone()
        .filter(predicate)
        .group_by([col("ID"), key.clone()])
        .agg([col("B").sum()]);
    let t: Transformation<_, _, _, FrameDistance<SymmetricDistance>> = make_stable_lazyframe(
        domain,
        SymmetricIdDistance {
            identifier: col("ID"),
        },
        plan,
    )?;

    let output_bounds = t.map(&1)?;
    assert!(output_bounds.0.iter().all(|b| !b.by.contains(&key)));
    let materialized_bound = output_bounds.get_bound(&HashSet::from([col("A")]));
    assert_eq!(materialized_bound.per_group, Some(1));
    // The matcher conservatively interprets <= 4 as an upper bound of 5.
    assert_eq!(materialized_bound.num_groups, Some(5));
    let recomputed_bound = output_bounds.get_bound(&HashSet::from([key.clone()]));
    assert!(recomputed_bound.per_group.is_none_or(|bound| bound >= 4));

    let output = t.invoke(&lf)?.group_by([key]).agg([len()]).collect()?;
    assert_eq!(output.column("len")?.u32()?.get(0), Some(4));
    Ok(())
}

#[test]
fn test_truncate_input_bounds_use_original_group_key() -> Fallible<()> {
    let lf = df!("ID" => [7i32, 7], "A" => [1i32, 2], "B" => [2i32, 3])?.lazy();
    let domain = LazyFrameDomain::new_from_schema((*lf.clone().collect_schema()?).clone())?;
    let key = (col("A") + col("B")).alias("key");
    let plan = lf.group_by([col("ID"), key.clone()]).agg([col("B").sum()]);
    let t: Transformation<_, _, _, FrameDistance<SymmetricDistance>> = make_stable_lazyframe(
        domain,
        FrameDistance(SymmetricIdDistance {
            identifier: col("ID"),
        }),
        plan,
    )?;
    let input_bounds = Bounds(vec![
        Bound::default().with_per_group(2),
        Bound::by([key]).with_per_group(1).with_num_groups(4),
    ]);
    let output = t.map(&input_bounds)?;
    let bound = output.get_bound(&HashSet::from([col("key")]));

    // The input certificate applies to the original expression, so its
    // tighter one-ID bound must be read before relabeling the output key.
    assert_eq!(bound.per_group, Some(1));
    assert_eq!(bound.num_groups, Some(4));
    Ok(())
}

#[test]
fn test_truncate_rejects_filter_after_group_by() -> Fallible<()> {
    let lf = df!("ID" => [7i32], "A" => [1i32])?.lazy();
    let predicate = col("A")
        .rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
        .over([col("ID")])
        .lt_eq(lit(4));
    let plan = lf.group_by([col("ID"), col("A")]).agg([]).filter(predicate);
    let error = match_truncations(plan.logical_plan, &col("ID")).unwrap_err();
    assert!(
        error
            .message
            .unwrap()
            .starts_with("Groupby truncation must be the last truncation in the plan.")
    );
    Ok(())
}

#[test]
fn test_truncate_rejects_contiguous_group_bys() -> Fallible<()> {
    let plan = df!("ID" => [7i32], "A" => [1i32])?
        .lazy()
        .group_by([col("ID"), col("A")])
        .agg([])
        .group_by([col("ID"), col("A")])
        .agg([]);
    let error = match_truncations(plan.logical_plan, &col("ID")).unwrap_err();
    assert!(
        error
            .message
            .unwrap()
            .starts_with("Groupby truncation must be the last truncation in the plan.")
    );
    Ok(())
}

#[test]
fn test_truncate_plain_key_bound_after_group_by() -> Fallible<()> {
    let lf = df!("ID" => [7i32; 4], "A" => [1i32, 2, 3, 4])?.lazy();
    let domain = LazyFrameDomain::new_from_schema((*lf.clone().collect_schema()?).clone())?;
    let predicate = col("A")
        .rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
        .over([col("ID")])
        .lt_eq(lit(4));
    let plan = lf
        .clone()
        .filter(predicate)
        .group_by([col("ID"), col("A")])
        .agg([]);
    let t: Transformation<_, _, _, FrameDistance<SymmetricDistance>> = make_stable_lazyframe(
        domain,
        SymmetricIdDistance {
            identifier: col("ID"),
        },
        plan,
    )?;

    let bound = t.map(&1)?.get_bound(&HashSet::from([col("A")]));
    assert_eq!(bound.per_group, Some(1));
    assert_eq!(bound.num_groups, Some(5));
    Ok(())
}

#[test]
fn test_truncate_materializes_each_bound_grouping_subset() -> Fallible<()> {
    let lf = df!(
        "ID" => [7i32; 4],
        "A" => [10i32, 9, 8, 7],
        "B" => [0i32, 1, 2, 3],
        "C" => [0i32, 1, 2, 3],
    )?
    .lazy();
    let domain = LazyFrameDomain::new_from_schema((*lf.clone().collect_schema()?).clone())?;
    let key = col("A") + col("B");
    let predicate = key
        .clone()
        .rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
        .over([col("ID")])
        .lt_eq(lit(1));
    let plan = lf
        .clone()
        .filter(predicate)
        .group_by([col("ID"), key, col("C")])
        .agg([col("B").sum()]);
    let t: Transformation<_, _, _, FrameDistance<SymmetricDistance>> = make_stable_lazyframe(
        domain,
        SymmetricIdDistance {
            identifier: col("ID"),
        },
        plan,
    )?;

    let bounds = t.map(&1)?;
    let subset_bound = bounds.get_bound(&HashSet::from([col("A")]));
    // The <= 1 predicate receives the same conservative offset.
    assert_eq!(subset_bound.num_groups, Some(2));
    assert_eq!(subset_bound.per_group, None);
    let full_bound = bounds.get_bound(&HashSet::from([col("A"), col("C")]));
    assert_eq!(full_bound.per_group, Some(1));

    // The rank filter limits A + B groups, while the groupby emits one row
    // per (A + B, C). Four different C values still contribute to output A.
    let output = t.invoke(&lf)?.group_by([col("A")]).agg([len()]).collect()?;
    assert_eq!(output.column("len")?.u32()?.get(0), Some(4));
    Ok(())
}

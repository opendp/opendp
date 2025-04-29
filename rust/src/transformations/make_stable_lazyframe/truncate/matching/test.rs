use polars::{
    df,
    prelude::{IntoLazy, RankOptions, all_horizontal, as_struct, col},
};

use super::*;

#[test]
fn test_match_truncations_ok() -> Fallible<()> {
    let lf = df!["A" => [1], "B" => [1], "ID" => [1]]?.lazy();

    let rank_opt = RankOptions {
        method: RankMethod::Dense,
        descending: false,
    };
    let truncate_num_groups = col("A").rank(rank_opt, None).over([col("ID")]).lt(lit(10));

    let plan = lf
        .clone()
        .filter(truncate_num_groups.clone())
        .group_by([col("ID"), col("A")])
        .agg([col("B").sum()]);

    let (_, truncations, truncation_bounds) =
        match_truncations(plan.logical_plan.clone(), &col("ID")).unwrap();
    assert_eq!(
        truncations,
        vec![
            Truncation::Filter(truncate_num_groups),
            Truncation::GroupBy {
                keys: vec![col("ID"), col("A")],
                aggs: vec![col("B").sum()]
            },
        ]
    );
    assert_eq!(
        truncation_bounds,
        vec![
            Bound {
                by: HashSet::from([col("A")]),
                per_group: None,
                num_groups: Some(10),
            },
            Bound {
                by: HashSet::from([col("A")]),
                per_group: Some(1),
                num_groups: None,
            },
        ]
    );

    Ok(())
}

#[test]
fn test_match_truncations_groupby_not_last() -> Fallible<()> {
    let lf = df!["A" => [1], "B" => [1], "ID" => [1]]?.lazy();

    let rank_opt = RankOptions {
        method: RankMethod::Dense,
        descending: false,
    };
    let truncate_num_groups = col("A").rank(rank_opt, None).over([col("ID")]).lt(lit(10));

    let plan = lf
        .group_by([col("ID"), col("A")])
        .agg([col("B").sum()])
        .filter(truncate_num_groups.clone());

    // should get a nice error when group by truncation is not last
    let res = match_truncations(plan.logical_plan.clone(), &col("ID"));
    let msg = "Groupby truncation must be the last truncation in the plan.";
    assert!((res.map(|_| ()).unwrap_err().message.unwrap()).starts_with(msg));
    Ok(())
}

#[test]
fn test_match_truncations_phony_filter() -> Fallible<()> {
    let lf = df!["A" => [1], "B" => [1], "ID" => [1]]?.lazy();

    let rank_opt = RankOptions {
        method: RankMethod::Dense,
        descending: false,
    };
    let truncate_num_groups = col("A").rank(rank_opt, None).over([col("ID")]).lt(lit(10));

    let plan = lf
        .filter(truncate_num_groups.clone())
        .filter(col("B").lt(lit(0)));

    // no error, but doesn't match as a truncation due to leading filter
    assert!(
        match_truncations(plan.logical_plan.clone(), &col("ID"))?
            .1
            .is_empty()
    );
    Ok(())
}

#[test]
fn test_match_group_by_truncation_no_grouping() -> Fallible<()> {
    let lf = df!["A" => [1], "B" => [1], "ID" => [1]]?.lazy();

    let plan = lf.clone().group_by([col("ID")]).agg([col("B").sum()]);

    let (_, truncation, bound) = match_group_by_truncation(&plan.logical_plan, &col("ID")).unwrap();
    assert_eq!(
        truncation,
        Truncation::GroupBy {
            keys: vec![col("ID")],
            aggs: vec![col("B").sum()]
        }
    );

    assert_eq!(
        bound,
        Bound {
            by: HashSet::new(),
            per_group: Some(1),
            num_groups: None,
        }
    );

    Ok(())
}

#[test]
fn test_match_group_by_truncation_with_grouping() -> Fallible<()> {
    let lf = df!["A" => [1], "B" => [1], "ID" => [1]]?.lazy();

    let plan = lf
        .clone()
        .group_by([col("ID"), col("A")])
        .agg([col("B").sum()]);

    let (_, truncation, bound) = match_group_by_truncation(&plan.logical_plan, &col("ID")).unwrap();
    assert_eq!(
        truncation,
        Truncation::GroupBy {
            keys: vec![col("ID"), col("A")],
            aggs: vec![col("B").sum()]
        }
    );

    assert_eq!(
        bound,
        Bound {
            by: HashSet::from([col("A")]),
            per_group: Some(1),
            num_groups: None,
        }
    );

    Ok(())
}

#[test]
fn test_match_truncation_predicate_tree() -> Fallible<()> {
    let bounds = vec![
        Bound {
            by: HashSet::from([col("A")]),
            per_group: None,
            num_groups: Some(10),
        },
        Bound {
            by: HashSet::from([col("A")]),
            per_group: Some(11),
            num_groups: None,
        },
    ];

    let rank_opt = RankOptions {
        method: RankMethod::Dense,
        descending: false,
    };
    let truncate_num_groups = col("A").rank(rank_opt, None).over([col("ID")]).lt(lit(10));
    let truncate_per_group = int_range(lit(0), len(), 1, DataType::Int64)
        .over([col("A"), col("ID")])
        .lt(lit(11));

    assert_eq!(
        match_truncation_predicate(
            &truncate_num_groups.clone().and(truncate_per_group.clone()),
            &col("ID")
        )?,
        Some(bounds.clone())
    );

    assert_eq!(
        match_truncation_predicate(
            &all_horizontal([truncate_num_groups.clone(), truncate_per_group.clone()])?,
            &col("ID")
        )?,
        Some(bounds)
    );

    assert!(
        match_truncation_predicate(
            &all_horizontal([truncate_num_groups.clone(), col("D").lt(4)])?,
            &col("ID")
        )?
        // Any non-truncation predicate should return None, failure to match.
        // Want to ensure no other non-row-by-row or fallible predicates get through.
        .is_none()
    );

    Ok(())
}

#[test]
fn test_match_truncation_predicate_op() -> Fallible<()> {
    let bound = Bound {
        by: HashSet::from([col("A")]),
        per_group: Some(12),
        num_groups: None,
    };

    let check = |expr: Expr| {
        assert_eq!(
            match_truncation_predicate(&expr, &col("ID")),
            Ok(Some(vec![bound.clone()]))
        );
    };
    let over = int_range(lit(0), len(), 1, DataType::Int64).over([col("A"), col("ID")]);

    check(over.clone().lt_eq(lit(11)));
    check(over.clone().lt(lit(12)));
    check(lit(12).gt(over.clone()));
    check(lit(11).gt_eq(over.clone()));

    Ok(())
}

#[test]
fn test_match_num_groups_predicate() -> Fallible<()> {
    let bound = Some(Bound {
        by: HashSet::from([col("A")]),
        per_group: None,
        num_groups: Some(10),
    });

    let check = |expr: Expr| match_num_groups_predicate(&expr, &vec![col("ID")], &col("ID"), 10);
    assert_eq!(
        check(col("A").rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        ))?,
        bound
    );

    assert_eq!(
        check(as_struct(vec![col("A")]).rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: true,
            },
            None,
        ))?,
        bound
    );

    assert!(check(as_struct(vec![col("A")]))?.is_none());
    Ok(())
}

#[test]
fn test_match_per_group_predicate_enumerations() -> Fallible<()> {
    let bound = Some(Bound {
        by: HashSet::from([col("A")]),
        per_group: Some(10),
        num_groups: None,
    });

    let check =
        |expr: Expr| match_per_group_predicate(&expr, &vec![col("A"), col("ID")], &col("ID"), 10);

    let range = int_range(lit(0), len(), 1, DataType::Int64);
    assert_eq!(check(range.clone())?, bound);
    assert_eq!(check(range.clone().reverse())?, bound);
    assert_eq!(check(range.clone().shuffle(None))?, bound);
    assert_eq!(
        check(range.clone().sort_by([col("D")], Default::default()))?,
        bound
    );
    assert!(check(int_range(lit(0), len(), 1, DataType::Int64).head(None))?.is_none());
    Ok(())
}

#[test]
fn test_match_per_group_predicate_id() -> Fallible<()> {
    let range = int_range(lit(0), len(), 1, DataType::Int64);
    // no matches should error
    assert!(match_per_group_predicate(&range, &vec![col("A")], &col("ID"), 10).is_err());
    // multiple matches should pass
    assert!(match_per_group_predicate(&range, &vec![col("ID"); 2], &col("ID"), 0)?.is_some());
    Ok(())
}

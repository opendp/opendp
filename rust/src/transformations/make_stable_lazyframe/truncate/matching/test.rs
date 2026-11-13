#[cfg(feature = "ffi")]
use polars::prelude::all_horizontal;
use polars::{
    df,
    prelude::{IntoLazy, RankOptions, as_struct, col},
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
                num_groups: Some(9),
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

    // should get a nice error when group-by truncation is not last
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

#[cfg(feature = "ffi")]
#[test]
fn test_match_truncation_predicate_tree() -> Fallible<()> {
    let bounds = vec![
        Bound {
            by: HashSet::from([col("A")]),
            per_group: None,
            num_groups: Some(9),
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
    let truncate_per_group = random_enumeration().over([col("A"), col("ID")]).lt(lit(11));

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

#[cfg(feature = "ffi")]
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
    let over = random_enumeration().over([col("A"), col("ID")]);

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

    let keys = as_struct(vec![col("A")]);
    let hashed_keys = as_struct(vec![
        keys.clone().hash(0, 0, 0, 0).alias("__opendp_hash"),
        keys.alias("__opendp_key"),
    ]);
    assert_eq!(
        check(hashed_keys.rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        ))?,
        bound
    );
    Ok(())
}

#[test]
fn test_dense_rank_threshold_is_one_based() -> Fallible<()> {
    let ranks = col("A")
        .rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
        .over([col("ID")]);
    for k in [0u32, 1, 2, 10] {
        for (predicate, expected) in [
            (ranks.clone().lt(lit(k)), k.saturating_sub(1)),
            (ranks.clone().lt_eq(lit(k)), k),
        ] {
            let bound = match_truncation_predicate(&predicate, &col("ID"))?
                .expect("expected num_groups truncation");
            assert_eq!(bound[0].num_groups, Some(expected));
        }
    }
    Ok(())
}

#[test]
fn test_match_per_group_predicate_enumerations() -> Fallible<()> {
    let check =
        |expr: Expr| match_per_group_predicate(&expr, &vec![col("A"), col("ID")], &col("ID"), 10);

    let range = int_range(lit(0), len(), 1, DataType::Int64);
    assert!(check(range.clone()).is_err());
    // Unsupported reorderings are not recognized as truncations.
    assert!(check(range.clone().reverse())?.is_none());
    assert!(check(range.clone().shuffle(None))?.is_none());
    assert!(check(range.clone().sort_by([col("D")], Default::default()))?.is_none());
    assert!(check(int_range(lit(0), len(), 1, DataType::Int64).head(None))?.is_none());
    Ok(())
}

#[cfg(feature = "ffi")]
#[test]
fn test_match_per_group_predicate_id() -> Fallible<()> {
    let range = random_enumeration();
    // no matches should error
    assert!(match_per_group_predicate(&range, &vec![col("A")], &col("ID"), 10).is_err());
    // multiple matches should pass
    assert!(match_per_group_predicate(&range, &vec![col("ID"); 2], &col("ID"), 0)?.is_some());
    Ok(())
}

#[cfg(feature = "ffi")]
fn permute(data: Expr) -> Expr {
    Expr::Function {
        input: vec![data],
        function: FunctionExpr::FfiPlugin {
            flags: polars_plan::prelude::FunctionOptions::length_preserving(),
            lib: "opendp".into(),
            symbol: "permute_plugin".into(),
            kwargs: Default::default(),
        },
    }
}

#[cfg(feature = "ffi")]
fn random_enumeration() -> Expr {
    permute(int_range(lit(0), len(), 1, DataType::Int64))
}

#[cfg(feature = "ffi")]
#[test]
fn test_match_full_random_truncation_ranks() -> Fallible<()> {
    let range = int_range(lit(0), len(), 1, DataType::Int64);
    let check = |expr: Expr| match_per_group_predicate(&expr, &vec![col("ID")], &col("ID"), 1);
    assert!(check(random_enumeration())?.is_some());
    // A malformed plugin with an extra argument is not a trusted permutation.
    let malformed = Expr::Function {
        input: vec![range.clone(), lit(1)],
        function: FunctionExpr::FfiPlugin {
            flags: polars_plan::prelude::FunctionOptions::length_preserving(),
            lib: "opendp".into(),
            symbol: "permute_plugin".into(),
            kwargs: Default::default(),
        },
    };
    assert!(check(malformed.clone())?.is_none());
    assert!(
        match_truncation_predicate(&malformed.over([col("ID")]).lt(lit(1)), &col("ID")).is_err()
    );
    // Ordinal ranks are aligned to rows, and a final trusted permutation
    // breaks ties uniformly without relying on physical input order.
    let ranked = |keys: Vec<Expr>, method| {
        as_struct(keys).rank(
            RankOptions {
                method,
                descending: false,
            },
            None,
        )
    };
    let trusted = ranked(
        vec![
            col("A").alias("__sort0"),
            random_enumeration().alias("__tie"),
        ],
        RankMethod::Ordinal,
    );
    assert!(check(trusted.clone())?.is_some());
    // A uniqueness key is permutation-equivariant but not row-local.
    let uniqueness_key = ranked(
        vec![col("A").is_unique(), random_enumeration().alias("__tie")],
        RankMethod::Ordinal,
    );
    assert!(check(uniqueness_key)?.is_some());
    assert_eq!(
        match_per_group_predicate(&trusted, &vec![col("ID")], &col("ID"), 2)?
            .unwrap()
            .per_group,
        Some(1)
    );

    // A trusted tie-breaking key must be last and the method ordinal.
    assert!(check(ranked(vec![col("A")], RankMethod::Ordinal))?.is_none());
    assert!(
        check(ranked(
            vec![random_enumeration().alias("__tie"), col("A")],
            RankMethod::Ordinal
        ))?
        .is_none()
    );
    assert!(
        check(ranked(
            vec![col("A"), random_enumeration().alias("__tie")],
            RankMethod::Dense
        ))?
        .is_none()
    );

    // Positional sort keys do not follow rows under input permutation.
    assert!(
        check(ranked(
            vec![range, random_enumeration().alias("__tie")],
            RankMethod::Ordinal
        ))
        .is_err()
    );
    Ok(())
}

#[test]
fn test_native_permutation_matches_without_ffi() -> Fallible<()> {
    use crate::polars::PrivacyNamespace;

    let permutation = int_range(lit(0), len(), 1, DataType::Int64).dp().permute();
    let by = vec![col("ID")];
    let bound = match_per_group_predicate(&permutation, &by, &col("ID"), 2)?
        .expect("native permutation should be trusted");
    assert_eq!(bound.per_group, Some(2));

    let ranks = as_struct(vec![col("A").alias("__sort0"), permutation.alias("__tie")]).rank(
        RankOptions {
            method: RankMethod::Ordinal,
            descending: false,
        },
        None,
    );
    let bound = match_per_group_predicate(&ranks, &by, &col("ID"), 2)?
        .expect("native permutation must certify ordinal tie-breaking");
    assert_eq!(bound.per_group, Some(1));
    Ok(())
}

#[test]
fn test_rank_window_sort_keys_are_partition_local() -> Fallible<()> {
    // If is_unique were evaluated globally, changing ID=1 to contain 20
    // would change the uniqueness flag (and rank) of ID=2's row A=20.
    let rank = as_struct(vec![col("A").is_unique(), col("A")])
        .rank(
            RankOptions {
                method: RankMethod::Ordinal,
                descending: false,
            },
            None,
        )
        .over([col("ID")])
        .alias("rank");

    let ranks_for_id_two = |other_id_value: i32| -> Fallible<Vec<u32>> {
        let values = df![
            "ID" => [1i32, 1, 2, 2, 2],
            "A" => [other_id_value, other_id_value, 10, 20, 30],
        ]?
        .lazy()
        .with_columns([rank.clone()])
        .filter(col("ID").eq(lit(2)))
        .select([col("rank")])
        .collect()?;

        Ok(values.column("rank")?.u32()?.into_no_null_iter().collect())
    };

    assert_eq!(ranks_for_id_two(1)?, vec![1, 2, 3]);
    assert_eq!(ranks_for_id_two(20)?, vec![1, 2, 3]);
    Ok(())
}

use crate::polars::PrivacyNamespace;
use polars::prelude::all_horizontal;
use polars::{
    df,
    prelude::{Categories, Field, IntoLazy, RankOptions, as_struct, col},
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

    let MatchedTruncations {
        filters,
        group_by,
        bounds: truncation_bounds,
        ..
    } = match_truncations(plan.logical_plan.clone(), &col("ID"))?.expect("expected truncations");
    assert_eq!(filters, vec![truncate_num_groups]);
    assert_eq!(
        group_by,
        Some(GroupByTruncation {
            keys: vec![col("ID"), col("A")],
            aggs: vec![col("B").sum()],
        })
    );
    assert_eq!(
        truncation_bounds
            .into_iter()
            .map(|matched| matched.bound)
            .collect::<Vec<_>>(),
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
    assert!(match_truncations(plan.logical_plan.clone(), &col("ID"))?.is_none());
    Ok(())
}

#[test]
fn test_match_group_by_truncation() -> Fallible<()> {
    let lf = df!["A" => [1], "B" => [1], "ID" => [1]]?.lazy();
    for (keys, by) in [
        (vec![col("ID")], HashSet::new()),
        (vec![col("ID"), col("A")], HashSet::from([col("A")])),
    ] {
        let aggs = vec![col("B").sum()];
        let plan = lf.clone().group_by(keys.clone()).agg(aggs.clone());
        let (_, truncation, bound) =
            match_group_by_truncation(&plan.logical_plan, &col("ID")).unwrap();
        assert_eq!(truncation, GroupByTruncation { keys, aggs });
        assert_eq!(
            bound,
            Bound {
                by,
                per_group: Some(1),
                num_groups: None
            }
        );
    }
    Ok(())
}

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

    let check = |predicate: Expr| -> Fallible<_> {
        Ok(match_truncation_predicate(&predicate, &col("ID"))?
            .map(|matched| matched.into_iter().map(|m| m.bound).collect::<Vec<_>>()))
    };
    let a = truncate_num_groups;
    let b = truncate_per_group;
    let ordinary = col("D").lt(4);
    for predicate in [a.clone().and(b.clone()), all_horizontal([a.clone(), b])?] {
        assert_eq!(check(predicate)?, Some(bounds.clone()));
    }
    for predicate in [
        a.clone().and(ordinary.clone()),
        all_horizontal([a, ordinary])?,
    ] {
        assert!(check(predicate)?.is_none());
    }

    Ok(())
}

#[test]
fn test_match_truncation_predicate_op() -> Fallible<()> {
    let permutation = random_enumeration().over([col("A"), col("ID")]);
    let ranks = as_struct(vec![
        col("A").alias("__sort0"),
        random_enumeration().alias("__tie"),
    ])
    .rank(
        RankOptions {
            method: RankMethod::Ordinal,
            descending: false,
        },
        None,
    )
    .over([col("A"), col("ID")]);

    for (predicate, expected) in [
        (permutation.clone().lt_eq(lit(11)), 12),
        (permutation.clone().lt(lit(12)), 12),
        (lit(12).gt(permutation.clone()), 12),
        (lit(11).gt_eq(permutation.clone()), 12),
        (permutation.clone().lt(lit(u32::MAX)), u32::MAX),
        (ranks.clone().lt(lit(2)), 1),
        (ranks.lt_eq(lit(u32::MAX)), u32::MAX),
    ] {
        let bounds = match_truncation_predicate(&predicate, &col("ID"))?.unwrap();
        assert_eq!(bounds[0].bound.by, HashSet::from([col("A")]));
        assert_eq!(bounds[0].bound.per_group, Some(expected));
    }
    assert!(match_truncation_predicate(&permutation.lt_eq(lit(u32::MAX)), &col("ID")).is_err());
    Ok(())
}

#[test]
fn test_match_num_groups_predicate() -> Fallible<()> {
    let bound = Some(Bound {
        by: HashSet::from([col("A")]),
        per_group: None,
        num_groups: Some(10),
    });

    let check = |expr: Expr| match_num_groups_predicate(&expr, &[col("ID")], &col("ID"), 10);
    let dense = |expr: Expr| {
        expr.rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
    };
    let keys = as_struct(vec![col("A")]);
    let hashed = as_struct(vec![
        keys.clone().hash(0, 0, 0, 0).alias("__opendp_hash"),
        keys.clone().alias("__opendp_key"),
    ]);
    for expr in [col("A"), keys.clone(), hashed] {
        assert_eq!(check(dense(expr))?, bound);
    }
    assert!(check(keys.clone())?.is_none());

    // Extra struct fields affect ranking but must not disappear from the
    // certified grouping keys.
    let positional = as_struct(vec![
        keys.clone().hash(0, 0, 0, 0).alias("__opendp_hash"),
        keys.clone().alias("__opendp_key"),
        int_range(lit(0), len(), 1, DataType::Int64).alias("__position"),
    ]);
    assert!(check(dense(positional)).is_err());

    // Likewise, an additional hash input cannot be discarded.
    let malformed_hash = Expr::Function {
        input: vec![keys.clone(), col("B")],
        function: FunctionExpr::Hash(0, 0, 0, 0),
    };
    let malformed = as_struct(vec![
        malformed_hash.alias("__opendp_hash"),
        keys.alias("__opendp_key"),
    ]);
    assert!(check(dense(malformed)).is_err());
    Ok(())
}

#[test]
fn test_hashed_dense_rank_rejects_categorical_keys() -> Fallible<()> {
    let categories = Categories::global();
    let schema = Schema::from_iter([
        Field::new(
            "category".into(),
            DataType::Categorical(categories.clone(), categories.mapping()),
        ),
        Field::new("name".into(), DataType::String),
        Field::new("list".into(), DataType::List(Box::new(DataType::Int32))),
    ]);
    let rank = |keys: Vec<Expr>| {
        let keys = as_struct(keys);
        as_struct(vec![
            keys.clone().hash(0, 0, 0, 0).alias("__opendp_hash"),
            keys.alias("__opendp_key"),
        ])
        .rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
    };
    let check = |expr: Expr| {
        match_num_groups_predicate(&expr, &[col("ID")], &col("ID"), 1)?
            .map(|bound| MatchedBound::new(bound).validate_ranking(&schema))
            .transpose()
    };

    assert!(check(rank(vec![col("category")])).is_err());
    assert!(check(rank(vec![col("list")])).is_err());
    assert!(check(rank(vec![col("name")]))?.is_some());

    // Direct ranks also depend on categorical physical code assignments.
    assert!(
        check(as_struct(vec![col("category")]).rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        ))
        .is_err()
    );
    Ok(())
}

#[test]
fn test_truncation_window_rejects_order_by() -> Fallible<()> {
    let rank = col("A").rank(
        RankOptions {
            method: RankMethod::Dense,
            descending: false,
        },
        None,
    );

    // An order-by expression is evaluated by Polars even though it is not
    // part of the ranker or partitioning keys. A strict cast can fail on
    // private values, so this cannot be certified as a truncation.
    let ordered = rank
        .clone()
        .over_with_options(
            Some(vec![col("ID")]),
            Some((
                vec![col("secret").strict_cast(DataType::Int64)],
                Default::default(),
            )),
            WindowMapping::GroupsToRows,
        )?
        .lt_eq(lit(1));
    assert!(match_truncation_predicate(&ordered, &col("ID"))?.is_none());

    let ordinary = rank.over([col("ID")]).lt_eq(lit(1));
    assert!(match_truncation_predicate(&ordinary, &col("ID"))?.is_some());
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
    for enumeration in [ranks.clone(), random_enumeration().over([col("ID")])] {
        assert!(match_truncation_predicate(&enumeration.lt(lit(2.5f64)), &col("ID")).is_err());
    }
    for k in [0u32, 1, 2, 10, u32::MAX] {
        for (predicate, expected) in [
            (ranks.clone().lt(lit(k)), k.saturating_sub(1)),
            (ranks.clone().lt_eq(lit(k)), k),
        ] {
            let bound = match_truncation_predicate(&predicate, &col("ID"))?
                .expect("expected num_groups truncation");
            assert_eq!(bound[0].bound.num_groups, Some(expected));
        }
    }

    Ok(())
}

#[test]
fn test_match_per_group_predicate_enumerations() -> Fallible<()> {
    let check =
        |expr: Expr| match_per_group_predicate(&expr, &[col("A"), col("ID")], &col("ID"), 10);

    let range = int_range(lit(0), len(), 1, DataType::Int64);
    assert!(check(range.clone()).is_err());
    // Unsupported reorderings are not recognized as truncations.
    assert!(check(range.shuffle(None))?.is_none());
    Ok(())
}

#[test]
fn test_match_per_group_predicate_id() -> Fallible<()> {
    let range = random_enumeration();
    // no matches should error
    assert!(match_per_group_predicate(&range, &[col("A")], &col("ID"), 10).is_err());
    // multiple matches should pass
    assert!(match_per_group_predicate(&range, &[col("ID"), col("ID")], &col("ID"), 0)?.is_some());
    Ok(())
}

fn random_enumeration() -> Expr {
    int_range(lit(0), len(), 1, DataType::Int64).dp().permute()
}

#[test]
fn test_match_invalid_random_truncation_ranks() -> Fallible<()> {
    let tie = random_enumeration().alias("__tie");
    for (keys, method) in [
        (vec![col("A")], RankMethod::Ordinal),
        (vec![tie.clone(), col("A")], RankMethod::Ordinal),
        (vec![col("A"), tie], RankMethod::Dense),
    ] {
        let ranks = as_struct(keys).rank(
            RankOptions {
                method,
                descending: false,
            },
            None,
        );
        assert!(match_per_group_predicate(&ranks, &[col("ID")], &col("ID"), 1)?.is_none());
    }
    Ok(())
}

#[test]
fn test_ordinal_rank_key_dtypes() -> Fallible<()> {
    let categories = Categories::global();
    let categorical = DataType::Categorical(categories.clone(), categories.mapping());
    let schema = Schema::from_iter([
        Field::new("category".into(), categorical),
        Field::new("score".into(), DataType::Float64),
        Field::new("list".into(), DataType::List(Box::new(DataType::Int32))),
    ]);
    let range = int_range(lit(0), len(), 1, DataType::Int64);
    let rank = |key: Expr| {
        as_struct(vec![
            key.alias("__sort"),
            range.clone().dp().permute().alias("__tie"),
        ])
        .rank(
            RankOptions {
                method: RankMethod::Ordinal,
                descending: false,
            },
            None,
        )
    };

    let check = |key| -> Fallible<Bound> {
        match_per_group_predicate(&rank(key), &[col("ID")], &col("ID"), 1)?
            .expect("expected ordinal rank")
            .validate_ranking(&schema)
    };
    for key in [col("category"), col("list")] {
        assert!(check(key).is_err());
    }
    // Numeric priorities remain admitted, including floating-point priorities.
    assert!(check(col("score")).is_ok());
    Ok(())
}

#[cfg(patch_polars)]
#[test]
fn test_group_by_truncation_rejects_having() -> Fallible<()> {
    let mut plan = df!("ID" => [1i32], "A" => [2i32])?
        .lazy()
        .group_by([col("ID")])
        .agg([col("A").sum()])
        .logical_plan;
    let DslPlan::GroupBy { predicates, .. } = &mut plan else {
        panic!("expected group-by");
    };
    predicates.push(col("A").gt(lit(0)));
    assert!(match_group_by_truncation(&plan, &col("ID")).is_none());
    Ok(())
}

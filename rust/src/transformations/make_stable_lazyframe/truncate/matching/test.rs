#[cfg(feature = "ffi")]
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
            &col("ID"),
            None
        )?,
        Some(bounds.clone())
    );

    assert_eq!(
        match_truncation_predicate(
            &all_horizontal([truncate_num_groups.clone(), truncate_per_group.clone()])?,
            &col("ID"),
            None
        )?,
        Some(bounds)
    );

    assert!(
        match_truncation_predicate(
            &all_horizontal([truncate_num_groups.clone(), col("D").lt(4)])?,
            &col("ID"),
            None
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
            match_truncation_predicate(&expr, &col("ID"), None),
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

    let check =
        |expr: Expr| match_num_groups_predicate(&expr, &vec![col("ID")], &col("ID"), 10, None);
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

    // Extra struct fields affect ranking but must not disappear from the
    // certified grouping keys.
    let keys = as_struct(vec![col("A")]);
    let dense = |expr: Expr| {
        expr.rank(
            RankOptions {
                method: RankMethod::Dense,
                descending: false,
            },
            None,
        )
    };
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
        Field::new(
            "nested".into(),
            DataType::Array(
                Box::new(DataType::Categorical(
                    categories.clone(),
                    categories.mapping(),
                )),
                2,
            ),
        ),
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
        match_num_groups_predicate(&expr, &vec![col("ID")], &col("ID"), 1, Some(&schema))
    };

    assert!(check(rank(vec![col("category")])).is_err());
    assert!(check(rank(vec![as_struct(vec![col("category")])])).is_err());
    assert!(check(rank(vec![col("nested")])).is_err());
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
    assert!(match_truncation_predicate(&ordered, &col("ID"), None)?.is_none());

    let ordinary = rank.over([col("ID")]).lt_eq(lit(1));
    assert!(match_truncation_predicate(&ordinary, &col("ID"), None)?.is_some());
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
    for k in [0u32, 1, 2, 10, u32::MAX - 1] {
        for (predicate, expected) in [
            (ranks.clone().lt(lit(k)), k.saturating_sub(1)),
            (ranks.clone().lt_eq(lit(k)), k),
        ] {
            let bound = match_truncation_predicate(&predicate, &col("ID"), None)?
                .expect("expected num_groups truncation");
            assert_eq!(bound[0].num_groups, Some(expected));
        }
    }
    // <= u32::MAX needs a public k+1 value that does not fit in u32.
    assert!(match_truncation_predicate(&ranks.lt_eq(lit(u32::MAX)), &col("ID"), None).is_err());
    Ok(())
}

#[test]
fn test_match_per_group_predicate_enumerations() -> Fallible<()> {
    let check = |expr: Expr| {
        match_per_group_predicate(&expr, &vec![col("A"), col("ID")], &col("ID"), 10, None)
    };

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
    assert!(match_per_group_predicate(&range, &vec![col("A")], &col("ID"), 10, None).is_err());
    // multiple matches should pass
    assert!(match_per_group_predicate(&range, &vec![col("ID"); 2], &col("ID"), 0, None)?.is_some());
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
    let check =
        |expr: Expr| match_per_group_predicate(&expr, &vec![col("ID")], &col("ID"), 1, None);
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
        match_truncation_predicate(&malformed.over([col("ID")]).lt(lit(1)), &col("ID"), None)
            .is_err()
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
    assert_eq!(
        match_per_group_predicate(&trusted, &vec![col("ID")], &col("ID"), 2, None)?
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

    // Structural matching recognizes this rank, but semantic validation
    // rejects its positional key when the prior plan's schema is available.
    let positional = ranked(
        vec![range, random_enumeration().alias("__tie")],
        RankMethod::Ordinal,
    );
    assert!(check(positional.clone())?.is_some());
    let schema = Schema::from_iter([Field::new("A".into(), DataType::Int64)]);
    assert!(
        match_per_group_predicate(&positional, &vec![col("ID")], &col("ID"), 1, Some(&schema))
            .is_err()
    );

    let negated = ranked(
        vec![-col("A"), random_enumeration().alias("__tie")],
        RankMethod::Ordinal,
    );
    assert!(
        match_per_group_predicate(&negated, &vec![col("ID")], &col("ID"), 1, Some(&schema))?
            .is_some()
    );
    Ok(())
}

#[test]
fn test_ordinal_rank_rejects_categorical_sort_keys() -> Fallible<()> {
    use crate::polars::PrivacyNamespace;

    let categories = Categories::global();
    let categorical = DataType::Categorical(categories.clone(), categories.mapping());
    let schema = Schema::from_iter([
        Field::new("category".into(), categorical.clone()),
        Field::new("nested".into(), DataType::Array(Box::new(categorical), 2)),
        Field::new("score".into(), DataType::Float64),
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

    for key in [col("category"), col("nested")] {
        assert!(match_ordinal_rank(&rank(key), &range, Some(&schema)).is_err());
    }
    // Numeric priorities remain admitted, including floating-point priorities.
    assert!(match_ordinal_rank(
        &rank(col("score")),
        &range,
        Some(&schema)
    )?);
    Ok(())
}

#[test]
fn test_native_permutation_matches_without_ffi() -> Fallible<()> {
    use crate::polars::PrivacyNamespace;

    let permutation = int_range(lit(0), len(), 1, DataType::Int64).dp().permute();
    let by = vec![col("ID")];
    let bound = match_per_group_predicate(&permutation, &by, &col("ID"), 2, None)?
        .expect("native permutation should be trusted");
    assert_eq!(bound.per_group, Some(2));
    let max_bound = match_per_group_predicate(&permutation, &by, &col("ID"), u32::MAX, None)?
        .expect("trusted permutation should admit the largest u32 bound");
    assert_eq!(max_bound.per_group, Some(u32::MAX));

    let ranks = as_struct(vec![col("A").alias("__sort0"), permutation.alias("__tie")]).rank(
        RankOptions {
            method: RankMethod::Ordinal,
            descending: false,
        },
        None,
    );
    let bound = match_per_group_predicate(&ranks, &by, &col("ID"), 2, None)?
        .expect("native permutation must certify ordinal tie-breaking");
    assert_eq!(bound.per_group, Some(1));
    Ok(())
}

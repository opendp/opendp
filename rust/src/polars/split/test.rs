use super::*;
use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    measurements::{
        expr_noise::{NoiseDistribution, Support},
        make_private_lazyframe,
    },
    measures::{Approximate, MaxDivergence},
    metrics::SymmetricDistance,
    polars::{PrivacyNamespace, apply_anonymous_function},
};
use polars::prelude::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct WorkDir(PathBuf);
impl WorkDir {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        Self(std::env::temp_dir().join(format!(
            "opendp-split-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Both stages round-trip before IPC exists; restored sinks then populate the scan.
#[test]
fn ipc_before_materialization() -> Fallible<()> {
    let dir = WorkDir::new();
    let (heads, _) = split_plan(
        data()?.select([noisy(col("value").sum(), 0.)]).logical_plan,
        &dir.0,
    )?;
    assert!(!dir.0.exists());
    let ipc = sinks(&heads)[0].1.to_path_buf();
    let mut bytes = vec![];
    heads.serialize_versioned(&mut bytes, Default::default())?;
    let restored = DslPlan::deserialize_versioned(bytes.as_slice())?;
    assert_eq!(
        sinks(&restored)
            .iter()
            .map(|(_, path)| *path)
            .collect::<Vec<_>>(),
        sinks(&heads)
            .iter()
            .map(|(_, path)| *path)
            .collect::<Vec<_>>()
    );
    let mut bytes = vec![];
    ipc_scan(&ipc)?.serialize_versioned(&mut bytes, Default::default())?;
    let scan = DslPlan::deserialize_versioned(bytes.as_slice())?;
    assert!(!ipc.exists());
    assert!(!dir.0.exists());
    materialize(&restored, &dir.0)?;
    let result = LazyFrame::from(scan).collect()?;
    assert_eq!(result.shape(), (1, 1));
    assert_eq!(result.get_columns()[0].i32()?.get(0), Some(10));
    Ok(())
}

fn data() -> Fallible<LazyFrame> {
    Ok(df!("key" => [0i32, 0, 1], "value" => [1i32, 6, 3])?.lazy())
}

fn aggregate(input: &LazyFrame, exprs: impl IntoIterator<Item = Expr>, grouped: bool) -> LazyFrame {
    let exprs = exprs.into_iter().collect::<Vec<_>>();
    if grouped {
        input.clone().group_by([col("key")]).agg(exprs)
    } else {
        input.clone().select(exprs)
    }
}

fn test_domain(
    key: SeriesDomain,
    value: SeriesDomain,
    public_keys: bool,
) -> Fallible<LazyFrameDomain> {
    let by = Margin::by(["key"]).with_max_length(10);
    LazyFrameDomain::new(vec![key, value])?
        .with_margin(Margin::select().with_max_length(10))?
        .with_margin(if public_keys {
            by.with_invariant_keys()
        } else {
            by
        })
}

fn resolve_domain(
    input: &LazyFrame,
    query: LazyFrame,
    domain: LazyFrameDomain,
    threshold: Option<u32>,
) -> Fallible<DslPlan> {
    Ok(make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        query,
        Some(1.),
        threshold,
    )?
    .invoke(input)?
    .lazyframe()
    .logical_plan)
}

fn resolve(
    input: &LazyFrame,
    query: LazyFrame,
    public_keys: bool,
    threshold: Option<u32>,
) -> Fallible<DslPlan> {
    resolve_domain(
        input,
        query,
        test_domain(
            SeriesDomain::new("key", AtomDomain::<i32>::default()),
            SeriesDomain::new("value", AtomDomain::<i32>::default()),
            public_keys,
        )?,
        threshold,
    )
}

fn empty_data() -> Fallible<LazyFrame> {
    Ok(df!("key" => Vec::<i32>::new(), "value" => Vec::<i32>::new())?.lazy())
}

fn noisy(expr: Expr, scale: f64) -> Expr {
    apply_anonymous_function(
        vec![expr],
        NoisePlugin {
            distribution: NoiseDistribution::Laplace,
            scale,
            support: Support::Integer,
        },
    )
}

fn sum() -> Expr {
    col("value")
        .dp()
        .sum((lit(0), lit(5)), Some(0.))
        .alias("sum")
}
fn count() -> Expr {
    len().dp().noise(Some(0.)).alias("count")
}
fn mean() -> Expr {
    col("value")
        .dp()
        .mean((lit(0), lit(5)), Some(0.))
        .alias("mean")
}

fn walk(plan: &DslPlan, f: &mut impl FnMut(&DslPlan)) {
    f(plan);
    match plan {
        DslPlan::Select { input, .. }
        | DslPlan::GroupBy { input, .. }
        | DslPlan::Filter { input, .. }
        | DslPlan::HStack { input, .. }
        | DslPlan::Distinct { input, .. }
        | DslPlan::MapFunction { input, .. }
        | DslPlan::IR { dsl: input, .. }
        | DslPlan::Sink { input, .. } => walk(input, f),
        DslPlan::SinkMultiple { inputs } => {
            for input in inputs {
                walk(input, f);
            }
        }
        DslPlan::Join {
            input_left,
            input_right,
            ..
        } => {
            walk(input_left, f);
            walk(input_right, f);
        }
        _ => {}
    }
}

fn expressions(plan: &DslPlan) -> &[Expr] {
    match plan {
        DslPlan::Select { expr, .. }
        | DslPlan::GroupBy { aggs: expr, .. }
        | DslPlan::HStack { exprs: expr, .. } => expr,
        DslPlan::Filter { predicate, .. } => std::slice::from_ref(predicate),
        _ => &[],
    }
}

fn noise_count(plan: &DslPlan) -> usize {
    noise_parameters(plan).len()
}

fn noise_parameters(plan: &DslPlan) -> Vec<NoisePlugin> {
    let mut parameters = vec![];
    walk(plan, &mut |node| {
        let exprs = expressions(node);
        parameters.extend(
            exprs
                .iter()
                .flat_map(|expr| expr.into_iter())
                .filter_map(|expr| {
                    match_trusted_plugin::<NoisePlugin>(expr)
                        .unwrap()
                        .map(|(_, args)| args)
                }),
        );
    });
    parameters
}

/// Inspect native sinks, also asserting their format and distinct local paths.
fn sinks(plan: &DslPlan) -> Vec<(&DslPlan, &Path)> {
    let DslPlan::SinkMultiple { inputs } = plan else {
        panic!("expected SinkMultiple")
    };
    let sinks: Vec<_> = inputs
        .iter()
        .map(|head| {
            let DslPlan::Sink {
                input,
                payload: SinkType::File(options),
            } = head
            else {
                panic!("expected file sink")
            };
            assert!(matches!(options.file_format.as_ref(), FileType::Ipc(_)));
            let SinkTarget::Path(PlPath::Local(path)) = &options.target else {
                panic!("expected local path")
            };
            (input.as_ref(), path.as_ref())
        })
        .collect();
    let unique: HashSet<_> = sinks.iter().map(|(_, path)| *path).collect();
    assert_eq!(unique.len(), sinks.len());
    sinks
}

fn base(plan: &DslPlan) -> &DslPlan {
    match sinks(plan)[0].0 {
        DslPlan::Select { input, .. } | DslPlan::GroupBy { input, .. } => input,
        _ => panic!("expected private aggregate head"),
    }
}

/// Execute actual native IPC sinks using Rust Polars only as a test oracle.
fn materialize(deterministic: &DslPlan, dir: &Path) -> Fallible<()> {
    fs::create_dir_all(dir).unwrap();
    let DslPlan::SinkMultiple { inputs } = deterministic else {
        panic!()
    };
    LazyFrame::collect_all_with_engine(inputs.clone(), Engine::InMemory, Default::default())?;
    Ok(())
}

fn public_join(private: LazyFrame, public: LazyFrame, key: &str, public_left: bool) -> LazyFrame {
    if public_left {
        public.join(private, [col(key)], [col(key)], JoinType::Left.into())
    } else {
        private.join(public, [col(key)], [col(key)], JoinType::Right.into())
    }
}

fn collect_sorted(plan: DslPlan, key: Option<&str>) -> Fallible<DataFrame> {
    let lf = LazyFrame::from(plan);
    Ok(match key {
        Some(key) => lf.sort([key], Default::default()),
        None => lf,
    }
    .collect()?)
}

fn compare(plan: DslPlan, expected_heads: usize, grouped: bool) -> Fallible<(DslPlan, DslPlan)> {
    let dir = WorkDir::new();
    let (deterministic, continuation) = split_plan(plan.clone(), &dir.0)?;
    assert!(
        !dir.0.exists(),
        "splitter must not create the working directory"
    );
    assert_eq!(sinks(&deterministic).len(), expected_heads);
    assert_eq!(noise_count(&deterministic), 0);
    assert_eq!(noise_parameters(&continuation), noise_parameters(&plan));
    assert_eq!(
        noisy_max_parameters(&continuation),
        noisy_max_parameters(&plan)
    );
    assert!(noisy_max_parameters(&deterministic).is_empty());
    walk(&continuation, &mut |node| {
        assert!(!matches!(node, DslPlan::DataFrameScan { .. }))
    });
    materialize(&deterministic, &dir.0)?;
    let key = grouped.then_some("key");
    assert_eq!(
        collect_sorted(continuation.clone(), key)?,
        collect_sorted(plan, key)?
    );
    Ok((deterministic, continuation))
}

#[test]
fn ungrouped_sum_and_private_select() -> Fallible<()> {
    let input = data()?;
    let (deterministic, _) = compare(
        resolve(&input, input.clone().select([sum()]), false, None)?,
        1,
        false,
    )?;
    let DslPlan::Select { expr, .. } = sinks(&deterministic)[0].0 else {
        panic!()
    };
    assert_eq!(expr.len(), 1);
    assert!(
        expr[0]
            .into_iter()
            .any(|e| matches!(e, Expr::Agg(AggExpr::Sum(_))))
    );
    let query = input
        .clone()
        .select([count(), (sum() + lit(1)).alias("plus_one")]);
    compare(resolve(&input, query, false, None)?, 1, false)?;
    Ok(())
}

#[test]
fn mean_and_compatible_aggregates_share_fragment() -> Fallible<()> {
    let input = data()?;
    for exprs in [vec![mean()], vec![sum(), count(), mean()]] {
        let plan = resolve(&input, input.clone().select(exprs), false, None)?;
        let (deterministic, continuation) = compare(plan, 1, false)?;
        let DslPlan::Select { expr, .. } = sinks(&deterministic)[0].0 else {
            panic!()
        };
        assert_eq!(expr.len(), noise_count(&continuation));
        let DslPlan::Select {
            input, expr: outer, ..
        } = &continuation
        else {
            panic!()
        };
        assert!(!outer.iter().flat_map(|e| e.into_iter()).any(|e| matches!(
            e,
            Expr::BinaryExpr {
                op: Operator::TrueDivide,
                ..
            }
        )));
        let DslPlan::Select { expr, .. } = input.as_ref() else {
            panic!()
        };
        assert!(expr.iter().flat_map(|e| e.into_iter()).any(|e| matches!(
            e,
            Expr::BinaryExpr {
                op: Operator::TrueDivide,
                ..
            }
        )));
    }
    Ok(())
}

#[test]
fn public_lengths_remain_in_the_original_aggregation_context() -> Fallible<()> {
    let input = data()?;
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        Margin::select()
            .with_max_length(10)
            .with_invariant_lengths(),
    )?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_invariant_lengths(),
    )?;
    for grouped in [false, true] {
        let exprs = [len().alias("exact_count"), sum()];
        let query = aggregate(&input, exprs, grouped);
        let plan = resolve_domain(&input, query, domain.clone(), None)?;
        compare(plan, 1, grouped)?;
    }
    Ok(())
}

#[test]
fn grouped_count_and_threshold() -> Fallible<()> {
    let input = data()?;
    for threshold in [None, Some(1)] {
        let query = input.clone().group_by([col("key")]).agg([count()]);
        let (deterministic, continuation) = compare(
            resolve(&input, query, threshold.is_none(), threshold)?,
            1,
            true,
        )?;
        let DslPlan::GroupBy { keys, .. } = sinks(&deterministic)[0].0 else {
            panic!()
        };
        assert_eq!(keys, &[col("key")]);
        if threshold.is_some() {
            assert!(matches!(&continuation, DslPlan::Filter { .. }));
            assert!(!matches!(
                sinks(&deterministic)[0].0,
                DslPlan::Filter { .. }
            ));
        }
    }
    Ok(())
}

fn assert_collision_safe(
    plan: DslPlan,
    expected_heads: usize,
    key: Option<&str>,
    reserved: &[&str],
) -> Fallible<DslPlan> {
    let dir = WorkDir::new();
    let (heads, continuation) = split_plan(plan.clone(), &dir.0)?;
    let mut schemas = vec![];
    let mut generated = HashSet::new();
    assert!(!dir.0.exists());
    assert_eq!(sinks(&heads).len(), expected_heads);
    for (head, path) in sinks(&heads) {
        assert_eq!(path.parent(), Some(dir.0.as_path()));
        // Private aliases and histogram intermediates must be globally unique
        // and disjoint from all input/public/downstream reserved names.
        let exprs = match head {
            DslPlan::Select { expr, .. } => expr.clone(),
            DslPlan::GroupBy { keys, aggs, .. } => keys
                .iter()
                .skip(usize::from(key.is_some()))
                .chain(aggs)
                .cloned()
                .collect(),
            _ => vec![], // public head keeps its original schema
        };
        for expr in exprs {
            let name = expr.meta().output_name()?;
            assert!(!reserved.contains(&name.as_str()), "reserved: {name}");
            assert!(generated.insert(name), "generated aliases must be unique");
        }
        let schema: HashSet<_> = LazyFrame::from(head.clone())
            .collect_schema()?
            .iter_names()
            .cloned()
            .collect();
        for previous in &schemas {
            assert_eq!(
                schema
                    .intersection(previous)
                    .map(|n| n.as_str())
                    .collect::<Vec<_>>(),
                key.into_iter().collect::<Vec<_>>()
            );
        }
        schemas.push(schema);
    }
    materialize(&heads, &dir.0)?;
    assert_eq!(
        collect_sorted(continuation, key)?,
        collect_sorted(plan, key)?
    );
    Ok(heads)
}

#[test]
fn names_are_reserved_across_preprocessing_heads_and_join_branches() -> Fallible<()> {
    let input = data()?;
    let query = input
        .clone()
        .filter(col("value").gt(lit(1)))
        .with_column(col("value").cast(DataType::Int64))
        .select([col("value")
            .dp()
            .sum((lit(0i64), lit(5i64)), Some(0.))
            .alias("__opendp_split_0")]);
    let heads = assert_collision_safe(
        resolve(&input, query, false, None)?,
        1,
        None,
        &["__opendp_split_0"],
    )?;
    assert!(matches!(base(&heads), DslPlan::HStack { .. }));

    let key = "__opendp_split_0";
    let input = df!(key => [0i32, 1], "value" => [2i32, 3])?.lazy();
    let private = input.group_by([col(key)]).agg([
        noisy(col("value").sum(), 0.).alias("__opendp_split_1"),
        noisy(len().cast(DataType::Int64), 0.).alias("count"),
    ]);
    let public = df!(key => [1i32, 0], "__opendp_split_2" => [7i32, 8])?.lazy();
    let reserved = [
        key,
        "__opendp_split_1",
        "__opendp_split_2",
        "__opendp_split_3",
    ];
    for left in [true, false] {
        let joined = public_join(private.clone(), public.clone(), key, left).select([
            col(key),
            col("__opendp_split_1").alias("__opendp_split_3"),
            col("__opendp_split_2"),
            col("count"),
        ]);
        assert_collision_safe(joined.logical_plan, 2, Some(key), &reserved)?;
    }

    let input =
        df!("key" => [0i32, 0, 1], "value" => [1i32, 6, 3], "__opendp_split_0" => [1,2,3])?.lazy();
    let private = input.clone().group_by([col("key")]).agg([
        quantile().alias("__opendp_split_1"),
        sum().alias("__opendp_split_2"),
        quantile().alias("__opendp_split_3"),
    ]);
    let public = df!("key" => [0i32, 1], "__opendp_split_4" => [4,5])?.lazy();
    for left in [true, false] {
        let query = public_join(private.clone(), public.clone(), "key", left);
        assert_collision_safe(
            resolve(&input, query, false, None)?,
            4,
            Some("key"),
            &[
                "__opendp_split_0",
                "__opendp_split_1",
                "__opendp_split_2",
                "__opendp_split_3",
                "__opendp_split_4",
            ],
        )?;
    }
    Ok(())
}

fn callback_expr() -> Expr {
    Expr::Function {
        input: vec![col("value")],
        function: FunctionExpr::ReduceHorizontal {
            callback: polars_plan::prelude::PlanCallback::Rust(polars_plan::dsl::SpecialEq::new(
                Arc::new(|(left, _)| Ok(left)),
            )),
            returns_scalar: false,
            return_dtype: None,
        },
    }
}

#[test]
fn opaque_bases_and_histogram_inputs_are_preserved_verbatim() -> Fallible<()> {
    let input = data()?;
    let mut bases = [
        ("callback", callback_expr()),
        ("drop nulls", col("value").drop_nulls()),
        ("drop nans", col("value").drop_nans()),
    ]
    .map(|(name, expr)| {
        (
            name,
            input.clone().with_column(expr.alias("value")).logical_plan,
        )
    })
    .to_vec();
    bases.push((
        "union",
        DslPlan::Union {
            inputs: vec![input.logical_plan.clone(), input.logical_plan.clone()],
            args: Default::default(),
        },
    ));
    bases.push(("filter", input.filter(col("value").gt(lit(1))).logical_plan));
    #[cfg(feature = "polars-ffi")]
    bases.push((
        "unknown FFI",
        data()?
            .with_column(
                Expr::Function {
                    input: vec![col("value")],
                    function: FunctionExpr::FfiPlugin {
                        lib: "unknown-library".into(),
                        symbol: "unknown-plugin".into(),
                        kwargs: vec![1, 2, 3].into(),
                        flags: Default::default(),
                    },
                }
                .alias("value"),
            )
            .logical_plan,
    ));
    for (name, base) in bases.drain(..) {
        let base = Arc::new(base);
        for histogram in [false, true] {
            let expression = if histogram {
                apply_anonymous_function(
                    vec![apply_anonymous_function(
                        vec![col("value")],
                        DiscreteQuantileScorePlugin {
                            candidates: Series::new("".into(), [1i32, 3, 6]),
                            alpha: (1, 2),
                            size_limit: 10,
                        },
                    )],
                    NoisyMaxPlugin {
                        replacement: false,
                        scale: 0.,
                        negate: true,
                    },
                )
            } else {
                noisy(col("value").sum(), 0.)
            };
            let key = (col("value") + lit(1)).alias("derived");
            let mut plan = aggregate(&data()?, [expression], histogram).logical_plan;
            match &mut plan {
                DslPlan::Select { input, .. } => *input = base.clone(),
                DslPlan::GroupBy { input, keys, .. } => {
                    *input = base.clone();
                    *keys = vec![key.clone()];
                }
                _ => panic!(),
            }
            let dir = WorkDir::new();
            let (heads, _) = split_plan(plan, &dir.0)?;
            let mut actual = self::base(&heads);
            if histogram {
                let DslPlan::GroupBy { keys, .. } = sinks(&heads)[0].0 else {
                    panic!()
                };
                assert_eq!(keys[0], key);
                for _ in 0..2 {
                    let DslPlan::HStack { input, .. } = actual else {
                        panic!()
                    };
                    actual = input;
                }
            }
            assert!(
                std::ptr::eq(actual, base.as_ref()),
                "{name}, histogram={histogram}"
            );
            assert!(!dir.0.exists());
        }
    }
    Ok(())
}

#[test]
fn contribution_bounding_and_derived_keys_stay_upstream() -> Fallible<()> {
    let input = data()?;
    let base = input.filter(
        col("value")
            .rank(
                RankOptions {
                    method: RankMethod::Dense,
                    descending: false,
                },
                None,
            )
            .over([col("key")])
            .lt_eq(lit(1)),
    );
    let query = base
        .clone()
        .group_by([col("key").cast(DataType::Int64)])
        .agg([noisy(col("value").sum(), 0.).alias("sum")]);
    let (deterministic, _) = compare(query.logical_plan, 1, true)?;
    assert_eq!(
        format!("{:?}", self::base(&deterministic)),
        format!("{:?}", base.logical_plan)
    );
    Ok(())
}

#[test]
fn null_keys_and_float_aggregation() -> Fallible<()> {
    let input = df!("key" => [Some(0i32), None, None], "value" => [1f64, 2., 3.])?.lazy();
    let expr = apply_anonymous_function(
        vec![col("value").sum()],
        NoisePlugin {
            distribution: NoiseDistribution::Gaussian,
            scale: 0.,
            support: Support::Float,
        },
    );
    compare(
        input
            .group_by([col("key")])
            .agg([expr.alias("sum")])
            .logical_plan,
        1,
        true,
    )?;
    Ok(())
}

#[test]
fn empty_and_null_aggregates() -> Fallible<()> {
    // Already-resolved expressions let us check Polars' null/empty aggregation
    // semantics without imposing a different input-domain null policy.
    for values in [vec![], vec![None, None], vec![Some(2i32), None, Some(5)]] {
        let input = df!("value" => values)?.lazy();
        let plan = input
            .select([
                noisy(col("value").sum(), 0.).alias("sum"),
                noisy(col("value").count().cast(DataType::Int64), 0.).alias("count"),
                noisy(col("value").null_count().cast(DataType::Int64), 0.).alias("null_count"),
                noisy(col("value").n_unique().cast(DataType::Int64), 0.).alias("n_unique"),
            ])
            .logical_plan;
        compare(plan, 1, false)?;
    }
    let input = empty_data()?;
    let plan = resolve(
        &input,
        input.clone().group_by([col("key")]).agg([count(), sum()]),
        true,
        None,
    )?;
    compare(plan, 1, true)?;
    Ok(())
}

#[test]
fn randomized_occurrences_are_not_deduplicated() -> Fallible<()> {
    let input = data()?;
    let plan = input
        .select([
            noisy(len().cast(DataType::Int64), 1.).alias("a"),
            noisy(len().cast(DataType::Int64), 1.).alias("b"),
        ])
        .logical_plan;
    let dir = WorkDir::new();
    let (deterministic, continuation) = split_plan(plan, &dir.0)?;
    assert_eq!(noise_count(&continuation), 2);
    materialize(&deterministic, &dir.0)?;
    let output = LazyFrame::from(continuation).collect()?;
    assert_eq!(output.shape(), (1, 2));
    Ok(())
}

#[test]
fn literal_only_releases_preserve_cardinality() -> Fallible<()> {
    for input in [
        df!("key" => [0i32, 1])?.lazy(),
        df!("key" => Vec::<i32>::new())?.lazy(),
    ] {
        for grouped in [false, true] {
            let expression = noisy((lit(0i64) + lit(1i64)).cast(DataType::Int64), 0.).alias("n");
            let query = aggregate(&input, [expression], grouped);
            compare(query.logical_plan, 1, grouped)?;
        }
    }
    Ok(())
}

#[test]
fn literal_noise_keeps_scalar_broadcasting() -> Fallible<()> {
    let input = df!("key" => (0..100i32).collect::<Vec<_>>())?.lazy();
    let plan = input
        .group_by([col("key")])
        .agg([
            noisy(
                (lit(0i64) + lit(1i64))
                    .cast(DataType::Int64)
                    .alias("constant"),
                10.,
            )
            .alias("n"),
            noisy(len().cast(DataType::Int64), 0.).alias("count"),
        ])
        .logical_plan;
    assert_eq!(
        LazyFrame::from(plan.clone())
            .collect()?
            .column("n")?
            .n_unique()?,
        1
    );
    let dir = WorkDir::new();
    let (deterministic, continuation) = split_plan(plan, &dir.0)?;
    let DslPlan::GroupBy { aggs, .. } = sinks(&deterministic)[0].0 else {
        panic!()
    };
    assert_eq!(aggs.len(), 1, "literal input must not become a column");
    materialize(&deterministic, &dir.0)?;
    let output = LazyFrame::from(continuation).collect()?;
    assert_eq!(output.column("n")?.n_unique()?, 1);
    Ok(())
}

#[test]
fn deterministic_noise_inputs_are_preserved_without_an_operator_whitelist() -> Fallible<()> {
    for expression in [
        (col("value") * lit(2)).sum().abs(),
        col("value").product(),
        col("value").sum().clip(lit(0), lit(5)),
        callback_expr().sum(), // an opaque callback inside the aggregate
    ] {
        for grouped in [false, true] {
            let input = data()?;
            let release = noisy(expression.clone(), 0.).alias("release");
            let query = aggregate(&input, [release], grouped);
            let (deterministic, continuation) = compare(query.logical_plan, 1, grouped)?;
            assert_eq!(noise_count(&continuation), 1);
            let exprs = match sinks(&deterministic)[0].0 {
                DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => expr,
                _ => panic!(),
            };
            let Expr::Alias(actual, _) = &exprs[0] else {
                panic!()
            };
            assert_eq!(actual.as_ref(), &expression);
        }
    }
    // Pointwise postprocessing of noise is kept without an operator list too.
    for grouped in [false, true] {
        let input = data()?;
        let release = noisy(col("value").sum(), 0.)
            .abs()
            .clip(lit(0), lit(5))
            .alias("release");
        let query = aggregate(&input, [release], grouped);
        compare(query.logical_plan, 1, grouped)?;
    }
    // Gather is postprocessing in the original context, not something the
    // splitter needs to whitelist for either SQL or scalar extraction.
    let input = data()?;
    let resolved = resolve(&input, input.clone().select([sum()]), false, None)?;
    let postprocessed = LazyFrame::from(resolved).select([col("sum").gather(lit(0)).alias("out")]);
    compare(postprocessed.logical_plan, 1, false)?;
    Ok(())
}

#[test]
fn unsupported_plan_structures_fail_explicitly() -> Fallible<()> {
    let input = data()?;
    let plans = [
        input.clone().select([col("value").sum()]).logical_plan, // no private boundary
        input
            .clone()
            .select([noisy(len(), 1.)])
            .sort(["value"], Default::default())
            .logical_plan,
        input
            .clone()
            .select([noisy(len(), 1.)])
            .join(
                input.clone().select([noisy(len(), 1.)]),
                [col("len")],
                [col("len")],
                JoinType::Inner.into(),
            )
            .logical_plan,
    ];
    for plan in plans {
        let dir = WorkDir::new();
        assert!(split_plan(plan, &dir.0).is_err());
        assert!(!dir.0.exists());
    }
    Ok(())
}

fn noisy_max_parameters(plan: &DslPlan) -> Vec<(bool, u64, bool)> {
    let mut out = vec![];
    walk(plan, &mut |node| {
        let exprs = expressions(node);
        out.extend(
            exprs
                .iter()
                .flat_map(|expr| expr.into_iter())
                .filter_map(|expr| {
                    match_trusted_plugin::<NoisyMaxPlugin>(expr)
                        .unwrap()
                        .map(|(_, p)| (p.replacement, p.scale.to_bits(), p.negate))
                }),
        );
    });
    out.sort();
    out
}

fn argmax(score: Expr) -> Expr {
    apply_anonymous_function(
        vec![score],
        NoisyMaxPlugin {
            replacement: false,
            scale: 0.,
            negate: true,
        },
    )
}

fn quantile() -> Expr {
    col("value")
        .dp()
        .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))
        .alias("quantile")
}

#[test]
fn quantiles_medians_and_scalar_combinations() -> Fallible<()> {
    let input = data()?;
    for grouped in [false, true] {
        for (exprs, expected) in [
            (vec![quantile()], 1),
            (
                vec![
                    col("value")
                        .dp()
                        .median(Series::new("".into(), [1i32, 3, 6]), Some(0.))
                        .alias("median"),
                ],
                1,
            ),
            (vec![quantile(), sum(), count(), mean()], 2),
            (
                vec![
                    quantile(),
                    col("value")
                        .dp()
                        .quantile(0.5, Series::new("".into(), [0i32, 1, 3, 6, 7]), Some(0.))
                        .alias("other"),
                ],
                2,
            ),
            (
                vec![
                    (quantile() + lit(2)).alias("offset"),
                    (quantile() - mean()).alias("difference"),
                ],
                3,
            ),
        ] {
            let query = aggregate(&input, exprs, grouped);
            let plan = resolve(&input, query, grouped, None)?;
            let base = match &plan {
                DslPlan::Select { input, .. } | DslPlan::GroupBy { input, .. } => input.clone(),
                _ => panic!(),
            };
            let (deterministic, _) = compare(plan, expected, grouped)?;
            // All deterministic heads retain the exact same analyzed base.
            for (head, _) in sinks(&deterministic) {
                let mut found = false;
                walk(head, &mut |node| {
                    if std::ptr::eq(node, base.as_ref()) {
                        found = true;
                    }
                });
                assert!(found);
            }
        }
    }
    Ok(())
}

#[test]
fn postprocessing_is_placed_by_head_dependencies() -> Fallible<()> {
    let input = data()?;
    let plan = resolve(
        &input,
        input
            .clone()
            .select([(quantile() - mean()).alias("difference")]),
        false,
        None,
    )?;
    let (_, continuation) = compare(plan, 2, false)?;
    let DslPlan::Select { input, expr, .. } = continuation else {
        panic!()
    };
    assert!(expr.iter().flat_map(|e| e.into_iter()).any(|e| matches!(
        e,
        Expr::BinaryExpr {
            op: Operator::Minus,
            ..
        }
    )));
    let DslPlan::Join {
        input_left,
        input_right,
        ..
    } = input.as_ref()
    else {
        panic!()
    };
    let mut divides = 0;
    let mut lookups = 0;
    for local in [input_left, input_right] {
        let DslPlan::Select { expr, .. } = local.as_ref() else {
            panic!()
        };
        for node in expr.iter().flat_map(|e| e.into_iter()) {
            divides += usize::from(matches!(
                node,
                Expr::BinaryExpr {
                    op: Operator::TrueDivide,
                    ..
                }
            ));
            lookups += usize::from(
                match_trusted_plugin::<
                    crate::measurements::expr_index_candidates::IndexCandidatesPlugin,
                >(node)?
                .is_some(),
            );
            assert!(!matches!(
                node,
                Expr::BinaryExpr {
                    op: Operator::Minus,
                    ..
                }
            ));
        }
    }
    assert_eq!((divides, lookups), (1, 1));
    Ok(())
}

#[test]
fn empty_quantile_inputs_and_public_missing_groups() -> Fallible<()> {
    // A singleton candidate avoids random tie-breaking when every score is zero.
    let q = || {
        col("value")
            .dp()
            .quantile(0.5, Series::new("".into(), [3i32]), Some(0.))
            .alias("quantile")
    };
    for input in [data()?, empty_data()?] {
        compare(
            resolve(
                &input,
                input.clone().select([q(), sum(), count()]),
                false,
                None,
            )?,
            2,
            false,
        )?;
        compare(
            resolve(
                &input,
                input.clone().group_by([col("key")]).agg([q(), count()]),
                true,
                None,
            )?,
            2,
            true,
        )?;
        let mut releases = vec![(vec![q(), sum(), count(), mean()], 3)];
        if input.clone().collect()?.height() > 0 {
            releases.push((vec![sum(), count(), mean()], 2));
        }
        for (exprs, expected_heads) in releases {
            for public_left in [true, false] {
                let private = aggregate(&input, exprs.clone(), true);
                let public = df!("key" => [2i32, 0, 2, 1])?.lazy();
                let query = public_join(private, public, "key", public_left);
                let (_, continuation) =
                    compare(resolve(&input, query, false, None)?, expected_heads, true)?;
                let mut dedup = 0;
                let mut public_joins = 0;
                walk(&continuation, &mut |node| {
                    dedup += usize::from(matches!(node, DslPlan::Distinct { .. }));
                    if let DslPlan::Join { options, .. } = node {
                        public_joins += usize::from(matches!(
                            options.args.how,
                            JoinType::Left | JoinType::Right
                        ));
                    }
                });
                assert_eq!((dedup, public_joins), (1, 1));
                assert!(matches!(continuation, DslPlan::HStack { .. }));
            }
        }
    }
    Ok(())
}

#[test]
fn multiple_heads_share_one_threshold() -> Fallible<()> {
    let input = data()?;
    let query = input
        .clone()
        .group_by([col("key")])
        .agg([quantile(), sum(), count(), mean()]);
    let (_, continuation) = compare(resolve(&input, query, false, Some(1))?, 2, true)?;
    let mut filters = 0;
    walk(&continuation, &mut |node| {
        filters += usize::from(matches!(node, DslPlan::Filter { .. }));
    });
    assert_eq!(filters, 1);
    assert!(matches!(continuation, DslPlan::Filter { .. }));
    Ok(())
}

#[test]
fn quantile_null_keys_and_null_nan_preprocessing() -> Fallible<()> {
    use crate::domains::OptionDomain;
    let input = df!("key" => [None, None, Some(1i32)], "value" => [1i32, 6, 3])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", OptionDomain::new(AtomDomain::<i32>::default())),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_invariant_keys(),
    )?;
    let query = input
        .clone()
        .group_by([col("key")])
        .agg([quantile(), mean()]);
    let plan = resolve_domain(&input, query, domain, None)?;
    compare(plan, 2, true)?;
    let input =
        df!("key" => [0i32; 5], "value" => [Some(1f64), None, Some(f64::NAN), Some(3.), Some(6.)])?
            .lazy();
    let domain = test_domain(
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", OptionDomain::new(AtomDomain::<f64>::default())),
        true,
    )?;
    for grouped in [false, true] {
        let q = col("value")
            .dp()
            .quantile(0.5, Series::new("".into(), [1f64, 3., 6.]), Some(0.))
            .alias("q");
        let query = aggregate(&input, [q], grouped);
        let plan = resolve_domain(&input, query, domain.clone(), None)?;
        compare(plan, 1, grouped)?;
    }
    Ok(())
}

#[test]
fn histogram_heads_preserve_the_resolved_scorer_contract() -> Fallible<()> {
    let input = data()?;
    let plan = resolve(
        &input,
        input.clone().group_by([col("key")]).agg([quantile()]),
        true,
        None,
    )?;
    let DslPlan::GroupBy {
        input: original_base,
        aggs,
        ..
    } = &plan
    else {
        panic!()
    };
    let (original_inputs, original) = aggs
        .iter()
        .flat_map(|e| e.into_iter())
        .find_map(|e| score_plugin(e).unwrap())
        .unwrap();
    let dir = WorkDir::new();
    let (heads, continuation) = split_plan(plan.clone(), &dir.0)?;
    let DslPlan::GroupBy {
        input: buckets,
        keys,
        aggs,
        ..
    } = sinks(&heads)[0].0
    else {
        panic!()
    };
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], col("key"));
    assert!(aggs[0].into_iter().any(|e| matches!(e, Expr::Len)));
    let DslPlan::HStack {
        input: values,
        exprs,
        ..
    } = buckets.as_ref()
    else {
        panic!()
    };
    assert!(
        exprs[0]
            .into_iter()
            .any(|e| matches!(e, Expr::Ternary { .. }))
    );
    let DslPlan::HStack {
        input: base, exprs, ..
    } = values.as_ref()
    else {
        panic!()
    };
    let Expr::Alias(projected, _) = &exprs[0] else {
        panic!()
    };
    assert_eq!(
        projected.as_ref(),
        &original_inputs[0]
            .clone()
            .cast(original.candidates.dtype().clone())
    );
    assert!(Arc::ptr_eq(base, original_base));
    let mut found = 0;
    walk(&continuation, &mut |node| {
        let exprs = expressions(node);
        for e in exprs.iter().flat_map(|e| e.into_iter()) {
            if let Some((_, scorer)) =
                match_trusted_plugin::<HistogramQuantileScorePlugin>(e).unwrap()
            {
                assert_eq!(scorer.num_candidates, original.candidates.len());
                assert_eq!(scorer.alpha, original.alpha);
                assert_eq!(scorer.size_limit, original.size_limit);
                found += 1;
            }
        }
    });
    assert_eq!(found, 1);
    assert!(!dir.0.exists());
    Ok(())
}

#[test]
fn native_histograms_reproduce_original_score_arrays_exactly() -> Fallible<()> {
    let mut cases: Vec<(Series, Series)> = vec![];
    for candidates in [vec![0i64], vec![-3, 0, 2], vec![-4, -1, 0, 1, 5, 9]] {
        cases.push((
            Series::new(
                "value".into(),
                [-8i64, -4, -4, -3, -2, -1, 0, 0, 1, 2, 3, 5, 9, 12],
            ),
            Series::new("".into(), candidates),
        ));
    }
    cases.extend([
        (
            Series::new(
                "value".into(),
                [0u64, 1 << 53, (1 << 53) + 1, u64::MAX - 1, u64::MAX],
            ),
            Series::new("".into(), [1 << 53, (1 << 53) + 1, u64::MAX - 1]),
        ),
        (
            Series::new("value".into(), [i64::MIN, i64::MIN + 1, -1, 0, i64::MAX]),
            Series::new("".into(), [i64::MIN + 1, 0, i64::MAX]),
        ),
        (
            Series::new(
                "value".into(),
                [f64::NEG_INFINITY, -1., -0., 0., 1., f64::INFINITY],
            ),
            Series::new("".into(), [-1f64, 0., 1.]),
        ),
        (
            Series::new(
                "value".into(),
                [f32::NEG_INFINITY, -1., -0., 0., 1., f32::INFINITY],
            ),
            Series::new("".into(), [-1f32, 0., 1.]),
        ),
        (
            Series::new("value".into(), Vec::<i64>::new()),
            Series::new("".into(), [-1i64, 0, 1]),
        ),
        (
            Series::new(
                "value".into(),
                [Some(-1f64), None, Some(f64::NAN), Some(1.)],
            ),
            Series::new("".into(), [-1f64, 0., 1.]),
        ),
    ]);
    for dtype in [DataType::Int8, DataType::Int16, DataType::Int32] {
        cases.push((
            Series::new("value".into(), [-8i64, -3, -1, 0, 0, 1, 2, 12]).cast(&dtype)?,
            Series::new("".into(), [-3i64, 0, 2]).cast(&dtype)?,
        ));
    }
    cases.push((
        Series::new("value".into(), [0u32, 1, 2, 2, 3, u32::MAX]),
        Series::new("".into(), [0u32, 2, u32::MAX]),
    ));
    assert_eq!(cases.len(), 13);
    let mut constructions = 0;
    let mut comparisons = 0;
    for (values, candidates) in cases {
        let input = DataFrame::new(vec![values.clone().into_column()])?.lazy();
        let mut expr = col("value").fill_null(lit(0));
        if values.dtype().is_float() {
            expr = expr.fill_nan(lit(0));
        }
        let scorer = DiscreteQuantileScorePlugin {
            candidates: candidates.clone(),
            alpha: (2, 7),
            size_limit: 1,
        };
        let scores = apply_anonymous_function(vec![expr.clone()], scorer);
        let query = input.clone().select([argmax(scores)]);
        let dir = WorkDir::new();
        let (heads, continuation) = split_plan(query.logical_plan, &dir.0)?;
        assert_eq!(sinks(&heads).len(), 1);
        let mut reconstructed = 0;
        walk(&continuation, &mut |node| {
            for expr in expressions(node).iter().flat_map(|expr| expr.into_iter()) {
                if let Some((_, scorer)) =
                    match_trusted_plugin::<HistogramQuantileScorePlugin>(expr).unwrap()
                {
                    assert_eq!((scorer.alpha, scorer.size_limit), ((2, 7), 1));
                    assert_eq!(scorer.num_candidates, candidates.len());
                    reconstructed += 1;
                }
            }
        });
        assert_eq!(reconstructed, 1);
        materialize(&heads, &dir.0)?;
        let (head, path) = sinks(&heads)[0];
        let DslPlan::GroupBy { keys, aggs, .. } = head else {
            panic!()
        };
        let bucket = keys.last().unwrap().clone().meta().output_name()?;
        let count = aggs[0].clone().meta().output_name()?;
        let histogram = LazyFrame::from(ipc_scan(path)?).collect()?;
        assert_eq!(histogram.column(&bucket)?.dtype(), &DataType::UInt64);
        assert_eq!(histogram.column(&count)?.dtype(), &DataType::UInt64);
        let projected = DataFrame::new(vec![values.clone().into_column()])?
            .lazy()
            .select([expr.clone().cast(candidates.dtype().clone()).alias("value")])
            .collect()?;
        let mut expected_counts = vec![0u64; 2 * candidates.len() + 1];
        for value in projected.column("value")?.as_materialized_series().iter() {
            let less = candidates
                .iter()
                .filter(|c| c.partial_cmp(&value) == Some(std::cmp::Ordering::Less))
                .count();
            let equal = candidates.iter().filter(|c| *c == value).count();
            expected_counts[2 * less + equal] += 1;
        }
        let mut actual_counts = vec![0u64; expected_counts.len()];
        for (bucket, count) in histogram
            .column(&bucket)?
            .u64()?
            .into_no_null_iter()
            .zip(histogram.column(&count)?.u64()?.into_no_null_iter())
        {
            assert!(count > 0, "no scaffold rows");
            actual_counts[bucket as usize] = count;
        }
        assert_eq!(
            actual_counts,
            expected_counts,
            "exact typed buckets: {:?}",
            values.dtype()
        );
        if values.is_empty() {
            assert_eq!(histogram.height(), 0);
        }
        constructions += 1;
        for alpha in [(0, 1), (1, 2), (1, 1), (2, 7)] {
            for size_limit in [0u64, 1, 100] {
                let expected = input
                    .clone()
                    .select([apply_anonymous_function(
                        vec![expr.clone()],
                        DiscreteQuantileScorePlugin {
                            candidates: candidates.clone(),
                            alpha,
                            size_limit,
                        },
                    )
                    .alias("score")])
                    .collect()?;
                let score = apply_anonymous_function(
                    vec![col(bucket.clone()), col(count.clone())],
                    HistogramQuantileScorePlugin {
                        num_candidates: candidates.len(),
                        alpha,
                        size_limit,
                    },
                )
                .alias("score");
                let actual = histogram.clone().lazy().select([score]).collect()?;
                assert_eq!(
                    actual,
                    expected,
                    "dtype {:?}, alpha {alpha:?}, limit {size_limit}",
                    values.dtype()
                );
                comparisons += 1;
            }
        }
    }
    assert_eq!((constructions, comparisons), (13, 156));
    Ok(())
}

#[test]
fn literal_scorer_inputs_are_rejected() -> Fallible<()> {
    let typed = |value| {
        Expr::Literal(LiteralValue::Scalar(Scalar::new(
            DataType::Int32,
            AnyValue::Int32(value),
        )))
    };
    for input in [data()?, empty_data()?] {
        for grouped in [false, true] {
            for source in [typed(3), (typed(1) + typed(2)).cast(DataType::Int32)] {
                let scorer = apply_anonymous_function(
                    vec![source.clone()],
                    DiscreteQuantileScorePlugin {
                        candidates: Series::new("".into(), [1i32, 3, 6]),
                        alpha: (1, 2),
                        size_limit: 10,
                    },
                );
                let randomized = argmax(scorer);
                let query = aggregate(&input, [randomized], grouped);
                let dir = WorkDir::new();
                assert!(
                    format!("{:?}", split_plan(query.logical_plan, &dir.0).unwrap_err())
                        .contains("literal-only scorer")
                );
                let q = source
                    .dp()
                    .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))
                    .alias("q");
                let query = aggregate(&input, [q], grouped);
                assert!(
                    format!(
                        "{:?}",
                        split_plan(resolve(&input, query, grouped, None)?, &dir.0).unwrap_err()
                    )
                    .contains("literal-only scorer")
                );
                assert!(!dir.0.exists());
            }
        }
    }
    Ok(())
}

/// Compare reconstructed arrays rather than tied random indices on empty input.
fn compare_resolved_score_arrays(plan: DslPlan, grouped: bool) -> Fallible<()> {
    let mut expected = plan.clone();
    let exprs = match &mut expected {
        DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => expr,
        _ => panic!("expected private aggregation"),
    };
    let score = exprs
        .iter()
        .flat_map(|expr| expr.into_iter())
        .find(|expr| score_plugin(expr).unwrap().is_some())
        .unwrap()
        .clone();
    *exprs = vec![score.alias("scores")];
    let dir = WorkDir::new();
    let (heads, continuation) = split_plan(plan, &dir.0)?;
    materialize(&heads, &dir.0)?;
    let mut reconstruction = None;
    walk(&continuation, &mut |node| {
        let exprs = match node {
            DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => expr,
            _ => return,
        };
        if let Some(expr) = exprs.iter().find(|expr| {
            expr.into_iter().any(|node| {
                match_trusted_plugin::<HistogramQuantileScorePlugin>(node)
                    .unwrap()
                    .is_some()
            })
        }) {
            reconstruction = Some((node.clone(), expr.clone().meta().output_name().unwrap()));
        }
    });
    let (reconstruction, name) = reconstruction.unwrap();
    let mut outputs = vec![];
    if grouped {
        outputs.push(col("key"));
    }
    outputs.push(col(name).alias("scores"));
    let actual = LazyFrame::from(reconstruction).select(outputs);
    let expected = LazyFrame::from(expected);
    let (actual, expected) = if grouped {
        (
            actual.sort(["key"], Default::default()),
            expected.sort(["key"], Default::default()),
        )
    } else {
        (actual, expected)
    };
    let (actual, expected) = (actual.collect()?, expected.collect()?);
    assert_eq!(actual, expected);
    if !grouped {
        assert_eq!(actual.height(), 1);
        assert_eq!(
            actual
                .column("scores")?
                .array()?
                .get_as_series(0)
                .unwrap()
                .u64()?
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![0, 0, 0]
        );
    } else {
        assert_eq!(actual.height(), 0);
    }
    Ok(())
}

#[test]
fn empty_histograms_reconstruct_exact_zero_arrays() -> Fallible<()> {
    let input = empty_data()?;
    for grouped in [false, true] {
        let query = aggregate(&input, [quantile()], grouped);
        compare_resolved_score_arrays(resolve(&input, query, grouped, None)?, grouped)?;
    }
    Ok(())
}

#[test]
fn aggregation_local_removals_are_rejected() -> Fallible<()> {
    use crate::domains::OptionDomain;
    let keys = [Some(0i32), Some(0), None, None, Some(2)];
    let integers = test_domain(
        SeriesDomain::new("key", OptionDomain::new(AtomDomain::<i32>::default())),
        SeriesDomain::new("value", OptionDomain::new(AtomDomain::<i32>::default())),
        true,
    )?;
    let floats = test_domain(
        SeriesDomain::new("key", OptionDomain::new(AtomDomain::<i32>::default())),
        SeriesDomain::new("value", OptionDomain::new(AtomDomain::<f64>::default())),
        true,
    )?;
    let cases = [
        (df!("key" => keys, "value" => [Some(1i32),Some(3),Some(2),Some(4),Some(5)])?.lazy(), integers.clone(), col("value").filter(col("value").gt(lit(3)))),
        (df!("key" => keys, "value" => [Some(1i32),Some(3),Some(2),Some(4),Some(5)])?.lazy(), integers.clone(), col("value").filter(col("value").gt(lit(10)))),
        (df!("key" => keys, "value" => [Some(1i32),None,None,None,Some(5)])?.lazy(), integers.clone(), col("value").drop_nulls()),
        (df!("key" => keys, "value" => [None::<i32>;5])?.lazy(), integers.clone(), col("value").drop_nulls()),
        (df!("key" => keys, "value" => [Some(1f64),Some(f64::NAN),Some(f64::NAN),Some(f64::NAN),Some(5.)])?.lazy(), floats.clone(), col("value").drop_nans()),
        (df!("key" => keys, "value" => [Some(f64::NAN);5])?.lazy(), floats.clone(), col("value").drop_nans()),
        (df!("key" => keys, "value" => [Some(1f64),None,Some(f64::NAN),None,Some(5.)])?.lazy(), floats, col("value").drop_nulls().drop_nans()),
        (df!("key" => Vec::<Option<i32>>::new(), "value" => Vec::<Option<i32>>::new())?.lazy(), integers, col("value").filter(col("value").gt(lit(3)))),
    ];
    for (input, domain, source) in cases {
        for grouped in [false, true] {
            let q = source
                .clone()
                .dp()
                .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))
                .alias("q");
            let query = aggregate(&input, [q], grouped);
            let dir = WorkDir::new();
            let err = format!(
                "{:?}",
                split_plan(resolve_domain(&input, query, domain.clone(), None)?, &dir.0)
                    .unwrap_err()
            );
            assert!(err.contains("cardinality-changing scorer"), "{err}");
            assert!(!dir.0.exists());
        }
    }
    Ok(())
}

#[test]
fn ordinary_bucket_ids_are_exact_and_balanced() -> Fallible<()> {
    fn depth(expr: &Expr) -> usize {
        expr.clone()
            .meta()
            .pop(None)
            .unwrap()
            .iter()
            .map(depth)
            .max()
            .unwrap_or(0)
            + 1
    }
    for k in [1usize, 3, 6, 31, 127] {
        let candidates = Series::new("".into(), (0..k).map(|i| 2 * i as i64).collect::<Vec<_>>());
        let values = (-1..=(2 * k as i64 - 1)).collect::<Vec<_>>();
        let bucket = quantile_bucket(col("value"), &candidates, 0, k)?;
        assert!(
            depth(&bucket) <= 2 * (k + 1).ilog2() as usize + 5,
            "k={k}, depth={}",
            depth(&bucket)
        );
        let result = df!("value" => values)?
            .lazy()
            .select([bucket.alias("bucket")])
            .collect()?;
        assert_eq!(
            result
                .column("bucket")?
                .u64()?
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            (0..=(2 * k) as u64).collect::<Vec<_>>()
        );
    }
    Ok(())
}

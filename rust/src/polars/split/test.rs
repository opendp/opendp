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

/// Every ordinary head and IPC scan can round-trip before its file exists.
#[test]
fn ordered_heads_roundtrip_before_materialization() -> Fallible<()> {
    let dir = WorkDir::new();
    let plan = data()?.select([quantile(), sum()]);
    let input = data()?;
    let (heads, continuation) = split(resolve(&input, plan, false, None)?, &dir.0)?;
    let mut scans = vec![];
    walk(&continuation, &mut |node| {
        if let DslPlan::Scan {
            sources: ScanSources::Paths(paths),
            ..
        } = node
        {
            scans.extend(paths.iter().cloned());
        }
    });
    let expected = (0..heads.len())
        .map(|i| PlPath::Local(dir.0.join(format!("{i}.arrow")).into()))
        .collect::<Vec<_>>();
    // Continuation traversal order need not match head emission order.
    assert_eq!(scans.len(), expected.len());
    assert!(expected.iter().all(|path| scans.contains(path)));
    let mut restored = vec![];
    for (i, head) in heads.iter().enumerate() {
        let mut bytes = vec![];
        head.serialize_versioned(&mut bytes, Default::default())?;
        restored.push(DslPlan::deserialize_versioned(bytes.as_slice())?);
        bytes.clear();
        ipc_scan(&dir.0.join(format!("{i}.arrow")))?
            .serialize_versioned(&mut bytes, Default::default())?;
        let _ = DslPlan::deserialize_versioned(bytes.as_slice())?;
    }
    assert!(!dir.0.exists());
    materialize(&restored, &dir.0)?;
    assert_eq!(collect_sorted(continuation, None)?.shape(), (1, 2));
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
        | DslPlan::IR { dsl: input, .. } => walk(input, f),
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

fn plugins<P: crate::polars::OpenDPPlugin>(plan: &DslPlan) -> Vec<P> {
    let mut parameters = vec![];
    walk(plan, &mut |node| {
        parameters.extend(
            expressions(node)
                .iter()
                .flat_map(|expr| expr.into_iter())
                .filter_map(|expr| {
                    match_trusted_plugin::<P>(expr)
                        .unwrap()
                        .map(|(_, args)| args)
                }),
        );
    });
    parameters
}

fn noise_parameters(plan: &DslPlan) -> Vec<NoisePlugin> {
    plugins::<NoisePlugin>(plan)
}

/// Tests execute ordinary heads themselves, using the same positional IPC contract
/// as the trusted executor. No native sink plan or Python runtime is involved.
fn split(plan: DslPlan, dir: &Path) -> Fallible<(Vec<DslPlan>, DslPlan)> {
    let mut plans = split_plan(plan, dir)?;
    let continuation = plans.pop().expect("final continuation");
    assert!(!plans.is_empty());
    Ok((plans, continuation))
}

fn base(heads: &[DslPlan]) -> &DslPlan {
    match &heads[0] {
        DslPlan::Select { input, .. } | DslPlan::GroupBy { input, .. } => input,
        _ => panic!("expected private aggregate head"),
    }
}

fn materialize(heads: &[DslPlan], dir: &Path) -> Fallible<()> {
    fs::create_dir_all(dir).unwrap();
    for (i, head) in heads.iter().enumerate() {
        let mut result = LazyFrame::from(head.clone()).collect()?;
        IpcWriter::new(fs::File::create(dir.join(format!("{i}.arrow"))).unwrap())
            .finish(&mut result)?;
    }
    Ok(())
}

fn collect_sorted(plan: DslPlan, key: Option<&str>) -> Fallible<DataFrame> {
    let lf = LazyFrame::from(plan);
    Ok(match key {
        Some(key) => lf.sort([key], Default::default()),
        None => lf,
    }
    .collect()?)
}

fn compare(
    plan: DslPlan,
    expected_heads: usize,
    grouped: bool,
) -> Fallible<(Vec<DslPlan>, DslPlan)> {
    compare_key(plan, expected_heads, grouped.then_some("key"))
}

fn compare_key(
    plan: DslPlan,
    expected_heads: usize,
    key: Option<&str>,
) -> Fallible<(Vec<DslPlan>, DslPlan)> {
    let dir = WorkDir::new();
    let (deterministic, continuation) = split(plan.clone(), &dir.0)?;
    assert!(
        !dir.0.exists(),
        "splitter must not create the working directory"
    );
    assert_eq!(deterministic.len(), expected_heads);
    for head in &deterministic {
        assert_eq!(noise_count(head), 0);
        assert!(noisy_max_parameters(head).is_empty());
        walk(head, &mut |node| {
            assert!(!matches!(
                node,
                DslPlan::Sink { .. } | DslPlan::SinkMultiple { .. }
            ))
        });
    }
    assert_eq!(noise_parameters(&continuation), noise_parameters(&plan));
    assert_eq!(
        noisy_max_parameters(&continuation),
        noisy_max_parameters(&plan)
    );
    walk(&continuation, &mut |node| {
        assert!(!matches!(node, DslPlan::DataFrameScan { .. }))
    });
    materialize(&deterministic, &dir.0)?;
    assert_eq!(
        collect_sorted(continuation.clone(), key)?,
        collect_sorted(plan, key)?
    );
    Ok((deterministic, continuation))
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

fn assert_collision_safe(
    plan: DslPlan,
    expected_heads: usize,
    key: Option<&str>,
    reserved: &[&str],
) -> Fallible<Vec<DslPlan>> {
    let (heads, _) = compare_key(plan, expected_heads, key)?;
    let mut schemas = vec![];
    let mut generated = HashSet::new();
    for head in &heads {
        // Private aliases and histogram intermediates must be globally unique
        // and disjoint from key, release and downstream reserved names.
        let exprs = match head {
            DslPlan::Select { expr, .. } => expr.clone(),
            DslPlan::GroupBy { keys, aggs, .. } => keys
                .iter()
                .skip(usize::from(key.is_some()))
                .chain(aggs)
                .cloned()
                .collect(),
            _ => panic!("expected private aggregate head"),
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
    Ok(heads)
}

#[test]
fn names_are_reserved_across_preprocessing_heads_and_postprocessing() -> Fallible<()> {
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
    let query = aggregate(
        &input,
        [
            quantile().alias("__opendp_split_1"),
            sum().alias("__opendp_split_2"),
            quantile().alias("__opendp_split_3"),
        ],
        true,
    );
    let mut plan = resolve(&input, query, true, None)?;
    let DslPlan::GroupBy { keys, .. } = &mut plan else {
        panic!()
    };
    *keys = vec![col("key").alias(key)];
    let plan = LazyFrame::from(plan)
        .with_column(col("__opendp_split_1").alias("__opendp_split_4"))
        .with_column(col("__opendp_split_2").alias("__opendp_split_5"));
    assert_collision_safe(
        plan.logical_plan,
        3,
        Some(key),
        &[
            key,
            "__opendp_split_1",
            "__opendp_split_2",
            "__opendp_split_3",
            "__opendp_split_4",
            "__opendp_split_5",
        ],
    )?;
    Ok(())
}

#[test]
fn additional_sources_outside_the_private_boundary_are_rejected() -> Fallible<()> {
    let private = data()?.select([noisy(col("value").sum(), 0.)]);
    let dir = WorkDir::new();
    for source in [
        data()?.logical_plan,
        ipc_scan(&dir.0.join("missing.arrow"))?,
    ] {
        for source_on_left in [false, true] {
            let source = LazyFrame::from(source.clone());
            let (left, right) = if source_on_left {
                (source, private.clone())
            } else {
                (private.clone(), source)
            };
            let plan = left.join(right, [col("value")], [col("value")], JoinType::Left.into());
            let err = split_plan(plan.logical_plan, &dir.0).unwrap_err();
            assert!(format!("{err:?}").contains("unsupported additional source"));
            assert!(!dir.0.exists());
        }
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
    bases.push((
        "join",
        input
            .clone()
            .join(
                input.clone(),
                [col("key")],
                [col("key")],
                JoinType::Inner.into(),
            )
            .logical_plan,
    ));
    bases.push((
        "unresolved scan",
        ipc_scan(Path::new("missing-input.arrow"))?,
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
            let (heads, _) = split(plan, &dir.0)?;
            let mut actual = self::base(&heads);
            if histogram {
                let DslPlan::GroupBy { keys, .. } = &heads[0] else {
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
    let (deterministic, continuation) = split(plan, &dir.0)?;
    assert_eq!(noise_count(&continuation), 2);
    let DslPlan::Select { expr, .. } = &deterministic[0] else {
        panic!()
    };
    assert_eq!(expr.len(), 2, "each occurrence gets its own input slot");
    assert_ne!(
        expr[0].clone().meta().output_name()?,
        expr[1].clone().meta().output_name()?
    );
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
    let (deterministic, continuation) = split(plan, &dir.0)?;
    let DslPlan::GroupBy { aggs, .. } = &deterministic[0] else {
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
            let exprs = match &deterministic[0] {
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
    let mut parameters = plugins::<NoisyMaxPlugin>(plan)
        .into_iter()
        .map(|p| (p.replacement, p.scale.to_bits(), p.negate))
        .collect::<Vec<_>>();
    parameters.sort();
    parameters
}

fn quantile() -> Expr {
    col("value")
        .dp()
        .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))
        .alias("quantile")
}

#[test]
fn compatible_aggregates_and_independent_histograms() -> Fallible<()> {
    let input = data()?;
    for grouped in [false, true] {
        for (exprs, expected) in [
            (
                vec![sum(), count(), mean(), (sum() + lit(1)).alias("offset")],
                1,
            ),
            (vec![quantile()], 1),
            (vec![quantile().alias("a"), quantile().alias("b")], 2),
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
            for head in &deterministic {
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
    let mut divides = 0;
    let mut lookups = 0;
    walk(&input, &mut |local| {
        for node in expressions(local).iter().flat_map(|e| e.into_iter()) {
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
                >(node)
                .unwrap()
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
    });
    assert_eq!((divides, lookups), (1, 1));
    Ok(())
}

#[test]
fn empty_quantile_inputs_preserve_output_cardinality() -> Fallible<()> {
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
    }
    Ok(())
}

#[test]
fn missing_later_group_is_imputed_before_randomization() -> Fallible<()> {
    let input = data()?;
    let quantile = col("value")
        .dp()
        .quantile(0.5, Series::new("".into(), [3i32]), Some(0.))
        .alias("quantile");
    let query = input
        .clone()
        .group_by([col("key")])
        .agg([count(), quantile]);
    let dir = WorkDir::new();
    let (heads, continuation) = split(resolve(&input, query, false, Some(1))?, &dir.0)?;
    assert_eq!(heads.len(), 2, "threshold-bearing length must be in the first head");
    let DslPlan::GroupBy { aggs, .. } = &heads[0] else {
        panic!("the first head must hold the scalar aggregates");
    };
    assert!(aggs.iter().any(|expr| expr.into_iter().any(|node| matches!(node, Expr::Len))));
    fs::create_dir_all(&dir.0).unwrap();
    for (i, head) in heads.into_iter().enumerate() {
        let lf = LazyFrame::from(head);
        // Simulate a later snapshot in which group 0 has disappeared.
        let mut frame = if i == 0 {
            lf.collect()?
        } else {
            lf.filter(col("key").eq(lit(1))).collect()?
        };
        IpcWriter::new(fs::File::create(dir.0.join(format!("{i}.arrow"))).unwrap())
            .finish(&mut frame)?;
    }
    let result = collect_sorted(continuation, Some("key"))?;
    assert_eq!(result.height(), 1);
    assert_eq!(result.column("key")?.i32()?.get(0), Some(0));
    assert_eq!(result.column("quantile")?.i32()?.get(0), Some(3));
    Ok(())
}

#[test]
fn one_threshold_follows_combination() -> Fallible<()> {
    let input = data()?;
    for exprs in [vec![count()], vec![quantile(), sum(), count(), mean()]] {
        for threshold in [None, Some(1)] {
            let query = aggregate(&input, exprs.clone(), true);
            let expected_heads = 1 + usize::from(exprs.len() > 1);
            let (heads, continuation) = compare(
                resolve(&input, query, threshold.is_none(), threshold)?,
                expected_heads,
                true,
            )?;
            let mut filters = 0;
            walk(&continuation, &mut |node| {
                filters += usize::from(matches!(node, DslPlan::Filter { .. }))
            });
            assert_eq!(filters, usize::from(threshold.is_some()));
            if threshold.is_some() {
                assert!(matches!(continuation, DslPlan::Filter { .. }));
            }
            for head in heads {
                let DslPlan::GroupBy { keys, .. } = head else {
                    panic!()
                };
                assert_eq!(keys[0], col("key"));
            }
        }
    }
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
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["key"]).with_max_length(7).with_invariant_keys())?;
    let plan = resolve_domain(
        &input,
        input
            .clone()
            .group_by([col("key")])
            .agg([col("value").dp().quantile(
                0.25,
                Series::new("".into(), [1i32, 3, 6]),
                Some(0.),
            )]),
        domain,
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
    let (heads, continuation) = split(plan.clone(), &dir.0)?;
    let DslPlan::GroupBy {
        input: buckets,
        keys,
        aggs,
        ..
    } = &heads[0]
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
fn literal_scorer_inputs_are_rejected() -> Fallible<()> {
    let input = data()?;
    let typed = |value| {
        Expr::Literal(LiteralValue::Scalar(Scalar::new(
            DataType::Int32,
            AnyValue::Int32(value),
        )))
    };
    for grouped in [false, true] {
        for source in [typed(3), (typed(1) + typed(2)).cast(DataType::Int32)] {
            let query = aggregate(
                &input,
                [source
                    .dp()
                    .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))],
                grouped,
            );
            let dir = WorkDir::new();
            let err = split_plan(resolve(&input, query, grouped, None)?, &dir.0).unwrap_err();
            assert!(format!("{err:?}").contains("literal-only scorer"));
            assert!(!dir.0.exists());
        }
    }
    Ok(())
}

#[test]
fn aggregation_local_removals_are_preserved_in_histogram_heads() -> Fallible<()> {
    use crate::domains::OptionDomain;
    let input = df!("key" => [0i32; 3], "value" => [Some(1f64), None, Some(f64::NAN)])?.lazy();
    let domain = test_domain(
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", OptionDomain::new(AtomDomain::<f64>::default())),
        true,
    )?;
    for source in [
        col("value").filter(col("value").gt(lit(3))),
        col("value").drop_nulls(),
        col("value").drop_nans(),
        col("value").drop_nulls().drop_nans(),
    ] {
        for grouped in [false, true] {
            let q =
                source
                    .clone()
                    .dp()
                    .quantile(0.5, Series::new("".into(), [1f64, 3., 6.]), Some(0.));
            let query = aggregate(&input, [q], grouped);
            let dir = WorkDir::new();
            let plans = split_plan(resolve_domain(&input, query, domain.clone(), None)?, &dir.0)?;
            assert_eq!(plans.len(), 2);
            let DslPlan::GroupBy { input: buckets, .. } = &plans[0] else {
                panic!("expected histogram head");
            };
            let DslPlan::HStack { input: values, .. } = buckets.as_ref() else {
                panic!("expected bucket computation");
            };
            let DslPlan::HStack { exprs, .. } = values.as_ref() else {
                panic!("expected scorer input computation");
            };
            assert!(exprs[0].into_iter().any(|node| node == &source));
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

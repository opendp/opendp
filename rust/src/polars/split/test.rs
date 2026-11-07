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
    process::Command,
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

fn python(script: &str, args: &[&Path]) {
    let output = Command::new(std::env::var("PYTHON").unwrap_or("python".into()))
        .arg("-c")
        .arg(script)
        .args(args)
        .output()
        .expect("Python with Polars 1.36.1 is required");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Both native stages serialize before IPC exists; deserialized sinks write the
/// files, and Python then executes the file-backed scan without noise.
#[test]
fn ipc_before_materialization() -> Fallible<()> {
    let dir = WorkDir::new();
    let (deterministic, _) = split_plan(
        data()?.select([noisy(col("value").sum(), 0.)]).logical_plan,
        &dir.0,
    )?;
    assert!(!dir.0.exists());
    fs::create_dir_all(&dir.0).unwrap();
    let ipc = sinks(&deterministic)[0].1.to_path_buf();
    let mut bytes = vec![];
    deterministic.serialize_versioned(&mut bytes, Default::default())?;
    let restored = DslPlan::deserialize_versioned(bytes.as_slice())?;
    assert_eq!(
        sinks(&restored)
            .iter()
            .map(|(_, path)| *path)
            .collect::<Vec<_>>(),
        sinks(&deterministic)
            .iter()
            .map(|(_, path)| *path)
            .collect::<Vec<_>>()
    );
    let serialized_sinks = dir.0.join("deterministic.bin");
    fs::write(&serialized_sinks, bytes).unwrap();
    let serialized_scan = dir.0.join("continuation.bin");
    ipc_scan(&ipc)?.serialize_versioned(
        fs::File::create(&serialized_scan).unwrap(),
        Default::default(),
    )?;
    assert!(!ipc.exists());
    python(
        "import polars as pl, sys\npl.LazyFrame.deserialize(sys.argv[1])\npl.LazyFrame.deserialize(sys.argv[2])",
        &[&serialized_sinks, &serialized_scan],
    );
    assert!(!ipc.exists());
    materialize(&restored, &dir.0)?;
    python(
        "import polars as pl, sys\nassert pl.LazyFrame.deserialize(sys.argv[1]).collect().to_dict(as_series=False) == {'__opendp_split_0': [10]}",
        &[&serialized_scan],
    );
    Ok(())
}

fn data() -> Fallible<LazyFrame> {
    Ok(df!("key" => [0i32, 0, 1], "value" => [1i32, 6, 3])?.lazy())
}

fn resolve(
    input: &LazyFrame,
    query: LazyFrame,
    public_keys: bool,
    threshold: Option<u32>,
) -> Fallible<DslPlan> {
    let by = Margin::by(["key"]).with_max_length(10);
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::select().with_max_length(10))?
    .with_margin(if public_keys {
        by.with_invariant_keys()
    } else {
        by
    })?;
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

fn noise_count(plan: &DslPlan) -> usize {
    noise_parameters(plan).len()
}

fn noise_parameters(plan: &DslPlan) -> Vec<NoisePlugin> {
    let mut parameters = vec![];
    walk(plan, &mut |node| {
        let exprs = match node {
            DslPlan::Select { expr, .. }
            | DslPlan::GroupBy { aggs: expr, .. }
            | DslPlan::HStack { exprs: expr, .. } => expr.as_slice(),
            DslPlan::Filter { predicate, .. } => std::slice::from_ref(predicate),
            _ => &[],
        };
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
    let collect = |plan| -> Fallible<DataFrame> {
        let lf = LazyFrame::from(plan);
        Ok(if grouped {
            lf.sort(["key"], Default::default())
        } else {
            lf
        }
        .collect()?)
    };
    assert_eq!(collect(continuation.clone())?, collect(plan)?);
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
        let query = if grouped {
            input.clone().group_by([col("key")]).agg(exprs)
        } else {
            input.clone().select(exprs)
        };
        let plan = make_private_lazyframe(
            domain.clone(),
            SymmetricDistance,
            Approximate(MaxDivergence),
            query,
            Some(1.),
            None,
        )?
        .invoke(&input)?
        .lazyframe()
        .logical_plan;
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

#[test]
fn public_key_join_multiple_fragments() -> Fallible<()> {
    let input = data()?;
    for left in [true, false] {
        let private = input
            .clone()
            .group_by([col("key")])
            .agg([sum(), count(), mean()]);
        // Deliberately different ordering and duplicate keys. Association must
        // be by explicit keys, never by fragment row positions.
        let keys = df!("key" => [2i32, 0, 2, 1])?.lazy();
        let query = if left {
            keys.join(private, [col("key")], [col("key")], JoinType::Left.into())
        } else {
            private.join(keys, [col("key")], [col("key")], JoinType::Right.into())
        };
        let (_, continuation) = compare(resolve(&input, query, false, None)?, 2, true)?;
        assert!(matches!(&continuation, DslPlan::HStack { .. }));
        let mut distinct = 0;
        walk(&continuation, &mut |p| {
            if matches!(p, DslPlan::Distinct { .. }) {
                distinct += 1;
            }
        });
        assert_eq!(distinct, 1);
    }
    Ok(())
}

#[test]
fn preprocessing_and_name_collisions() -> Fallible<()> {
    let input = data()?;
    let query = input
        .clone()
        .filter(col("value").gt(lit(1)))
        .with_column(col("value").cast(DataType::Int64))
        .select([col("value")
            .dp()
            .sum((lit(0i64), lit(5i64)), Some(0.))
            .alias("__opendp_split_0")]);
    let plan = resolve(&input, query, false, None)?;
    let (deterministic, _) = compare(plan, 1, false)?;
    assert!(matches!(base(&deterministic), DslPlan::HStack { .. }));
    let DslPlan::Select { expr, .. } = sinks(&deterministic)[0].0 else {
        panic!()
    };
    assert_ne!(
        expr[0].clone().meta().output_name()?.as_str(),
        "__opendp_split_0"
    );
    Ok(())
}

#[test]
fn names_are_reserved_across_heads_and_join_branches() -> Fallible<()> {
    let input = df!("__opendp_split_0" => [0i32, 1], "value" => [2i32, 3])?.lazy();
    let private = input.group_by([col("__opendp_split_0")]).agg([
        noisy(col("value").sum(), 0.).alias("__opendp_split_1"),
        noisy(len().cast(DataType::Int64), 0.).alias("count"),
    ]);
    let public = df!("__opendp_split_0" => [1i32, 0], "__opendp_split_2" => [7i32, 8])?.lazy();
    for left in [true, false] {
        let joined = if left {
            public.clone().join(
                private.clone(),
                [col("__opendp_split_0")],
                [col("__opendp_split_0")],
                JoinType::Left.into(),
            )
        } else {
            private.clone().join(
                public.clone(),
                [col("__opendp_split_0")],
                [col("__opendp_split_0")],
                JoinType::Right.into(),
            )
        }
        .select([
            col("__opendp_split_0"),
            col("__opendp_split_1").alias("__opendp_split_3"),
            col("__opendp_split_2"),
            col("count"),
        ]);
        let dir = WorkDir::new();
        let (deterministic, continuation) = split_plan(joined.logical_plan.clone(), &dir.0)?;
        let heads = sinks(&deterministic);
        assert_eq!(heads.len(), 2);
        assert!(
            heads
                .iter()
                .all(|(_, path)| path.parent() == Some(dir.0.as_path()))
        );
        let aggs = heads
            .iter()
            .find_map(|(head, _)| match head {
                DslPlan::GroupBy { aggs, .. } => Some(aggs),
                _ => None,
            })
            .expect("aggregate head");
        let names = aggs
            .iter()
            .map(|agg| agg.clone().meta().output_name())
            .collect::<PolarsResult<Vec<_>>>()?;
        assert_eq!(
            names.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
            ["__opendp_split_4", "__opendp_split_5"]
        );
        let schema = |head: &DslPlan| -> Fallible<HashSet<PlSmallStr>> {
            Ok(LazyFrame::from(head.clone())
                .collect_schema()?
                .iter_names()
                .cloned()
                .collect())
        };
        let first = schema(heads[0].0)?;
        let second = schema(heads[1].0)?;
        assert_eq!(
            first
                .intersection(&second)
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            ["__opendp_split_0"]
        );
        materialize(&deterministic, &dir.0)?;
        let collect = |plan| {
            LazyFrame::from(plan)
                .sort(["__opendp_split_0"], Default::default())
                .collect()
        };
        assert_eq!(collect(continuation)?, collect(joined.logical_plan)?);
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
fn base_is_opaque_and_preserved_verbatim() -> Fallible<()> {
    let source = data()?;
    for base in [
        source
            .clone()
            .with_column(callback_expr().alias("value"))
            .logical_plan,
        DslPlan::Union {
            inputs: vec![source.logical_plan.clone(), source.logical_plan],
            args: Default::default(),
        },
    ] {
        let base = Arc::new(base);
        let plan = DslPlan::Select {
            input: base.clone(),
            expr: vec![noisy(col("value").sum(), 0.)],
            options: Default::default(),
        };
        let dir = WorkDir::new();
        let (deterministic, _) = split_plan(plan, &dir.0)?;
        assert!(!dir.0.exists());
        let DslPlan::Select { input, .. } = sinks(&deterministic)[0].0 else {
            panic!()
        };
        assert!(
            Arc::ptr_eq(input, &base),
            "base input must pass through unchanged"
        );
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
    let input = df!("key" => Vec::<i32>::new(), "value" => Vec::<i32>::new())?.lazy();
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
            let query = if grouped {
                input.clone().group_by([col("key")]).agg([expression])
            } else {
                input.clone().select([expression])
            };
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
    ] {
        for grouped in [false, true] {
            let input = data()?;
            let release = noisy(expression.clone(), 0.).alias("release");
            let query = if grouped {
                input.group_by([col("key")]).agg([release])
            } else {
                input.select([release])
            };
            let (deterministic, _) = compare(query.logical_plan, 1, grouped)?;
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
        let query = if grouped {
            input.group_by([col("key")]).agg([release])
        } else {
            input.select([release])
        };
        compare(query.logical_plan, 1, grouped)?;
    }
    Ok(())
}

#[test]
fn deterministic_operations_are_left_to_the_translator() -> Fallible<()> {
    // Opaque callbacks are not SQL compatibility errors for the splitter. Their
    // deterministic expression subtree must remain intact below the cut.
    let input = data()?;
    let expression = callback_expr().sum();
    let (deterministic, continuation) = compare(
        input.select([noisy(expression.clone(), 0.)]).logical_plan,
        1,
        false,
    )?;
    let DslPlan::Select { expr, .. } = sinks(&deterministic)[0].0 else {
        panic!()
    };
    let Expr::Alias(actual, _) = &expr[0] else {
        panic!()
    };
    assert_eq!(actual.as_ref(), &expression);
    assert_eq!(noise_count(&continuation), 1);
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

/// Needs a matching plugin binary, e.g. OPENDP_POLARS_LIB_PATH=/.../libopendp.dylib.
/// Run with --features polars-ffi,untrusted -- --ignored.
#[cfg(feature = "polars-ffi")]
#[test]
#[ignore = "requires OPENDP_POLARS_LIB_PATH pointing to a matching OpenDP plugin binary"]
fn python_noise_ffi_round_trip() -> Fallible<()> {
    use crate::{
        measurements::{expr_dp_mean::DPMeanShim, expr_dp_sum::DPSumShim},
        polars::{OpenDPPlugin, apply_plugin},
    };
    let lib = std::env::var("OPENDP_POLARS_LIB_PATH").expect("set OPENDP_POLARS_LIB_PATH");
    let plugin = |input| Expr::Function {
        input: vec![input],
        function: FunctionExpr::FfiPlugin {
            lib: lib.clone().into(),
            symbol: NoisePlugin::NAME.into(),
            kwargs: serde_pickle::to_vec(
                &NoisePlugin {
                    distribution: NoiseDistribution::Laplace,
                    scale: 0.,
                    support: Support::Integer,
                },
                Default::default(),
            )
            .unwrap()
            .as_slice()
            .into(),
            flags: NoisePlugin::function_options(),
        },
    };
    let input = data()?;
    let threshold = input
        .clone()
        .group_by([col("key")])
        .agg([
            plugin(col("value").sum()).alias("sum"),
            Expr::BinaryExpr {
                left: Arc::new(plugin(col("value").sum())),
                op: Operator::TrueDivide,
                right: Arc::new(plugin(len().cast(DataType::Int64))),
            }
            .alias("mean"),
        ])
        .filter(col("sum").gt(lit(4)));
    // Resolve actual FFI shims through OpenDP, including public-key imputation.
    let sum = apply_plugin(
        vec![col("value"), lit(0), lit(5), lit(0.)],
        plugin(col("value")),
        DPSumShim,
    )
    .alias("sum");
    let mean = apply_plugin(
        vec![col("value"), lit(0), lit(5), lit(0.)],
        plugin(col("value")),
        DPMeanShim,
    )
    .alias("mean");
    let public = df!("key" => [2i32, 0, 2, 1])?.lazy();
    let joined = input
        .clone()
        .group_by([col("key")])
        .agg([sum.clone(), mean])
        .join(public, [col("key")], [col("key")], JoinType::Right.into());
    let plans = [
        (
            threshold.logical_plan,
            "assert result == {'key': [0], 'sum': [7], 'mean': [3.5]}, result",
        ),
        (
            resolve(&input, input.clone().select([sum]), false, None)?,
            "assert result == {'sum': [9]}, result",
        ),
        (
            resolve(&input, joined, false, None)?,
            "assert result['key'] == [0, 1, 2], result\nassert result['sum'] == [6, 3, 0], result\nassert result['mean'][:2] == [3.0, 3.0] and math.isnan(result['mean'][2]), result",
        ),
    ];
    for (plan, expected) in plans {
        let dir = WorkDir::new();
        let (deterministic, continuation) = split_plan(plan, &dir.0)?;
        fs::create_dir_all(&dir.0).unwrap();
        let serialized = dir.0.join("continuation.bin");
        continuation
            .serialize_versioned(fs::File::create(&serialized).unwrap(), Default::default())?;
        let mut bytes = vec![];
        deterministic.serialize_versioned(&mut bytes, Default::default())?;
        let deterministic = DslPlan::deserialize_versioned(bytes.as_slice())?;
        assert!(sinks(&deterministic).iter().all(|(_, p)| !p.exists()));
        materialize(&deterministic, &dir.0)?;
        python(
            &format!(
                "import polars as pl, sys, math\nresult = pl.LazyFrame.deserialize(sys.argv[1]).collect()\nif 'key' in result.columns: result = result.sort('key')\nresult = result.to_dict(as_series=False)\n{expected}"
            ),
            &[&serialized],
        );
    }
    Ok(())
}

fn noisy_max_parameters(plan: &DslPlan) -> Vec<(bool, u64, bool)> {
    let mut out = vec![];
    walk(plan, &mut |node| {
        let exprs = match node {
            DslPlan::Select { expr, .. }
            | DslPlan::GroupBy { aggs: expr, .. }
            | DslPlan::HStack { exprs: expr, .. } => expr.as_slice(),
            _ => &[],
        };
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
        for exprs in [
            vec![quantile()],
            vec![
                col("value")
                    .dp()
                    .median(Series::new("".into(), [1i32, 3, 6]), Some(0.))
                    .alias("median"),
            ],
            vec![quantile(), sum(), count(), mean()],
            vec![
                quantile(),
                col("value")
                    .dp()
                    .quantile(0.5, Series::new("".into(), [0i32, 1, 3, 6, 7]), Some(0.))
                    .alias("other"),
            ],
            vec![
                (quantile() + lit(2)).alias("offset"),
                (quantile() - mean()).alias("difference"),
            ],
        ] {
            let expected = if exprs.len() == 1 {
                1
            } else if exprs.len() == 4 {
                2
            } else if exprs[0].clone().meta().output_name()? == "offset" {
                3
            } else {
                2
            };
            let query = if grouped {
                input.clone().group_by([col("key")]).agg(exprs)
            } else {
                input.clone().select(exprs)
            };
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
    for input in [
        data()?,
        df!("key" => Vec::<i32>::new(), "value" => Vec::<i32>::new())?.lazy(),
    ] {
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
        for private_left in [true, false] {
            let private = input
                .clone()
                .group_by([col("key")])
                .agg([q(), sum(), count(), mean()]);
            let public = df!("key" => [2i32, 0, 2, 1])?.lazy();
            let query = if private_left {
                private.join(public, [col("key")], [col("key")], JoinType::Right.into())
            } else {
                public.join(private, [col("key")], [col("key")], JoinType::Left.into())
            };
            let (_, continuation) = compare(resolve(&input, query, false, None)?, 3, true)?;
            let mut dedup = 0;
            let mut public_joins = 0;
            walk(&continuation, &mut |node| {
                dedup += usize::from(matches!(node, DslPlan::Distinct { .. }));
                if let DslPlan::Join { options, .. } = node {
                    public_joins +=
                        usize::from(matches!(options.args.how, JoinType::Left | JoinType::Right));
                }
            });
            assert_eq!((dedup, public_joins), (1, 1));
            assert!(matches!(continuation, DslPlan::HStack { .. }));
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
    let plan = make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        query,
        Some(1.),
        None,
    )?
    .invoke(&input)?
    .lazyframe()
    .logical_plan;
    compare(plan, 2, true)?;
    let input =
        df!("key" => [0i32; 5], "value" => [Some(1f64), None, Some(f64::NAN), Some(3.), Some(6.)])?
            .lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", OptionDomain::new(AtomDomain::<f64>::default())),
    ])?
    .with_margin(Margin::select().with_max_length(10))?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_invariant_keys(),
    )?;
    for grouped in [false, true] {
        let q = col("value")
            .dp()
            .quantile(0.5, Series::new("".into(), [1f64, 3., 6.]), Some(0.))
            .alias("q");
        let query = if grouped {
            input.clone().group_by([col("key")]).agg([q])
        } else {
            input.clone().select([q])
        };
        let plan = make_private_lazyframe(
            domain.clone(),
            SymmetricDistance,
            Approximate(MaxDivergence),
            query,
            Some(1.),
            None,
        )?
        .invoke(&input)?
        .lazyframe()
        .logical_plan;
        compare(plan, 1, grouped)?;
    }
    Ok(())
}

#[test]
fn histogram_names_do_not_collide() -> Fallible<()> {
    let input =
        df!("key" => [0i32, 0, 1], "value" => [1i32, 6, 3], "__opendp_split_0" => [1,2,3])?.lazy();
    let query = input.clone().group_by([col("key")]).agg([
        quantile().alias("__opendp_split_1"),
        sum().alias("__opendp_split_2"),
        quantile().alias("__opendp_split_3"),
    ]);
    let public = df!("key" => [0i32, 1], "__opendp_split_4" => [4,5])?.lazy();
    let query = query.join(public, [col("key")], [col("key")], JoinType::Right.into());
    compare(resolve(&input, query, false, None)?, 4, true)?;
    Ok(())
}

#[test]
fn histogram_heads_preserve_the_resolved_scorer_contract() -> Fallible<()> {
    let input = data()?;
    let plan = resolve(
        &input,
        input.clone().group_by([col("key")]).agg([col("value")
            .filter(col("value").gt(lit(1)))
            .dp()
            .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))
            .alias("q")]),
        true,
        None,
    )?;
    let mut original = None;
    walk(&plan, &mut |node| {
        if let DslPlan::GroupBy { aggs, .. } = node {
            for expr in aggs.iter().flat_map(|e| e.into_iter()) {
                if let Some((inputs, scorer)) =
                    match_trusted_plugin::<DiscreteQuantileScorePlugin>(expr).unwrap()
                {
                    original = Some((inputs.clone(), scorer));
                }
            }
        }
    });
    let original = original.unwrap();
    let dir = WorkDir::new();
    let (heads, _) = split_plan(plan, &dir.0)?;
    let mut found = 0;
    walk(sinks(&heads)[0].0, &mut |node| {
        if let DslPlan::GroupBy { keys, aggs, .. } = node {
            assert_eq!(keys, &vec![col("key")]);
            for expr in aggs.iter().flat_map(|e| e.into_iter()) {
                if let Some((inputs, source)) =
                    match_trusted_plugin::<DiscreteQuantileHistogramPlugin>(expr).unwrap()
                {
                    assert_eq!(inputs, &original.0);
                    assert!(source.scorer.candidates.equals(&original.1.candidates));
                    assert_eq!(source.scorer.alpha, original.1.alpha);
                    assert_eq!(source.scorer.size_limit, original.1.size_limit);
                    assert_ne!(source.bucket_name, source.count_name);
                    found += 1;
                }
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
    for (values, candidates) in cases {
        for alpha in [(0, 1), (1, 2), (1, 1), (2, 7)] {
            for size_limit in [0u64, 1, 100] {
                let input = DataFrame::new(vec![values.clone().into_column()])?.lazy();
                let mut expr = col("value").fill_null(lit(0));
                if values.dtype().is_float() {
                    expr = expr.fill_nan(lit(0));
                }
                let scorer = DiscreteQuantileScorePlugin {
                    candidates: candidates.clone(),
                    alpha,
                    size_limit,
                };
                let scores = apply_anonymous_function(vec![expr], scorer);
                let expected = input
                    .clone()
                    .select([scores.clone().alias("score")])
                    .collect()?;
                let query = input.select([apply_anonymous_function(
                    vec![scores],
                    NoisyMaxPlugin {
                        replacement: false,
                        scale: 0.,
                        negate: true,
                    },
                )]);
                let dir = WorkDir::new();
                let (heads, _) = split_plan(query.logical_plan, &dir.0)?;
                assert_eq!(sinks(&heads).len(), 1);
                materialize(&heads, &dir.0)?;
                let (head, path) = sinks(&heads)[0];
                let mut args = None;
                walk(head, &mut |node| {
                    let exprs = match node {
                        DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => {
                            expr.as_slice()
                        }
                        _ => &[],
                    };
                    for e in exprs.iter().flat_map(|e| e.into_iter()) {
                        if let Some((_, kw)) =
                            match_trusted_plugin::<DiscreteQuantileHistogramPlugin>(e).unwrap()
                        {
                            args = Some(kw);
                        }
                    }
                });
                let args = args.unwrap();
                let bucket = args.bucket_name;
                let count = args.count_name;
                let score = apply_anonymous_function(
                    vec![col(bucket), col(count)],
                    HistogramQuantileScorePlugin {
                        num_candidates: candidates.len(),
                        alpha,
                        size_limit,
                    },
                )
                .alias("score");
                let actual = LazyFrame::from(ipc_scan(path)?).select([score]).collect()?;
                assert_eq!(
                    actual,
                    expected,
                    "dtype {:?}, alpha {alpha:?}, limit {size_limit}",
                    values.dtype()
                );
            }
        }
    }
    Ok(())
}

#[cfg(feature = "polars-ffi")]
#[test]
#[ignore = "requires rebuilt OPENDP_POLARS_LIB_PATH with histogram scorer"]
fn python_quantile_ffi_round_trip() -> Fallible<()> {
    use crate::{
        measurements::{
            expr_dp_mean::DPMeanShim, expr_dp_median::DPMedianShim,
            expr_dp_quantile::DPQuantileShim, expr_dp_sum::DPSumShim,
        },
        polars::{OpenDPPlugin, apply_plugin},
    };
    let lib = std::env::var("OPENDP_POLARS_LIB_PATH").expect("set OPENDP_POLARS_LIB_PATH");
    let template = Expr::Function {
        input: vec![],
        function: FunctionExpr::FfiPlugin {
            lib: lib.into(),
            symbol: NoisePlugin::NAME.into(),
            kwargs: Default::default(),
            flags: NoisePlugin::function_options(),
        },
    };
    let q = |single: bool| {
        apply_plugin(
            vec![
                col("value"),
                lit(0.5),
                lit(Series::new(
                    "".into(),
                    if single { vec![3i32] } else { vec![1, 3, 6] },
                )),
                lit(0.),
            ],
            template.clone(),
            DPQuantileShim,
        )
        .alias("quantile")
    };
    let mean = || {
        apply_plugin(
            vec![col("value"), lit(0), lit(5), lit(0.)],
            template.clone(),
            DPMeanShim,
        )
        .alias("mean")
    };
    let sum = || {
        apply_plugin(
            vec![col("value"), lit(0), lit(5), lit(0.)],
            template.clone(),
            DPSumShim,
        )
        .alias("sum")
    };
    let input = data()?;
    let public = df!("key" => [2i32,0,2,1])?.lazy();
    let joined = input
        .clone()
        .group_by([col("key")])
        .agg([q(true), sum(), mean()])
        .join(public, [col("key")], [col("key")], JoinType::Right.into());
    let empty = df!("key" => Vec::<i32>::new(), "value" => Vec::<i32>::new())?.lazy();
    let median = apply_plugin(
        vec![
            col("value"),
            lit(Series::new("".into(), [1i32, 3, 6])),
            lit(0.),
        ],
        template.clone(),
        DPMedianShim,
    )
    .alias("median");
    let removed = apply_plugin(
        vec![
            col("value").filter(col("value").gt(lit(10))),
            lit(0.5),
            lit(Series::new("".into(), [3i32])),
            lit(0.),
        ],
        template.clone(),
        DPQuantileShim,
    )
    .alias("removed");
    let plans = [
        (
            resolve(
                &input,
                input
                    .clone()
                    .group_by([col("key")])
                    .agg([removed, sum(), mean()]),
                true,
                None,
            )?,
            "assert result == {'key':[0,1], 'removed':[3,3], 'sum':[6,3], 'mean':[3.0,3.0]}, result",
        ),
        (
            resolve(&input, input.clone().select([median]), false, None)?,
            "assert result == {'median':[3]}, result",
        ),
        (
            resolve(
                &input,
                input
                    .clone()
                    .select([(q(false) - mean()).alias("difference")]),
                false,
                None,
            )?,
            "assert result == {'difference': [0.0]}, result",
        ),
        (
            resolve(
                &input,
                input.clone().group_by([col("key")]).agg([
                    q(false),
                    sum(),
                    mean(),
                    q(false).alias("independent"),
                ]),
                true,
                None,
            )?,
            "assert result == {'key':[0,1], 'quantile':[3,3], 'sum':[6,3], 'mean':[3.0,3.0], 'independent':[3,3]}, result",
        ),
        (
            resolve(&input, joined, false, None)?,
            "assert result['key'] == [0,1,2] and result['quantile'] == [3,3,3] and result['sum'] == [6,3,0], result\nassert result['mean'][:2] == [3.0,3.0] and math.isnan(result['mean'][2]), result",
        ),
        (
            resolve(&empty, empty.clone().select([q(true), sum()]), false, None)?,
            "assert result == {'quantile':[3], 'sum':[0]}, result",
        ),
    ];
    for (plan, expected) in plans {
        let dir = WorkDir::new();
        let (heads, continuation) = split_plan(plan, &dir.0)?;
        assert!(!dir.0.exists());
        fs::create_dir_all(&dir.0).unwrap();
        let serialized = dir.0.join("continuation.bin");
        continuation
            .serialize_versioned(fs::File::create(&serialized).unwrap(), Default::default())?;
        let mut bytes = vec![];
        heads.serialize_versioned(&mut bytes, Default::default())?;
        let restored = DslPlan::deserialize_versioned(bytes.as_slice())?;
        assert!(sinks(&restored).iter().all(|(_, p)| !p.exists()));
        python(
            "import polars as pl,sys\npl.LazyFrame.deserialize(sys.argv[1])",
            &[&serialized],
        );
        materialize(&restored, &dir.0)?;
        python(
            &format!(
                "import polars as pl,sys,math\nresult=pl.LazyFrame.deserialize(sys.argv[1]).collect()\nif 'key' in result.columns: result=result.sort('key')\nresult=result.to_dict(as_series=False)\n{expected}"
            ),
            &[&serialized],
        );
    }
    Ok(())
}

#[test]
fn literal_scorer_inputs_retain_one_value_per_group() -> Fallible<()> {
    for input in [
        data()?,
        df!("key" => Vec::<i32>::new(), "value" => Vec::<i32>::new())?.lazy(),
    ] {
        for grouped in [false, true] {
            let scorer = apply_anonymous_function(
                vec![(lit(1i32) + lit(2i32)).cast(DataType::Int32)],
                DiscreteQuantileScorePlugin {
                    candidates: Series::new("".into(), [1i32, 3, 6]),
                    alpha: (1, 2),
                    size_limit: 10,
                },
            );
            let expected = if grouped {
                input
                    .clone()
                    .group_by([col("key")])
                    .agg([scorer.clone().alias("scores")])
            } else {
                input.clone().select([scorer.clone().alias("scores")])
            };
            let randomized = apply_anonymous_function(
                vec![scorer],
                NoisyMaxPlugin {
                    replacement: false,
                    scale: 0.,
                    negate: true,
                },
            )
            .alias("index");
            let query = if grouped {
                input.clone().group_by([col("key")]).agg([randomized])
            } else {
                input.clone().select([randomized])
            };
            let dir = WorkDir::new();
            let (heads, continuation) = split_plan(query.logical_plan, &dir.0)?;
            materialize(&heads, &dir.0)?;
            let DslPlan::Select { input: local, .. } = continuation else {
                panic!()
            };
            let DslPlan::Select { input: scoring, .. } = local.as_ref() else {
                panic!()
            };
            let name = match scoring.as_ref() {
                DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => {
                    expr[0].clone().meta().output_name()?
                }
                _ => panic!(),
            };
            let mut exprs = vec![];
            if grouped {
                exprs.push(col("key"));
            }
            exprs.push(col(name).alias("scores"));
            let actual = LazyFrame::from(scoring.as_ref().clone()).select(exprs);
            let (actual, expected) = if grouped {
                (
                    actual.sort(["key"], Default::default()),
                    expected.sort(["key"], Default::default()),
                )
            } else {
                (actual, expected)
            };
            assert_eq!(actual.collect()?, expected.collect()?);
            let q = lit(3i32)
                .dp()
                .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))
                .alias("q");
            let query = if grouped {
                input.clone().group_by([col("key")]).agg([q])
            } else {
                input.clone().select([q])
            };
            compare(resolve(&input, query, grouped, None)?, 1, grouped)?;
        }
    }
    Ok(())
}

/// Compare the reconstructed arrays, not sampled indices: all-removed groups
/// have tied zero scores even when the requested noise scale is zero.
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
    assert_eq!(actual.collect()?, expected.collect()?);
    Ok(())
}

#[test]
fn aggregation_local_removals_preserve_exact_scores_and_observed_groups() -> Fallible<()> {
    use crate::domains::OptionDomain;
    let keys = [Some(0i32), Some(0), None, None, Some(2)];
    let integers = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", OptionDomain::new(AtomDomain::<i32>::default())),
        SeriesDomain::new("value", OptionDomain::new(AtomDomain::<i32>::default())),
    ])?
    .with_margin(Margin::select().with_max_length(10))?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_invariant_keys(),
    )?;
    let floats = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", OptionDomain::new(AtomDomain::<i32>::default())),
        SeriesDomain::new("value", OptionDomain::new(AtomDomain::<f64>::default())),
    ])?
    .with_margin(Margin::select().with_max_length(10))?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_invariant_keys(),
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
            let resolve_case = |query| -> Fallible<DslPlan> {
                Ok(make_private_lazyframe(
                    domain.clone(),
                    SymmetricDistance,
                    Approximate(MaxDivergence),
                    query,
                    Some(1.),
                    None,
                )?
                .invoke(&input)?
                .lazyframe()
                .logical_plan)
            };
            let q = source
                .clone()
                .dp()
                .quantile(0.5, Series::new("".into(), [1i32, 3, 6]), Some(0.))
                .alias("q");
            let query = if grouped {
                input.clone().group_by([col("key")]).agg([q])
            } else {
                input.clone().select([q])
            };
            compare_resolved_score_arrays(resolve_case(query)?, grouped)?;
            // Singleton candidates make zero-observation complete results exact.
            // The scalar count still counts original rows, not filtered values.
            let q = source
                .clone()
                .dp()
                .quantile(0.5, Series::new("".into(), [3i32]), Some(0.))
                .alias("q");
            let query = if grouped {
                input.clone().group_by([col("key")]).agg([q, count()])
            } else {
                input.clone().select([q, count()])
            };
            compare(resolve_case(query.clone())?, 2, grouped)?;
            if grouped {
                let public = df!("key" => [Some(9i32),Some(0),None,Some(2),Some(9)])?.lazy();
                let joined = query.join(public, [col("key")], [col("key")], JoinType::Right.into());
                compare(resolve_case(joined)?, 3, true)?;
            }
        }
    }
    Ok(())
}

/// Requires actual sibling polars-to-ibis installed in PYTHON, not a test adapter.
#[cfg(feature = "polars-ffi")]
#[test]
#[ignore = "requires matching plugin and actual polars-to-ibis with DuckDB"]
fn python_sparse_histogram_ibis_round_trip() -> Fallible<()> {
    use crate::{
        measurements::{
            expr_dp_mean::DPMeanShim, expr_dp_quantile::DPQuantileShim, expr_dp_sum::DPSumShim,
        },
        polars::{OpenDPPlugin, apply_plugin},
    };
    let lib = std::env::var("OPENDP_POLARS_LIB_PATH").expect("set OPENDP_POLARS_LIB_PATH");
    let template = Expr::Function {
        input: vec![],
        function: FunctionExpr::FfiPlugin {
            lib: lib.into(),
            symbol: NoisePlugin::NAME.into(),
            kwargs: Default::default(),
            flags: NoisePlugin::function_options(),
        },
    };
    let q = |input: Expr, singleton: bool| {
        apply_plugin(
            vec![
                input,
                lit(0.5),
                lit(Series::new(
                    "".into(),
                    if singleton { vec![3i32] } else { vec![1, 3, 6] },
                )),
                lit(0.),
            ],
            template.clone(),
            DPQuantileShim,
        )
        .alias("q")
    };
    let s = || {
        apply_plugin(
            vec![col("value"), lit(0), lit(5), lit(0.)],
            template.clone(),
            DPSumShim,
        )
        .alias("sum")
    };
    let m = || {
        apply_plugin(
            vec![col("value"), lit(0), lit(5), lit(0.)],
            template.clone(),
            DPMeanShim,
        )
        .alias("mean")
    };
    let input = data()?;
    let empty = df!("key" => Vec::<i32>::new(), "value" => Vec::<i32>::new())?.lazy();
    let mut plans = vec![
        resolve(
            &input,
            input
                .clone()
                .group_by([col("key")])
                .agg([q(col("value"), false), s(), m()]),
            true,
            None,
        )?,
        resolve(
            &input,
            input
                .clone()
                .select([(q(col("value"), false) - m()).alias("difference")]),
            false,
            None,
        )?,
        resolve(
            &input,
            input.clone().group_by([col("key")]).agg([
                q(col("value").filter(col("value").gt(lit(10))), true),
                s(),
                m(),
            ]),
            true,
            None,
        )?,
        resolve(
            &input,
            input
                .clone()
                .group_by([col("key")])
                .agg([q(col("value").filter(col("value").gt(lit(1))), true), s()]),
            true,
            None,
        )?,
        resolve(
            &empty,
            empty.clone().select([q(col("value"), true), s()]),
            false,
            None,
        )?,
        resolve(
            &empty,
            empty
                .clone()
                .group_by([col("key")])
                .agg([q(col("value"), true), s()]),
            true,
            None,
        )?,
        resolve(
            &input,
            input.clone().select([q(lit(3i32), false)]),
            false,
            None,
        )?,
        resolve(
            &empty,
            empty.clone().select([q(lit(3i32), false)]),
            false,
            None,
        )?,
    ];
    for input in [input.clone(), empty] {
        let public = df!("key" => [2i32,0,2,1])?.lazy();
        let query = input
            .clone()
            .group_by([col("key")])
            .agg([
                q(col("value").filter(col("value").gt(lit(10))), true),
                s(),
                m(),
            ])
            .join(public, [col("key")], [col("key")], JoinType::Right.into());
        plans.push(resolve(&input, query, false, None)?);
    }
    // Keep observed null keys and all-filtered groups while mixing scalar heads.
    let null_keys = df!("key" => [None, None, Some(1i32)], "value" => [1i32, 6, 3])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new(
            "key",
            crate::domains::OptionDomain::new(AtomDomain::<i32>::default()),
        ),
        SeriesDomain::new("value", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::select().with_max_length(10))?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_invariant_keys(),
    )?;
    plans.push(
        make_private_lazyframe(
            domain,
            SymmetricDistance,
            Approximate(MaxDivergence),
            null_keys
                .clone()
                .group_by([col("key")])
                .agg([q(col("value"), false), s(), m()]),
            Some(1.),
            None,
        )?
        .invoke(&null_keys)?
        .lazyframe()
        .logical_plan,
    );
    let floats = df!("key" => [0i32,0,0,1,1], "value" => [Some(1f64),None,Some(f64::NAN),Some(3.),Some(6.)])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new(
            "value",
            crate::domains::OptionDomain::new(AtomDomain::<f64>::default()),
        ),
    ])?
    .with_margin(Margin::select().with_max_length(10).with_max_groups(1))?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_max_groups(2)
            .with_invariant_keys(),
    )?;
    for value in [
        col("value"),
        col("value").drop_nulls().drop_nans(),
        col("value").drop_nans(),
        col("value").drop_nulls(),
        col("value").filter(col("value").gt(lit(10))).drop_nans(),
    ] {
        let quantile = apply_plugin(
            vec![
                value,
                lit(0.5),
                lit(Series::new("".into(), [3f64])),
                lit(0.),
            ],
            template.clone(),
            DPQuantileShim,
        )
        .alias("q");
        plans.push(
            make_private_lazyframe(
                domain.clone(),
                SymmetricDistance,
                Approximate(MaxDivergence),
                floats
                    .clone()
                    .group_by([col("key")])
                    .agg([quantile, s(), m()]),
                Some(1.),
                None,
            )?
            .invoke(&floats)?
            .lazyframe()
            .logical_plan,
        );
    }
    let large =
        df!("key" => [0i32;5], "value" => [0u64,1<<53,(1<<53)+1,u64::MAX-1,u64::MAX])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<u64>::default()),
    ])?
    .with_margin(Margin::select().with_max_length(10))?;
    // Resolve the existing score/noisy-max/index shims directly: DPQuantile's
    // automatic floating midpoint fill would itself coerce these adjacent u64s.
    let candidates = Series::new("".into(), [1u64 << 53, (1 << 53) + 1, u64::MAX - 1]);
    let large_score = apply_plugin(
        vec![col("value"), lit(0.5), lit(candidates.clone())],
        template.clone(),
        crate::transformations::expr_discrete_quantile_score::DiscreteQuantileScoreShim,
    );
    let large_index = apply_plugin(
        vec![large_score, lit(true), lit(0.)],
        template.clone(),
        crate::measurements::expr_noisy_max::NoisyMaxShim,
    );
    let large_q = apply_plugin(
        vec![large_index, lit(candidates)],
        template.clone(),
        crate::measurements::expr_index_candidates::IndexCandidatesShim,
    )
    .alias("q");
    plans.push(
        make_private_lazyframe(
            domain,
            SymmetricDistance,
            Approximate(MaxDivergence),
            large.clone().select([large_q]),
            Some(1.),
            None,
        )?
        .invoke(&large)?
        .lazyframe()
        .logical_plan,
    );
    let threshold_count = apply_plugin(
        vec![len(), lit(0.)],
        template.clone(),
        crate::measurements::expr_noise::NoiseShim,
    )
    .alias("count");
    plans.push(resolve(
        &input,
        input
            .clone()
            .group_by([col("key")])
            .agg([q(col("value"), false), s(), threshold_count]),
        false,
        Some(1),
    )?);
    let collision = input
        .clone()
        .group_by([col("key")])
        .agg([
            q(col("value"), false).alias("__opendp_split_0"),
            s().alias("__opendp_split_1"),
        ])
        .join(
            df!("key" => [0i32,1], "__opendp_split_2" => [4i32,5])?.lazy(),
            [col("key")],
            [col("key")],
            JoinType::Right.into(),
        );
    plans.push(resolve(&input, collision, false, None)?);
    // Regression: stability analysis forces the default strict cast to
    // NonStrict. Overflow contributes the explicit zero fill, not a SQL error.
    let overflowing = df!("key" => [0i32,0,0,1,1], "value" => [128i32,-129,0,300,-300])?.lazy();
    for grouped in [false, true] {
        let cast_q = apply_plugin(
            vec![
                col("value").cast(DataType::Int8).fill_null(lit(0)),
                lit(0.5),
                lit(Series::new("".into(), [-1i8, 0, 1])),
                lit(0.),
            ],
            template.clone(),
            DPQuantileShim,
        )
        .alias("q");
        let query = if grouped {
            overflowing.clone().group_by([col("key")]).agg([cast_q])
        } else {
            overflowing.clone().select([cast_q])
        };
        let plan = resolve(&overflowing, query, grouped, None)?;
        let mut non_strict = false;
        walk(&plan, &mut |node| {
            let exprs = match node {
                DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => {
                    expr.as_slice()
                }
                _ => &[],
            };
            non_strict |= exprs.iter().flat_map(|expr| expr.into_iter()).any(|expr| matches!(
                expr,
                Expr::Cast { dtype, options: polars::chunked_array::cast::CastOptions::NonStrict, .. }
                    if dtype.as_literal() == Some(&DataType::Int8)
            ));
        });
        assert!(
            non_strict,
            "the actual resolved overflow cast must be NonStrict"
        );
        plans.push(plan);
    }
    // Dynamic literals coerce contextually to Float32 at each operation. A
    // final Float32 cast cannot repair the erroneous ~1.49e-9 Double residual.
    let weak_floats = df!("key" => [0i32,0,0,1,1], "value" => [None,Some(0.1f32),Some(f32::NAN),Some(0.1f32),Some(0.1f32)])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new(
            "value",
            crate::domains::OptionDomain::new(AtomDomain::<f32>::default()),
        ),
    ])?
    .with_margin(Margin::select().with_max_length(10))?
    .with_margin(
        Margin::by(["key"])
            .with_max_length(10)
            .with_invariant_keys(),
    )?;
    for (value, candidates) in [
        (
            col("value").fill_null(lit(0.1f64)).fill_nan(lit(0.1f64)),
            Series::new("".into(), [0f32, 0.1, 1.]),
        ),
        (
            (col("value").fill_null(lit(0.1f64)) - lit(0.1f64)).fill_nan(lit(0f64)),
            Series::new("".into(), [-1e-10f32, 0., 1e-10]),
        ),
    ] {
        for grouped in [false, true] {
            let quantile = apply_plugin(
                vec![value.clone(), lit(0.5), lit(candidates.clone()), lit(0.)],
                template.clone(),
                DPQuantileShim,
            )
            .alias("q");
            let query = if grouped {
                weak_floats.clone().group_by([col("key")]).agg([quantile])
            } else {
                weak_floats.clone().select([quantile])
            };
            let plan = make_private_lazyframe(
                domain.clone(),
                SymmetricDistance,
                Approximate(MaxDivergence),
                query,
                Some(1.),
                None,
            )?
            .invoke(&weak_floats)?
            .lazyframe()
            .logical_plan;
            let mut scorers = 0;
            walk(&plan, &mut |node| {
                let exprs = match node {
                    DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => {
                        expr.as_slice()
                    }
                    _ => &[],
                };
                for expr in exprs.iter().flat_map(|expr| expr.into_iter()) {
                    if let Some((_, scorer)) =
                        match_trusted_plugin::<DiscreteQuantileScorePlugin>(expr).unwrap()
                    {
                        // Resolution casts candidates to the active input dtype.
                        assert_eq!(scorer.candidates.dtype(), &DataType::Float32);
                        scorers += 1;
                    }
                }
            });
            assert_eq!(scorers, 1);
            plans.push(plan);
        }
    }
    for (case, plan) in plans.into_iter().enumerate() {
        let dir = WorkDir::new();
        let (heads, continuation) = split_plan(plan.clone(), &dir.0)?;
        fs::create_dir_all(&dir.0).unwrap();
        for (index, (head, _)) in sinks(&heads).iter().enumerate() {
            head.serialize_versioned(
                fs::File::create(dir.0.join(format!("head_{index}.bin"))).unwrap(),
                Default::default(),
            )?;
        }
        if case == 0 {
            if let Ok(path) = std::env::var("OPENDP_SPLIT_WIRE_FIXTURE") {
                python(
                    "import polars as pl,sys,warnings\nwarnings.simplefilter('ignore')\nopen(sys.argv[2],'w').write(pl.LazyFrame.deserialize(sys.argv[1]).serialize(format='json'))",
                    &[&dir.0.join("head_0.bin"), Path::new(&path)],
                );
            }
        }
        for (index, (head, _)) in sinks(&heads).iter().enumerate() {
            IpcWriter::new(
                fs::File::create(dir.0.join(format!("reference_{index}.arrow"))).unwrap(),
            )
            .finish(&mut LazyFrame::from((*head).clone()).collect()?)?;
        }
        IpcWriter::new(fs::File::create(dir.0.join("expected.arrow")).unwrap())
            .finish(&mut LazyFrame::from(plan).collect()?)?;
        continuation.serialize_versioned(
            fs::File::create(dir.0.join("continuation.bin")).unwrap(),
            Default::default(),
        )?;
        python(
            include_str!("test_ibis.py"),
            &[&dir.0, Path::new(&case.to_string())],
        );
    }
    Ok(())
}

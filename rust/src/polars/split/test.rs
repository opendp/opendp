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
        let DslPlan::Select { expr, .. } = &continuation else {
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

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

/// Serialize in Rust before the file exists, then materialize and execute in Python.
#[test]
fn ipc_before_materialization() -> Fallible<()> {
    let dir = WorkDir::new();
    fs::create_dir_all(&dir.0).unwrap();
    let ipc = dir.0.join("missing.arrow");
    let plan = ipc_scan(&ipc)?;
    let serialized = dir.0.join("continuation.bin");
    plan.serialize_versioned(fs::File::create(&serialized).unwrap(), Default::default())?;
    assert!(!ipc.exists());
    python(
        "import polars as pl, sys\npl.DataFrame({'x': [1, 2]}).write_ipc(sys.argv[1])\nassert pl.LazyFrame.deserialize(sys.argv[2]).collect().to_dict(as_series=False) == {'x': [1, 2]}",
        &[&ipc, &serialized],
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

/// Rust execution is only a test oracle/materializer. The splitter itself never
/// calls collect, resolves a schema, or adds an execution-engine dependency.
fn materialize(split: &SplitPlan, dir: &Path) -> Fallible<()> {
    fs::create_dir_all(dir).unwrap();
    for (fragment, path) in &split.fragments {
        let mut df = LazyFrame::from(fragment.clone()).collect()?;
        IpcWriter::new(fs::File::create(path).unwrap()).finish(&mut df)?;
    }
    Ok(())
}

fn compare(plan: DslPlan, expected_fragments: usize, grouped: bool) -> Fallible<SplitPlan> {
    let dir = WorkDir::new();
    let split = split_plan(plan.clone(), &dir.0)?;
    assert!(
        !dir.0.exists(),
        "splitter must not create the working directory"
    );
    assert_eq!(split.fragments.len(), expected_fragments);
    assert_eq!(noise_count(&split.base), 0);
    assert_eq!(
        noise_parameters(&split.continuation),
        noise_parameters(&plan)
    );
    for (fragment, _) in &split.fragments {
        assert_eq!(noise_count(fragment), 0);
    }
    walk(&split.continuation, &mut |node| {
        assert!(!matches!(node, DslPlan::DataFrameScan { .. }))
    });
    materialize(&split, &dir.0)?;
    let collect = |plan| -> Fallible<DataFrame> {
        let lf = LazyFrame::from(plan);
        Ok(if grouped {
            lf.sort(["key"], Default::default())
        } else {
            lf
        }
        .collect()?)
    };
    assert_eq!(collect(split.continuation.clone())?, collect(plan)?);
    Ok(split)
}

#[test]
fn ungrouped_sum_and_private_select() -> Fallible<()> {
    let input = data()?;
    let split = compare(
        resolve(&input, input.clone().select([sum()]), false, None)?,
        1,
        false,
    )?;
    let DslPlan::Select { expr, .. } = &split.fragments[0].0 else {
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
        let split = compare(plan, 1, false)?;
        let DslPlan::Select { expr, .. } = &split.fragments[0].0 else {
            panic!()
        };
        assert_eq!(expr.len(), noise_count(&split.continuation));
        let DslPlan::Select { expr, .. } = &split.continuation else {
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
fn grouped_count_and_threshold() -> Fallible<()> {
    let input = data()?;
    for threshold in [None, Some(1)] {
        let query = input.clone().group_by([col("key")]).agg([count()]);
        let split = compare(
            resolve(&input, query, threshold.is_none(), threshold)?,
            1,
            true,
        )?;
        let DslPlan::GroupBy { keys, .. } = &split.fragments[0].0 else {
            panic!()
        };
        assert_eq!(keys, &[col("key")]);
        if threshold.is_some() {
            assert!(matches!(&split.continuation, DslPlan::Filter { .. }));
            assert!(!matches!(&split.fragments[0].0, DslPlan::Filter { .. }));
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
        let split = compare(resolve(&input, query, false, None)?, 2, true)?;
        assert!(matches!(&split.continuation, DslPlan::HStack { .. }));
        let mut distinct = 0;
        walk(&split.continuation, &mut |p| {
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
    let split = compare(plan, 1, false)?;
    assert!(matches!(&split.base, DslPlan::HStack { .. }));
    let DslPlan::Select { expr, .. } = &split.fragments[0].0 else {
        panic!()
    };
    assert_ne!(
        expr[0].clone().meta().output_name()?.as_str(),
        "__opendp_split_0"
    );
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
        let split = split_plan(plan, &dir.0)?;
        assert!(!dir.0.exists());
        assert_eq!(format!("{:?}", split.base), format!("{:?}", base));
        let DslPlan::Select { input, .. } = &split.fragments[0].0 else {
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
    let split = compare(query.logical_plan, 1, true)?;
    assert_eq!(
        format!("{:?}", split.base),
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
    let split = split_plan(plan, &dir.0)?;
    assert_eq!(noise_count(&split.continuation), 2);
    materialize(&split, &dir.0)?;
    let output = LazyFrame::from(split.continuation).collect()?;
    assert_eq!(output.shape(), (1, 2));
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
    let dir = WorkDir::new();
    let split = split_plan(plan, &dir.0)?;
    let DslPlan::GroupBy { aggs, .. } = &split.fragments[0].0 else {
        panic!()
    };
    assert_eq!(aggs.len(), 1, "literal input must not become a column");
    materialize(&split, &dir.0)?;
    let output = LazyFrame::from(split.continuation).collect()?;
    assert_eq!(output.column("n")?.n_unique()?, 1);
    Ok(())
}

#[test]
fn unsupported_shapes_fail_explicitly() -> Fallible<()> {
    let input = data()?;
    let callback = callback_expr();
    let hidden_noise = col("value").quantile(
        noisy(lit(0i64).cast(DataType::Int64), 0.),
        QuantileMethod::Nearest,
    );
    let plans = [
        input
            .clone()
            .select([
                noisy(col("value").sum(), 0.).cast(DataTypeExpr::OfExpr(Box::new(col("value"))))
            ])
            .logical_plan,
        input
            .clone()
            .select([noisy(hidden_noise.sum(), 0.)])
            .logical_plan,
        input
            .clone()
            .select([noisy(callback.sum(), 0.)])
            .logical_plan,
        input
            .clone()
            .select([col("value").dp().sum((lit(0), lit(5)), None)])
            .logical_plan, // unresolved shim
        input
            .clone()
            .select([noisy(col("value"), 1.), noisy(len(), 1.).alias("n")])
            .logical_plan, // mixed vector/scalar
        input
            .clone()
            .select([noisy(col("value"), 1.).sum()])
            .logical_plan, // aggregate after noise
        input
            .clone()
            .group_by([col("key")])
            .agg([noisy(col("value").sum(), 0.).sort(Default::default())])
            .logical_plan,
        input
            .clone()
            .group_by([col("key")])
            .agg([noisy(len(), 0.), col("key").alias("raw_key")])
            .logical_plan,
        input
            .clone()
            .select([noisy(col("value").implode(), 1.)])
            .logical_plan, // unsupported scoring/list aggregate
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
    use crate::polars::OpenDPPlugin;
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
    let query = input
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
    let dir = WorkDir::new();
    let split = split_plan(query.logical_plan, &dir.0)?;
    fs::create_dir_all(&dir.0).unwrap();
    let serialized = dir.0.join("continuation.bin");
    split
        .continuation
        .serialize_versioned(fs::File::create(&serialized).unwrap(), Default::default())?;
    assert!(split.fragments.iter().all(|(_, p)| !p.exists()));
    materialize(&split, &dir.0)?;
    python(
        "import polars as pl, sys\nresult = pl.LazyFrame.deserialize(sys.argv[1]).collect().to_dict(as_series=False)\nassert result == {'key': [0], 'sum': [7], 'mean': [3.5]}, result",
        &[&serialized],
    );
    Ok(())
}

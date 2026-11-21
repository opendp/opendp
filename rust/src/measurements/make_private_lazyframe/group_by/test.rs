use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::error::ErrorVariant::MakeMeasurement;
use crate::error::*;
use crate::measurements::{make_private_expr, make_private_lazyframe};
use crate::measures::MaxDivergence;
use crate::metrics::{L0PInfDistance, SymmetricDistance};
use crate::polars::PrivacyNamespace;
#[cfg(not(feature = "polars-ffi"))]
use crate::traits::samplers::test::{check_chi_square, check_kolmogorov_smirnov};
use polars::prelude::*;

use super::*;

#[test]
fn test_reject_float_grouping_keys() -> Fallible<()> {
    macro_rules! check {
        ($ty:ty, $values:expr) => {{
            let domain = LazyFrameDomain::new(vec![
                SeriesDomain::new("key", AtomDomain::<$ty>::default()),
            ])?;
            let lf = df!("key" => $values)?.lazy();
            for key in [col("key"), col("key").alias("aliased")] {
                let err = make_private_lazyframe(
                    domain.clone(),
                    SymmetricDistance,
                    Approximate(MaxDivergence),
                    lf.clone().group_by(&[key]).agg(&[len().dp().noise(None)]),
                    Some(1.),
                    Some(10),
                )
                .map(|_| ())
                .unwrap_err();

                let message = err.message.as_deref().unwrap_or_default();
                assert!(message.contains("Floating-point grouping keys"), "{err:?}");
                assert_eq!(err.variant, MakeMeasurement);
            }
        }};
    }

    check!(f32, [-0.0f32, 0.0f32]);
    check!(f64, [f64::NAN, f64::from_bits(0x7ff8_0000_0000_0001)]);
    Ok(())
}

#[test]
fn test_contains_float_nested() {
    assert!(contains_float(&DataType::Float16));
    assert!(contains_float(&DataType::Array(
        Box::new(DataType::Float32),
        2
    )));
    assert!(contains_float(&DataType::List(Box::new(DataType::Float64))));
    assert!(contains_float(&DataType::Struct(vec![Field::new(
        "nested".into(),
        DataType::Array(Box::new(DataType::Float32), 2),
    )])));

    assert!(!contains_float(&DataType::Int32));
    assert!(!contains_float(&DataType::Array(
        Box::new(DataType::Int32),
        2
    )));
}

#[test]
fn test_reject_nested_categorical_grouping_key() -> Fallible<()> {
    let categories = Categories::global();
    let categorical = DataType::Categorical(categories.clone(), categories.mapping());
    assert!(contains_categorical(&categorical));
    assert!(contains_categorical(&DataType::List(Box::new(
        categorical.clone()
    ))));
    assert!(contains_categorical(&DataType::Struct(vec![Field::new(
        "nested".into(),
        categorical.clone(),
    )])));
    assert!(!contains_categorical(&DataType::String));

    let schema = Schema::from_iter([Field::new(
        "key".into(),
        DataType::Array(Box::new(categorical), 2),
    )]);
    let domain = LazyFrameDomain::new_from_schema(schema.clone())?;
    let lf = DataFrame::empty_with_schema(&schema).lazy();
    let err = make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        lf.group_by(&[col("key")]).agg(&[len().dp().noise(None)]),
        Some(1.),
        Some(10),
    )
    .map(|_| ())
    .unwrap_err();
    assert_eq!(err.variant, MakeMeasurement);
    assert!(
        err.message
            .as_deref()
            .unwrap_or_default()
            .contains("Categories are data-dependent"),
        "{err:?}"
    );
    Ok(())
}

#[test]
fn test_cast_float_grouping_keys_allowed() -> Fallible<()> {
    let domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("key", AtomDomain::<f64>::default())])?;
    let lf = df!("key" => [-0.0f64, 0.0, f64::NAN])?.lazy();

    make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        lf.group_by(&[col("key").cast(DataType::Int64)])
            .agg(&[len().dp().noise(None)]),
        Some(1.),
        Some(10),
    )?;
    Ok(())
}

#[test]
fn test_float_non_grouping_column_allowed() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("key", AtomDomain::<i32>::default()),
        SeriesDomain::new("value", AtomDomain::<f64>::default()),
    ])?;
    let lf = df!("key" => [1i32, 1], "value" => [-0.0f64, f64::NAN])?.lazy();
    make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        lf.group_by(&[col("key")]).agg(&[len().dp().noise(None)]),
        Some(1.),
        Some(10),
    )?;
    Ok(())
}

#[test]
fn test_aggregate() -> Fallible<()> {
    let lf_domain = DslPlanDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
        SeriesDomain::new("C", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["A", "C"]).with_invariant_keys())?;

    let lf = df!(
        "A" => &[1i32, 2, 2],
        "B" => &[1.0f64, 2.0, 2.0],
        "C" => &[8i32, 9, 10],)?
    .lazy();

    let error_variant_res = make_private_group_by::<_, _>(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        lf.group_by(&[col("A"), col("C")])
            .agg(&[col("B").sum()])
            .logical_plan,
        Some(1.),
        None,
    )
    .map(|_| ())
    .unwrap_err()
    .variant;

    assert_eq!(MakeMeasurement, error_variant_res);

    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_stable_keys_puredp() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => [[1i32; 1000], [2; 1000]].concat())?.lazy();

    let meas = make_private_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        Approximate(MaxDivergence),
        lf.clone()
            .group_by(&[col("A")])
            .agg(&[len().dp().noise(None)]),
        Some(1.),
        Some(40),
    )?;

    let counts = meas.invoke(&lf)?;
    let params = meas.map(&1.into())?;

    println!("counts {}", counts.collect()?);
    println!("params {:?}", params);

    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_stable_keys_zcdp() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;

    let lf = df!("A" => [[1i32; 1000], [2; 1000]].concat())?.lazy();

    let meas = make_private_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        Approximate(ZeroConcentratedDivergence),
        lf.clone()
            .group_by(&[col("A")])
            .agg(&[len().dp().noise(None)]),
        Some(1.),
        Some(40),
    )?;

    let counts = meas.invoke(&lf)?;
    let params = meas.map(&1.into())?;

    println!("counts {}", counts.collect()?);
    println!("params {:?}", params);

    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_stable_keys_delta_empirical() -> Fallible<()> {
    const N: u32 = 2048;
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<u32>::default())])?
        .with_margin(Margin::by(["A"]).with_max_length(1))?;
    let data = df!("A" => &(0..N).collect::<Vec<_>>())?.lazy();

    // Match N, scale, threshold, and tolerance in
    // noise_threshold/distribution/{laplace,gaussian}/test.rs::
    // test_{laplace,gaussian}_threshold_delta_empirical (positive case).
    // Each group is unstable relative to a neighboring frame without its row.
    // Releasing a group at the threshold would exceed the reported delta.
    macro_rules! check {
        ($measure:expr) => {{
            let meas = make_private_lazyframe(
                domain.clone(),
                FrameDistance(SymmetricDistance),
                Approximate($measure),
                data.clone()
                    .group_by(&[col("A")])
                    .agg(&[len().dp().noise(None)]),
                Some(0.5),
                Some(2),
            )?;
            let delta = meas.map(&1.into())?.1;
            let release_rate = meas.invoke(&data)?.collect()?.height() as f64 / f64::from(N);
            assert!(
                release_rate <= delta + 0.05,
                "release rate {release_rate} exceeds delta {delta}"
            );
        }};
    }
    check!(MaxDivergence);
    check!(ZeroConcentratedDivergence);
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_explicit_keys() -> Fallible<()> {
    static N_SAMPLES: u32 = 5000;
    static N_CANDIDATES: usize = 10;

    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<u32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(1))?;

    let lf = df!("A" => &[0u32], "B" => &[0.0f64])?.lazy();
    let mut key_values = (0u32..N_SAMPLES).collect::<Vec<_>>();
    key_values.push(0); // Public key zero occurs twice; output must still have N_SAMPLES rows.
    let keys = df!("A" => key_values)?.lazy();

    let sum_expr = col("B").dp().sum((lit(0.0), lit(1.0)), None).alias("sum");
    let candidates = (0..N_CANDIDATES).map(|v| v as f64).collect::<Vec<_>>();
    let median_expr = col("B")
        .fill_nan(0.0)
        .fill_null(0.0)
        .dp()
        .median(Series::new("".into(), candidates.clone()), None)
        .alias("med");
    let meas = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence,
        lf.clone()
            .group_by(&[col("A")])
            .agg(&[sum_expr, median_expr])
            // add a DP join (the constructor adds an imputer to the resulting onceframe)
            .join(keys, [col("A")], [col("A")], JoinType::Right.into()),
        Some(1.),
        None,
    )?;

    let release = meas.invoke(&lf)?.collect()?;
    let lap_samples: Vec<_> = release.column("sum")?.f64()?.iter().flatten().collect();
    let lap_samples = <[f64; 5000]>::try_from(lap_samples).unwrap();

    pub fn laplace_cdf(x: f64) -> f64 {
        if x < 0.0 {
            0.5 * x.exp()
        } else {
            1.0 - 0.5 * (-x).exp()
        }
    }

    check_kolmogorov_smirnov(lap_samples, laplace_cdf)?;

    // check for uniformity of samples (all scores are matching)
    let unif_samples: Vec<_> = release.column("med")?.f64()?.iter().flatten().collect();
    let mut counts = [0; N_CANDIDATES];
    unif_samples.iter().for_each(|&s| counts[s as usize] += 1);
    check_chi_square(
        &counts,
        &[N_SAMPLES as f64 / (N_CANDIDATES as f64); N_CANDIDATES],
    )?;

    Ok(())
}

#[test]
fn test_explicit_key_join_rejects_partial_keys() -> Fallible<()> {
    let data = df!("A" => [0i32], "B" => ["private"])?.lazy();
    let grouped = data
        .group_by(&[col("A"), col("B")])
        .agg(&[len().dp().noise(Some(1.0))]);
    for left in [true, false] {
        let public = df!("A" => [0i32])?.lazy();
        let joined = if left {
            public.join(
                grouped.clone(),
                [col("A")],
                [col("A")],
                JoinType::Left.into(),
            )
        } else {
            grouped
                .clone()
                .join(public, [col("A")], [col("A")], JoinType::Right.into())
        };
        let error = match_group_by(joined.logical_plan).map(|_| ()).unwrap_err();
        assert_eq!(error.variant, MakeMeasurement);
        assert_eq!(
            error.message.as_deref(),
            Some("join keys must match group-by keys")
        );
    }
    Ok(())
}

#[test]
fn test_explicit_key_join_deduplicates_public_keys() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<String>::default()),
    ])?
    .with_margin(Margin::by(["A", "B"]).with_max_length(1))?;
    let data = df!("A" => [0i32], "B" => ["private"])?.lazy();
    let grouped = data
        .clone()
        .group_by([col("A"), col("B")])
        .agg([len().dp().noise(Some(1.0))]);
    let public = df!(
        "A" => [0i32, 0, 1],
        "B" => ["private", "private", "absent"],
        "meta" => ["first", "second", "third"]
    )?
    .lazy();

    for left in [true, false] {
        let joined = if left {
            public.clone().join(
                grouped.clone(),
                [col("A"), col("B")],
                [col("A"), col("B")],
                JoinType::Left.into(),
            )
        } else {
            grouped.clone().join(
                public.clone(),
                [col("A"), col("B")],
                [col("A"), col("B")],
                JoinType::Right.into(),
            )
        };

        assert!(match_group_by(joined.logical_plan.clone())?.is_some());
        let meas = make_private_group_by(
            domain.clone().cast_carrier(),
            FrameDistance(SymmetricDistance),
            MaxDivergence,
            joined.logical_plan,
            Some(1.0),
            None,
        )?;
        let released_plan = meas.invoke(&data.logical_plan)?;
        let DslPlan::HStack { input, .. } = &released_plan else {
            panic!("expected imputation after the join");
        };
        let DslPlan::Join {
            input_left,
            input_right,
            ..
        } = input.as_ref()
        else {
            panic!("expected public-key join");
        };
        let public_keys = if left { input_left } else { input_right };
        let DslPlan::Distinct { options, .. } = public_keys.as_ref() else {
            panic!("expected deduplication on the public side");
        };
        assert_eq!(options.subset, Some(vec![col("A"), col("B")]));

        let release = LazyFrame::from(released_plan).collect()?;
        assert_eq!(release.height(), 2);
        assert!(release.column("meta").is_ok());
    }

    Ok(())
}

#[test]
fn test_explicit_key_join_rejects_collisions() -> Fallible<()> {
    let private = df!("A" => [0i32])?
        .lazy()
        .group_by([col("A")])
        .agg([len().dp().noise(Some(1.0)).alias("count")]);

    for left in [true, false] {
        let keys = df!("A" => [0i32], "count" => [2u32])?.lazy();
        let joined = if left {
            keys.join(
                private.clone(),
                [col("A")],
                [col("A")],
                JoinType::Left.into(),
            )
        } else {
            private
                .clone()
                .join(keys, [col("A")], [col("A")], JoinType::Right.into())
        };
        let err = match_group_by(joined.logical_plan).map(|_| ()).unwrap_err();
        assert_eq!(err.variant, MakeMeasurement);
        assert_eq!(
            err.message.as_deref(),
            Some("public columns must not conflict with private output columns")
        );
    }

    // Even a public join key may shadow an aggregate with the same name.
    let keys = df!("count" => [0i32])?.lazy();
    let joined = private
        .clone()
        .join(keys, [col("A")], [col("count")], JoinType::Right.into());
    let err = match_group_by(joined.logical_plan).map(|_| ()).unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some("public columns must not conflict with private output columns")
    );

    // Different key names are fine when the schemas are otherwise disjoint.
    let keys = df!("K" => [0i32])?.lazy();
    let joined = private.join(keys, [col("A")], [col("K")], JoinType::Right.into());
    assert!(match_group_by(joined.logical_plan)?.is_some());

    // The private key A is coalesced even when public A is not paired with it.
    let private = df!("A" => [0i32], "B" => [1i32])?
        .lazy()
        .group_by([col("A"), col("B")])
        .agg([len().dp().noise(Some(1.0))]);
    let public = df!("A" => [1i32], "B" => [0i32])?.lazy();
    let joined = private.join(
        public,
        [col("A"), col("B")],
        [col("B"), col("A")],
        JoinType::Right.into(),
    );
    assert!(match_group_by(joined.logical_plan)?.is_some());

    Ok(())
}

#[test]
fn test_explicit_key_join_rejects_unsafe_options() -> Fallible<()> {
    let private = df!("A" => [0i32])?
        .lazy()
        .group_by([col("A")])
        .agg([len().dp().noise(Some(1.0))]);
    let keys = df!("A" => [0i32, 1])?.lazy();

    let mut args: JoinArgs = JoinType::Left.into();
    args.coalesce = JoinCoalesce::KeepColumns;
    let joined = keys
        .clone()
        .join(private.clone(), [col("A")], [col("A")], args);
    let err = match_group_by(joined.logical_plan).map(|_| ()).unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some("DP key joins must coalesce join keys")
    );

    // The LazyFrame join builder panics on slicing; construct the DSL directly.
    let joined = private.join(keys, [col("A")], [col("A")], JoinType::Right.into());
    let DslPlan::Join {
        input_left,
        input_right,
        left_on,
        right_on,
        predicates,
        options,
    } = joined.logical_plan
    else {
        panic!("expected join");
    };
    let mut options = (*options).clone();
    options.args.slice = Some((0, 1));
    let joined = DslPlan::Join {
        input_left,
        input_right,
        left_on,
        right_on,
        predicates,
        options: Arc::new(options),
    };
    let err = match_group_by(joined).map(|_| ()).unwrap_err();
    assert_eq!(
        err.message.as_deref(),
        Some("DP key joins cannot be sliced because slicing introduces order dependence")
    );
    Ok(())
}

#[test]
fn test_explicit_key_join_rejects_mismatched_keys() -> Fallible<()> {
    let data = df!("A" => [0i32], "B" => ["private"])?.lazy();
    let mismatched = data
        .group_by(&[col("A"), col("B")])
        .agg(&[len().dp().noise(Some(1.0)).alias("C")])
        .join(
            df!("A" => [0i32], "C" => [0u32])?.lazy(),
            [col("A"), col("C")],
            [col("A"), col("C")],
            JoinType::Right.into(),
        );
    let error = match_group_by(mismatched.logical_plan)
        .map(|_| ())
        .unwrap_err();
    assert_eq!(error.variant, MakeMeasurement);
    assert_eq!(
        error.message.as_deref(),
        Some("join keys must match group-by keys")
    );
    Ok(())
}

#[test]
fn test_explicit_key_join_accepts_computed_keys() -> Fallible<()> {
    let data = df!("A" => [0i32], "B" => ["private"])?.lazy();
    let computed = data
        .group_by(&[col("A").gt(lit(0))])
        .agg(&[len().dp().noise(Some(1.0))])
        .join(
            df!("A" => [true, false])?.lazy(),
            [col("A")],
            [col("A")],
            JoinType::Right.into(),
        );
    assert!(match_group_by(computed.logical_plan)?.is_some());
    Ok(())
}

#[test]
fn test_explicit_key_join_accepts_aliased_keys() -> Fallible<()> {
    let data = df!("A" => [0i32], "B" => ["private"])?.lazy();
    let aliased = data
        .group_by(&[col("A").alias("key")])
        .agg(&[len().dp().noise(Some(1.0))])
        .join(
            df!("key" => [0i32])?.lazy(),
            [col("key")],
            [col("key")],
            JoinType::Right.into(),
        );
    assert!(match_group_by(aliased.logical_plan)?.is_some());
    Ok(())
}

#[test]
fn test_fill_after_filter_privacy_map() -> Fallible<()> {
    // Regression for #3039: dp.sum implicitly fills nulls after a filter.
    // Filling must not restore the public-length invariant lost by the filter.
    let lf = df!("x" => [1i64, 3, 3], "k" => [0i64, 0, 0])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("x", AtomDomain::<i64>::default()),
        SeriesDomain::new("k", AtomDomain::<i64>::default()),
    ])?
    .with_margin(
        Margin::by(["k"])
            .with_invariant_lengths()
            .with_max_length(50)
            .with_max_groups(3),
    )?;

    let filtered = col("x").filter(col("x").gt(lit(1i64)));
    for expr in [
        filtered.clone().dp().sum((lit(2i64), lit(3i64)), Some(1.0)),
        filtered
            .fill_null(2i64)
            .clip(lit(2i64), lit(3i64))
            .sum()
            .dp()
            .noise(Some(1.0)),
    ] {
        let meas = make_private_lazyframe(
            domain.clone(),
            SymmetricDistance,
            MaxDivergence,
            lf.clone().group_by(&[col("k")]).agg(&[expr]),
            None,
            None,
        )?;

        let privacy_loss = meas.map(&2u32)?;
        assert!(
            (6.0..7.0).contains(&privacy_loss),
            "filtering must invalidate public lengths; got privacy loss {privacy_loss}"
        );
    }
    Ok(())
}

fn process_expr(expr: Expr) -> Fallible<Expr> {
    Ok(make_private_expr(
        WildExprDomain {
            columns: vec![],
            context: Context::Aggregation {
                margin: Margin::select(),
            },
        },
        L0PInfDistance(SymmetricDistance),
        MaxDivergence,
        expr,
        Some(1.0),
    )?
    .invoke(&df!["A" => [1, 2]]?.lazy().logical_plan)?
    .expr)
}

#[test]
fn test_find_len_expr() -> Fallible<()> {
    // len expressions supported
    let supported = vec![
        len().dp().noise(None),
        len().cast(DataType::Int64).dp().noise(None),
    ];

    // Supported test cases.
    for supported_expr in &supported {
        let dp_expr = process_expr(supported_expr.clone())?;
        let result = find_len_expr(&vec![dp_expr.clone()], None);
        assert!(
            result.is_ok(),
            "Supported len expression incorrectly not identified: {:?}",
            supported_expr,
        );
    }
    Ok(())
}

#[test]
fn test_max_length_bounds_symmetric_change() -> Fallible<()> {
    #[cfg(not(feature = "polars-ffi"))]
    use crate::core::Domain;

    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<String>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        Margin::by(["A"])
            .with_max_length(3)
            .with_max_groups(1)
            .with_invariant_keys(),
    )?;

    let x = df!("A" => ["a", "a", "a"], "B" => [10i32, 10, 10])?.lazy();
    #[cfg(not(feature = "polars-ffi"))]
    let y = df!("A" => ["a", "a", "a"], "B" => [-10i32, -10, -10])?.lazy();
    #[cfg(not(feature = "polars-ffi"))]
    {
        assert!(domain.member(&x)?);
        assert!(domain.member(&y)?);
    }

    let measurement = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        x.group_by(&[col("A")])
            .agg(&[col("B").dp().sum((lit(-10), lit(10)), Some(30.0))]),
        None,
        None,
    )?;

    // Replacing three rows changes six rows under SymmetricDistance.
    // The clipped sums differ by 60, giving epsilon 60 / 30 = 2.
    assert!((measurement.map(&6)? - 2.0).abs() < 1e-6);
    // A single insertion still incurs the sensitivity of just one row.
    assert!((measurement.map(&1)? - 1.0 / 3.0).abs() < 1e-6);
    Ok(())
}

#[test]
fn test_max_length_overflow() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;
    let margin = Margin::by(["A"])
        .with_max_length(u32::MAX)
        .with_invariant_keys();
    let domain = domain.with_margin(margin)?;

    let lf = df!("A" => [1i32])?.lazy();
    let measurement = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        lf.group_by(&[col("A")]).agg(&[len().dp().noise(Some(1.0))]),
        None,
        None,
    )?;

    assert!(measurement.map(&1).is_err());
    Ok(())
}

#[test]
fn test_max_groups_bounds_changed_keys() -> Fallible<()> {
    #[cfg(not(feature = "polars-ffi"))]
    use crate::core::Domain;
    use crate::metrics::{Bound, Bounds};

    let x = df!("A" => ["a"])?.lazy();
    #[cfg(not(feature = "polars-ffi"))]
    let y = df!("A" => ["b"])?.lazy();
    let public = df!("A" => ["a", "b"])?.lazy();
    let grouped = x
        .clone()
        .group_by([col("A")])
        .agg([len().dp().noise(Some(1.0)).alias("count")]);
    let plan = public.join(grouped, [col("A")], [col("A")], JoinType::Left.into());
    let bounds = Bounds::from(2).with_bound(Bound::by([col("A")]).with_per_group(1));

    for (margin, epsilon) in [
        (Margin::by(["A"]), 2.0),
        (Margin::by(["A"]).with_max_groups(1), 2.0),
        (
            Margin::by(["A"]).with_max_groups(1).with_invariant_keys(),
            1.0,
        ),
    ] {
        let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
            "A",
            AtomDomain::<String>::default(),
        )])?
        .with_margin(margin.clone())?;

        #[cfg(not(feature = "polars-ffi"))]
        if margin.invariant.is_none() {
            // These datasets have disjoint group keys, despite each having one group.
            assert!(domain.member(&x)? && domain.member(&y)?);
        }
        let measurement = make_private_lazyframe(
            domain,
            FrameDistance(SymmetricDistance),
            MaxDivergence,
            plan.clone(),
            None,
            None,
        )?;
        assert_eq!(measurement.map(&bounds)?, epsilon);

        // A genuine pairwise bound should still tighten the number of changed groups.
        let pairwise_bounds = bounds
            .clone()
            .with_bound(Bound::by([col("A")]).with_num_groups(1));
        assert_eq!(measurement.map(&pairwise_bounds)?, 1.0);
    }
    Ok(())
}


#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_group_by_means_share_count() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
        SeriesDomain::new("C", AtomDomain::<f64>::default()),
    ])?
    .with_margin(
        Margin::by(["A"])
            .with_invariant_keys()
            .with_max_length(3)
            .with_max_groups(1),
    )?;

    let data = df!(
        "A" => [1i32, 1, 1],
        "B" => [1.0f64, 2.0, 3.0],
        "C" => [4.0f64, 5.0, 6.0],
    )?
    .lazy();
    let query = data.clone().group_by([col("A")]).agg([
        col("B")
            .dp()
            .mean((lit(0.0), lit(10.0)), None)
            .alias("mean_b"),
        col("C")
            .dp()
            .mean((lit(0.0), lit(10.0)), None)
            .alias("mean_c"),
        crate::polars::dp_len(None, false).alias("count"),
    ]);

    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        query,
        Some(1.0),
        None,
    )?;
    let plan = meas.invoke(&data)?.logical_plan;

    let DslPlan::Select { input, .. } = &plan else {
        panic!("expected final projection");
    };
    let DslPlan::HStack { input, .. } = input.as_ref() else {
        panic!("expected shared-statistic postprocessing");
    };
    let DslPlan::GroupBy { aggs, .. } = input.as_ref() else {
        panic!("expected aggregate of noisy sufficient statistics");
    };
    assert_eq!(
        aggs.iter()
            .filter(|expr| super::matching::is_len_expr(expr, None).is_some())
            .count(),
        1,
        "both means must reuse the same materialized noisy count"
    );

    let release = LazyFrame::from(plan).collect()?;
    assert_eq!(release.width(), 4);
    assert!(release.column("mean_b").is_ok());
    assert!(release.column("mean_c").is_ok());
    assert!(release.column("count").is_ok());
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_shared_means_join_fills_before_postprocessing() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
        SeriesDomain::new("C", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(2).with_max_groups(1))?;

    let data = df!(
        "A" => [0i32, 0],
        "B" => [1.0f64, 2.0],
        "C" => [3.0f64, 4.0],
    )?
    .lazy();
    let keys = df!(
        "A" => [0i32, 1],
        "label" => ["present", "absent"],
    )?
    .lazy();
    let query = data
        .clone()
        .group_by([col("A")])
        .agg([
            col("B").dp().mean((lit(0.0), lit(10.0)), None).alias("mb"),
            col("C").dp().mean((lit(0.0), lit(10.0)), None).alias("mc"),
        ])
        .join(keys, [col("A")], [col("A")], JoinType::Right.into());

    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        query,
        Some(1.0),
        None,
    )?;
    let release = meas.invoke(&data)?.collect()?;
    assert_eq!(release.height(), 2);
    assert_eq!(release.width(), 4);
    for name in ["A", "label", "mb", "mc"] {
        assert!(release.column(name).is_ok(), "missing {name}");
    }
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_mean_only_private_key_filter_synthesizes_count() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(2))?;

    let data = df!(
        "A" => [0i32, 1],
        "B" => [2.0f64, 4.0],
    )?
    .lazy();
    // No user-visible count expression: selection uses the mean's shared
    // noisy denominator, and the intermediate count is removed afterwards.
    let query = data.clone().group_by([col("A")]).agg([
        col("B").dp().mean((lit(0.0), lit(10.0)), None).alias("mean_b"),
    ]);

    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        query,
        Some(1.0),
        Some(3),
    )?;
    let _ = meas.map(&1.into())?;
    let release = meas.invoke(&data)?.collect()?;
    assert_eq!(release.width(), 2);
    assert!(release.column("mean_b").is_ok());
    Ok(())
}


#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_nested_postprocessing_preserves_independent_leaves() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(
        Margin::by(["A"])
            .with_invariant_keys()
            .with_max_length(2)
            .with_max_groups(1),
    )?;
    let data = df!("A" => [0i32, 0], "B" => [2.0f64, 3.0])?.lazy();

    let sum = col("B").dp().sum((lit(0.0), lit(10.0)), Some(1.0));
    let combined = (sum.clone() + sum)
        .cast(DataType::Float64)
        .alias("combined");

    let plan = workload::PrivateAggPlan::new(vec![combined.clone()], &[col("A")], &None)?;
    assert_eq!(plan.aggs.len(), 2, "identical leaves must not yet be deduplicated");
    assert!(plan.count.is_none());

    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        data.clone().group_by([col("A")]).agg([combined]),
        Some(1.0),
        None,
    )?;
    let _ = meas.map(&1.into())?;
    let release = meas.invoke(&data)?.collect()?;
    assert_eq!(release.width(), 2);
    assert_eq!(release.column("combined")?.dtype(), &DataType::Float64);
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_nested_means_remain_independent() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
        SeriesDomain::new("C", AtomDomain::<f64>::default()),
    ])?
    .with_margin(
        Margin::by(["A"])
            .with_invariant_keys()
            .with_max_length(2)
            .with_max_groups(1),
    )?;
    let data = df!(
        "A" => [0i32, 0],
        "B" => [2.0f64, 3.0],
        "C" => [5.0f64, 6.0],
    )?
    .lazy();

    // Nested means remain opaque leaves, so this refactor does not
    // introduce new count sharing or change their privacy accounting.
    let expr = (
        col("B").dp().mean((lit(0.0), lit(10.0)), None)
            + col("C").dp().mean((lit(0.0), lit(10.0)), None)
    )
    .alias("combined");
    let plan = workload::PrivateAggPlan::new(vec![expr.clone()], &[col("A")], &None)?;
    assert_eq!(plan.aggs.len(), 2);
    assert!(plan.count.is_none());

    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        data.clone().group_by([col("A")]).agg([expr]),
        Some(1.0),
        None,
    )?;
    let _ = meas.map(&1.into())?;
    let release = meas.invoke(&data)?.collect()?;
    assert_eq!(release.width(), 2);
    assert!(release.column("combined").is_ok());
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_nested_postprocessing_follows_join_fills() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(2).with_max_groups(1))?;
    let data = df!("A" => [0i32, 0], "B" => [2.0f64, 3.0])?.lazy();
    let keys = df!(
        "A" => [0i32, 1],
        "label" => ["present", "absent"],
    )?
    .lazy();

    let sum = col("B").dp().sum((lit(0.0), lit(10.0)), Some(1.0));
    let combined = (sum + lit(1.0)).cast(DataType::Float64).alias("combined");
    let query = data
        .clone()
        .group_by([col("A")])
        .agg([combined])
        .join(keys, [col("A")], [col("A")], JoinType::Right.into());
    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        query,
        Some(1.0),
        None,
    )?;
    let output = meas.invoke(&data)?.collect()?;
    assert_eq!(output.height(), 2);
    assert_eq!(output.width(), 3);
    for name in ["A", "label", "combined"] {
        assert!(output.column(name).is_ok());
    }
    Ok(())
}

#[test]
fn test_postprocess_predicate_decomposition_is_deferred() -> Fallible<()> {
    let sum = col("B").dp().sum((lit(0.0), lit(10.0)), None);
    let expression = (sum + lit(1.0)).alias("combined");
    let plan = workload::PrivateAggPlan::new(
        vec![expression],
        &[col("A")],
        &Some(KeySanitizer::Filter(col("combined").gt(lit(3.0)))),
    )?;
    assert_eq!(plan.aggs.len(), 1, "decomposition exposes the noisy leaf");
    Ok(())
}


#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_joint_dedup_shares_implicit_sum_across_outputs() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(
        Margin::by(["A"])
            .with_invariant_keys()
            .with_max_length(2)
            .with_max_groups(1),
    )?;
    let data = df!("A" => [0i32, 0], "B" => [2.0f64, 3.0])?.lazy();

    let sum = col("B").dp().sum((lit(0.0), lit(10.0)), None);
    let aggs = vec![
        sum.clone().alias("total"),
        (sum.clone() + sum.clone()).alias("double"),
        (sum + lit(1.0)).alias("plus_one"),
    ];
    let plan = workload::PrivateAggPlan::new(aggs.clone(), &[col("A")], &None)?;
    assert_eq!(plan.aggs.len(), 1, "one noise draw for all identical leaves");

    let shared = make_private_lazyframe(
        domain.clone(),
        SymmetricDistance,
        MaxDivergence,
        data.clone().group_by([col("A")]).agg(aggs),
        Some(1.0),
        None,
    )?;
    let single = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        data.clone()
            .group_by([col("A")])
            .agg([col("B").dp().sum((lit(0.0), lit(10.0)), None).alias("total")]),
        Some(1.0),
        None,
    )?;
    assert_eq!(shared.map(&1.into())?, single.map(&1.into())?);

    let released = shared.invoke(&data)?.collect()?;
    let total = released.column("total")?.f64()?.get(0).unwrap();
    let double = released.column("double")?.f64()?.get(0).unwrap();
    let plus_one = released.column("plus_one")?.f64()?.get(0).unwrap();
    assert_eq!(double, 2.0 * total);
    assert_eq!(plus_one, total + 1.0);
    assert_eq!(released.width(), 4);
    Ok(())
}

#[test]
fn test_joint_dedup_respects_scales_bounds_and_sources() -> Fallible<()> {
    let input = col("B");
    let base = input.clone().dp().sum((lit(0.0), lit(10.0)), None);
    let other_bounds = input.clone().dp().sum((lit(0.0), lit(20.0)), None);
    let other_source = col("C").dp().sum((lit(0.0), lit(10.0)), None);
    let explicit = input.clone().dp().sum((lit(0.0), lit(10.0)), Some(1.0));

    let plan = workload::PrivateAggPlan::new(
        vec![
            base.alias("a"),
            other_bounds.alias("b"),
            other_source.alias("c"),
            (explicit.clone() + explicit).alias("d"),
        ],
        &[col("A")],
        &None,
    )?;
    assert_eq!(
        plan.aggs.len(),
        5,
        "different bounds, sources, and explicit-scale draws stay separate"
    );

    let count = crate::polars::dp_len(None, false);
    let signed = crate::polars::dp_len(None, true);
    let count_plan = workload::PrivateAggPlan::new(
        vec![count.alias("unsigned"), signed.alias("signed")],
        &[col("A")],
        &None,
    )?;
    assert_eq!(count_plan.aggs.len(), 2);
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_joint_dedup_private_count_threshold_alias() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(2))?;
    let data = df!("A" => [0i32, 1])?.lazy();
    let count = crate::polars::dp_len(None, false);
    let aggs = vec![count.clone().alias("count_a"), count.alias("count_b")];
    let filter = col("count_b").gt(lit(3u32));
    let plan = workload::PrivateAggPlan::new(
        aggs.clone(),
        &[col("A")],
        &Some(KeySanitizer::Filter(filter.clone())),
    )?;
    assert_eq!(plan.aggs.len(), 1);
    assert_eq!(plan.raw_count("count_b"), "count_a");

    let measurement = make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        data.clone().group_by([col("A")]).agg(aggs).filter(filter),
        Some(1.0),
        None,
    )?;
    let _ = measurement.map(&1.into())?;
    let release = measurement.invoke(&data)?.collect()?;
    assert_eq!(release.width(), 3);
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_joint_dedup_join_fills_one_noisy_sum() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(1))?;
    let data = df!("A" => [0i32], "B" => [2.0f64])?.lazy();
    let keys = df!("A" => [0i32, 1], "label" => ["present", "absent"])?.lazy();
    let sum = col("B").dp().sum((lit(0.0), lit(10.0)), None);
    let aggs = vec![sum.clone().alias("sum_a"), sum.alias("sum_b")];
    let plan = workload::PrivateAggPlan::new(
        aggs.clone(),
        &[col("A")],
        &None,
    )?;
    assert_eq!(plan.aggs.len(), 1);
    let query = data
        .clone()
        .group_by([col("A")])
        .agg(aggs)
        .join(keys, [col("A")], [col("A")], JoinType::Right.into());
    let measurement = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        query,
        Some(1.0),
        None,
    )?;
    let release = measurement.invoke(&data)?.collect()?;
    assert_eq!(release.height(), 2);
    assert_eq!(release.width(), 4);
    let a = release.column("sum_a")?.f64()?;
    let b = release.column("sum_b")?.f64()?;
    for i in 0..release.height() {
        assert_eq!(a.get(i), b.get(i), "missing groups reuse the same noise");
    }
    Ok(())
}


#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_filter_reconstructed_mean_after_noisy_groupby() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(
        Margin::by(["A"])
            .with_invariant_keys()
            .with_max_length(2)
            .with_max_groups(1),
    )?;
    let data = df!("A" => [0i32, 0], "B" => [2.0f64, 3.0])?.lazy();
    let grouped = data.clone().group_by([col("A")]).agg([
        col("B")
            .dp()
            .mean((lit(0.0), lit(10.0)), None)
            .alias("mean_b"),
    ]);
    let base = make_private_lazyframe(
        domain.clone(),
        SymmetricDistance,
        MaxDivergence,
        grouped.clone(),
        Some(1.0),
        None,
    )?;
    let filtered = make_private_lazyframe(
        domain,
        SymmetricDistance,
        MaxDivergence,
        grouped.filter(col("mean_b").gt(lit(2.0))),
        Some(1.0),
        None,
    )?;
    assert_eq!(base.map(&1.into())?, filtered.map(&1.into())?);

    let released = filtered.invoke(&data)?;
    let DslPlan::Select { input, .. } = &released.logical_plan else {
        panic!("expected final projection");
    };
    let DslPlan::HStack { input, .. } = input.as_ref() else {
        panic!("expected reconstruction after filter");
    };
    let DslPlan::Filter { input, predicate } = input.as_ref() else {
        panic!("expected filter over noisy intermediate columns");
    };
    assert!(matches!(predicate, Expr::Column(_)));
    let DslPlan::HStack { input, exprs, .. } = input.as_ref() else {
        panic!("expected materialized selection predicate");
    };
    assert_eq!(exprs.len(), 1);
    assert!(matches!(input.as_ref(), DslPlan::GroupBy { .. }));

    let result = released.collect()?;
    assert_eq!(result.width(), 2, "selection and noisy intermediates stay private");
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_private_filter_with_threshold_and_reconstructed_mean() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(2))?;
    let data = df!("A" => [0i32, 1], "B" => [2.0f64, 3.0])?.lazy();
    let query = data
        .clone()
        .group_by([col("A")])
        .agg([
            crate::polars::dp_len(None, false).alias("count"),
            col("B").dp().mean((lit(0.0), lit(10.0)), None).alias("mean_b"),
        ])
        .filter(
            col("count")
                .gt(lit(3u32))
                .and(col("mean_b").gt(lit(1.0))),
        );

    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        query,
        Some(1.0),
        None,
    )?;
    let _ = meas.map(&1.into())?;
    let plan = meas.invoke(&data)?.logical_plan;
    let DslPlan::Select { input, .. } = &plan else {
        panic!("expected final projection");
    };
    let DslPlan::HStack { input, .. } = input.as_ref() else {
        panic!("expected final reconstruction");
    };
    let DslPlan::Filter { input, .. } = input.as_ref() else {
        panic!("expected key sanitizer");
    };
    let DslPlan::HStack { exprs, .. } = input.as_ref() else {
        panic!("expected selection predicate over raw noisy aggregates");
    };
    assert_eq!(exprs.len(), 1);
    assert!(matches!(
        exprs[0].clone().meta().undo_aliases(),
        Expr::BinaryExpr { op: Operator::And | Operator::LogicalAnd, .. }
    ));
    let release = LazyFrame::from(plan).collect()?;
    assert_eq!(release.width(), 3);
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_constructor_threshold_preserves_additional_filter() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(2))?;
    let data = df!("A" => [0i32, 1], "B" => [2.0f64, 3.0])?.lazy();

    // The threshold is supplied by the constructor, but the user still wants
    // to filter the released mean. Both conditions must be applied.
    let query = data
        .clone()
        .group_by([col("A")])
        .agg([
            col("B").dp().mean((lit(0.0), lit(10.0)), None).alias("mean_b"),
        ])
        .filter(col("mean_b").gt(lit(1.5)));
    let meas = make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        query,
        Some(1.0),
        Some(3),
    )?;
    let _ = meas.map(&1.into())?;
    let plan = meas.invoke(&data)?.logical_plan;
    let DslPlan::Select { input, .. } = &plan else {
        panic!("expected final projection");
    };
    let DslPlan::HStack { input, .. } = input.as_ref() else {
        panic!("expected reconstructed mean");
    };
    let DslPlan::Filter { input, .. } = input.as_ref() else {
        panic!("expected key sanitizer");
    };
    let DslPlan::HStack { exprs, .. } = input.as_ref() else {
        panic!("expected materialized predicate");
    };
    assert!(matches!(
        exprs[0].clone().meta().undo_aliases(),
        Expr::BinaryExpr { op: Operator::And | Operator::LogicalAnd, .. }
    ));
    let output = LazyFrame::from(plan).collect()?;
    assert_eq!(output.width(), 2);
    Ok(())
}

#[test]
#[cfg(not(feature = "polars-ffi"))]
fn test_disjunction_cannot_certify_private_keys() -> Fallible<()> {
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(2))?;
    let data = df!("A" => [0i32], "B" => [2.0f64])?.lazy();
    let query = data.group_by([col("A")])
        .agg([
            crate::polars::dp_len(None, false).alias("count"),
            col("B").dp().mean((lit(0.0), lit(10.0)), None).alias("mean_b"),
        ])
        .filter(
            col("count").gt(lit(3u32))
                .or(col("mean_b").gt(lit(1.0))),
        );
    let err = make_private_lazyframe(
        domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        query,
        Some(1.0),
        None,
    ).map(|_| ()).unwrap_err();
    assert_eq!(err.variant, MakeMeasurement);
    Ok(())
}

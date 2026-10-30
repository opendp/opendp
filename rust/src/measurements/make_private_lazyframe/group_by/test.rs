use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::error::ErrorVariant::MakeMeasurement;
use crate::error::*;
use crate::measurements::{make_private_expr, make_private_lazyframe};
use crate::measures::MaxDivergence;
use crate::metrics::{L0PInfDistance, SymmetricDistance};
use crate::polars::PrivacyNamespace;
use crate::traits::samplers::test::{check_chi_square, check_kolmogorov_smirnov};
use polars::prelude::*;

use super::*;

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
    let y = df!("A" => ["a", "a", "a"], "B" => [-10i32, -10, -10])?.lazy();
    assert!(domain.member(&x)?);
    assert!(domain.member(&y)?);

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

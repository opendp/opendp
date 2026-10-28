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
    .with_margin(Margin::by(["A"]).with_max_length(1))?;

    let lf = df!("A" => &[0u32], "B" => &[0.0f64])?.lazy();
    let keys = df!("A" => &(0u32..N_SAMPLES).collect::<Vec<_>>())?.lazy();

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


// ADD A HIGH DELTA TEST FOR MAKE PRIVATE GROUPBY.
// OR CALL WITH A THRESHOLD OF 0.
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

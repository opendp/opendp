use super::*;
use crate::traits::CastInternalRational;

#[test]
fn exp_half_coin_parity_and_denominators() -> Fallible<()> {
    for successes in 0usize..10 {
        let mut calls = 0;
        let out = sample_bernoulli_exp_half_with(|denominator| {
            calls += 1;
            assert_eq!(*denominator, UBig::from(2 * calls));
            Ok(calls <= successes)
        })?;
        assert_eq!(out, successes % 2 == 0);
        assert_eq!(calls, successes + 1);
    }
    Ok(())
}

#[test]
fn half_gaussian_coin_paths_and_restarts() -> Fallible<()> {
    let run = |script: &[bool], expected: u8| -> Fallible<()> {
        let mut coins = script.iter().copied();
        let k = sample_standard_half_gaussian_with(|| {
            Ok(coins.next().expect("unexpected extra coin"))
        })?;
        assert_eq!(k, UBig::from(expected));
        assert!(coins.next().is_none(), "returned before consuming the path");
        Ok(())
    };
    for k in 0..=4 {
        let mut script = vec![true; k * k];
        script.push(false);
        run(&script, k as u8)?;
    }
    // A failure in any extra-success block must restart at zero, not return k.
    for k in 1..=4 {
        for failure_position in 0..2 * k {
            let mut script = vec![true; k * k + 1 + failure_position];
            script.extend([false, true, false]); // rejection, then return k=1
            run(&script, 1)?;
        }
    }
    Ok(())
}

#[test]
fn integer_sampler_propagates_entropy_errors() {
    assert!(
        sample_bernoulli_exp_half_with(|_| {
            fallible!(FailedFunction, "synthetic entropy failure")
        })
        .is_err()
    );
    let mut calls = 0;
    assert!(
        sample_standard_half_gaussian_with(|| {
            calls += 1;
            if calls == 3 {
                // fail inside the first extra-success block
                fallible!(FailedFunction, "synthetic entropy failure")
            } else {
                Ok(true)
            }
        })
        .is_err()
    );
}

#[test]
fn specialized_integer_sampler_distribution() -> Fallible<()> {
    let n = 50_000;
    let mut coin_successes = 0;
    let mut bins = [0usize; 5];
    for _ in 0..n {
        coin_successes += usize::from(sample_bernoulli_exp_half()?);
        let k = sample_standard_half_gaussian_with(sample_bernoulli_exp_half)?;
        let bin = if k >= UBig::from(4u8) {
            4
        } else {
            usize::try_from(k).unwrap()
        };
        bins[bin] += 1;
    }
    let coin_mean = coin_successes as f64 / n as f64;
    assert!(
        (coin_mean - (-0.5f64).exp()).abs() < 0.02,
        "coin mean={coin_mean}"
    );
    let weight = |k: usize| (-0.5 * (k * k) as f64).exp();
    let normalizer: f64 = (0..20).map(weight).sum();
    for (k, count) in bins.into_iter().enumerate() {
        let expected = if k < 4 {
            weight(k) / normalizer
        } else {
            (4..20).map(weight).sum::<f64>() / normalizer
        };
        let observed = count as f64 / n as f64;
        assert!(
            (observed - expected).abs() < 0.02,
            "k={k}, observed={observed}, expected={expected}"
        );
    }
    Ok(())
}

#[test]
fn reused_interval_matches_affine_certificate() -> Fallible<()> {
    for mu in [f64::MIN, -1.0, 0.0, f64::from_bits(1), f64::MAX] {
        for scale in [f64::from_bits(1), 0.5, 1.0, f64::MAX] {
            let mu = mu.into_rational()?;
            let scale = scale.into_rational()?;
            for negative in [false, true] {
                let mut z = ExactStdNormal {
                    negative,
                    k: RBig::ONE,
                    x: PartialUniform01::new(Uniform01RV),
                };
                for _ in 0..3 {
                    let interval = normal_interval(&z, &mu, &scale);
                    assert_eq!(
                        interval.0,
                        normal_endpoint(&z, &mu, &scale, Endpoint::Lower)
                    );
                    assert_eq!(
                        interval.1,
                        normal_endpoint(&z, &mu, &scale, Endpoint::Upper)
                    );
                    let (_, cell) = candidate_and_cell_from_interval::<f64>(&interval)?;
                    assert_eq!(
                        interval_inside_cell(&interval, &cell),
                        affine_trace_inside_cell(&z, &mu, &scale, None, &cell)
                    );
                    z.x.refine()?;
                }
            }
        }
    }
    Ok(())
}

#[test]
fn gaussian_distribution_smoke() -> Fallible<()> {
    let n = 10_000;
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut tail = 0;
    for _ in 0..n {
        let z = (sample_rounded_gaussian(1.0f64, 2.0)? - 1.0) / 2.0;
        sum += z;
        sum_sq += z * z;
        tail += usize::from(z.abs() > 1.0);
    }
    let mean = sum / n as f64;
    let second_moment = sum_sq / n as f64;
    let tail = tail as f64 / n as f64;
    assert!(mean.abs() < 0.1, "mean={mean}");
    assert!(
        (second_moment - 1.0).abs() < 0.12,
        "second_moment={second_moment}"
    );
    assert!((tail - 0.3173105078629141).abs() < 0.04, "tail={tail}");
    Ok(())
}

#[test]
fn native_f64_to_f32_rounded_gaussian_smoke() -> Fallible<()> {
    assert_eq!(
        sample_rounded_gaussian_f64_to_f32_native(1.25, 0.0)?,
        1.25_f32
    );
    assert!(sample_rounded_gaussian_f64_to_f32_native(0.0, 1.0).is_err());

    for _ in 0..1_000 {
        assert!(
            sample_rounded_gaussian_f64_to_f32_native_clipped(0.0, 1.0, Some(1.0))?.is_finite()
        );
    }
    Ok(())
}

#[test]
fn native_f64_to_f32_clipped_rounded_gaussian_clips_input_and_output() -> Fallible<()> {
    assert_eq!(
        sample_rounded_gaussian_f64_to_f32_native_clipped(2.0, 0.0, Some(1.0))?,
        1.0_f32
    );
    assert!(sample_rounded_gaussian_f64_to_f32_native_clipped(0.0, 1.0, Some(524_288.0)).is_err());

    for _ in 0..100 {
        let out = sample_rounded_gaussian_f64_to_f32_native_clipped(0.0, 100.0, Some(1.0))?;
        assert!((-1.0..=1.0).contains(&out));
    }

    for _ in 0..10 {
        let out = sample_rounded_gaussian_f64_to_f32_native_clipped(f64::MAX, 1.0, Some(1.0))?;
        assert!((-1.0..=1.0).contains(&out));
    }

    assert_eq!(
        sample_rounded_gaussian_f64_to_f32_native_clipped(
            1.0,
            f64::from(f32::from_bits(1)) * 0.5,
            Some(0.0),
        )?,
        0.0
    );

    Ok(())
}

#[test]
fn exact_clipped_rounded_gaussian_clips_input_and_output() -> Fallible<()> {
    assert_eq!(sample_rounded_gaussian_clipped(2.0_f64, 0.0, 1.0)?, 1.0);

    for _ in 0..100 {
        let out = sample_rounded_gaussian_clipped(0.0_f32, 100.0, 1.0)?;
        assert!((-1.0..=1.0).contains(&out));
    }

    Ok(())
}

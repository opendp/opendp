//! Verify the defining law of the bounded sampler: discrete Laplace noise
//! followed by clamping to the requested output interval.

use crate::traits::samplers::test::assert_close_binomial_mean;

use super::*;

const N: usize = 6_000;
const SCALE: f64 = 2.0;

/// For clamped input x in [L,U] and a=exp(-1/scale), the output probabilities
/// are a^(x-L)/(1+a) at L, a^(U-x)/(1+a) at U, and
/// (1-a)*a^abs(y-x)/(1+a) at every interior y.
/// Input and output are offsets from L; width is U-L.
fn clamped_discrete_laplace_pmf(output: usize, input: usize, width: usize, scale: f64) -> f64 {
    let a = (-1.0 / scale).exp();
    match output {
        0 => a.powf(input as f64) / (1.0 + a),
        y if y == width => a.powf((width - input) as f64) / (1.0 + a),
        y => (1.0 - a) / (1.0 + a) * a.powf(y.abs_diff(input) as f64),
    }
}

/// Compare every output atom against the independently specified PMF.
fn check_pmf<T: Integer>(shift: T, (lower, upper): (T, T)) -> Fallible<()>
where
    usize: ExactIntCast<T>,
{
    let width = usize::exact_int_cast(upper.alerting_sub(&lower)?)?;
    assert!(width > 0);
    let x = shift.total_clamp(lower, upper)?;
    let input = usize::exact_int_cast(x.alerting_sub(&lower)?)?;
    let probs: Vec<_> = (0..=width)
        .map(|output| clamped_discrete_laplace_pmf(output, input, width, SCALE))
        .collect();
    assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-12);

    let mut observed = vec![0usize; width + 1];
    for _ in 0..N {
        let sample = sample_discrete_laplace_linear::<T, f64>(shift, SCALE, (lower, upper))?;
        let offset = usize::exact_int_cast(sample.alerting_sub(&lower)?)?;
        observed[offset] += 1;
    }

    let statistic: f64 = observed
        .iter()
        .zip(probs.iter())
        .map(|(&count, &p)| {
            let expected = p * N as f64;
            assert!(expected >= 5.0, "expected count too small: {expected}");
            (count as f64 - expected).powi(2) / expected
        })
        .sum();
    // df = width. Python: from scipy.stats import chi2; chi2.isf(1e-8, [1, 10])
    let critical = match width {
        1 => 32.84125336123679,
        10 => 57.66396162619969,
        _ => unreachable!("missing chi-square critical value for df={width}"),
    };
    assert!(
        statistic <= critical,
        "clamped geometric PMF mismatch for {}: shift={shift}, bounds=({lower}, {upper}), statistic={statistic}, critical={critical}, observed={observed:?}, probs={probs:?}",
        std::any::type_name::<T>()
    );
    Ok(())
}

/// Cover integer-minimum saturation (the regression), maximum saturation,
/// and a translation away from the integer minimum.
fn check_extremes<T: Integer>() -> Fallible<()>
where
    usize: ExactIntCast<T>,
{
    let ten = T::exact_int_cast(10_i32)?;
    let min = T::MIN_FINITE;
    let max = T::MAX_FINITE;
    check_pmf(min, (min, min + ten))?;
    check_pmf(min + T::one(), (min + T::one(), min + T::one() + ten))?;
    check_pmf(max, (max - ten, max))?;
    Ok(())
}

#[test]
fn discrete_laplace_linear_matches_clamped_pmf_at_type_extremes() -> Fallible<()> {
    check_extremes::<u8>()?;
    check_extremes::<i8>()?;
    check_extremes::<u32>()?;
    check_extremes::<i32>()
}

#[test]
fn discrete_laplace_linear_matches_clamped_pmf_at_interior_and_outside_bounds() -> Fallible<()> {
    // Both choices of signed zero, interior inputs, and input clamping.
    for shift in [i8::MIN, -5, -4, 0, 4, 5, i8::MAX] {
        check_pmf(shift, (-5i8, 5))?;
    }
    // A one-step interval sets trials=Some(0).
    for shift in [0u8, 1] {
        check_pmf(shift, (0u8, 1))?;
    }
    Ok(())
}

#[test]
fn discrete_laplace_linear_zero_count_privacy_regression() -> Fallible<()> {
    // Issue #3012: previously P(output=0 | input=0) was near zero,
    // instead of the correct value ~1/2.
    const REGRESSION_N: usize = 3_000;
    let scale = 1000.0;
    for shift in [0u32, 1] {
        let expected = clamped_discrete_laplace_pmf(0, shift as usize, 200, scale);
        let mut hits = 0usize;
        for _ in 0..REGRESSION_N {
            if sample_discrete_laplace_linear::<u32, f64>(shift, scale, (0, 200))? == 0 {
                hits += 1;
            }
        }
        assert_close_binomial_mean(
            hits as f64 / REGRESSION_N as f64,
            expected,
            REGRESSION_N,
            &format!("issue #3012, input={shift}"),
        );
    }
    Ok(())
}

#[test]
fn discrete_laplace_linear_zero_scale_clamps_input() -> Fallible<()> {
    // Zero noise still applies the input/output bounds.
    for (shift, expected) in [(0u8, 3), (4, 4), (10, 7)] {
        assert_eq!(
            sample_discrete_laplace_linear::<u8, f64>(shift, 0.0, (3, 7))?,
            expected
        );
    }
    for (shift, expected) in [(-10i8, -3), (0, 0), (10, 3)] {
        assert_eq!(
            sample_discrete_laplace_linear::<i8, f64>(shift, 0.0, (-3, 3))?,
            expected
        );
    }
    Ok(())
}

#[test]
fn discrete_laplace_linear_extreme_bounds_in_range() -> Fallible<()> {
    // Wide intervals and out-of-range inputs also exercise integer arithmetic.
    for bounds in [(i8::MIN, -1), (0, i8::MAX), (-127, 0)] {
        for shift in [i8::MIN, bounds.0, bounds.1, i8::MAX] {
            for _ in 0..50 {
                let v = sample_discrete_laplace_linear::<i8, f64>(shift, 50.0, bounds)?;
                assert!((bounds.0..=bounds.1).contains(&v));
            }
        }
    }
    Ok(())
}

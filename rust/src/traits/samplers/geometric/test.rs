//! Invariants tested
//! - the output distribution is invariant to translating `shift` and `bounds` together, including onto the type minimum and maximum
//! - outputs stay within `bounds` without error when `shift` and `bounds` sit at the type extremes

use crate::traits::samplers::test::TEST_Z;

use super::*;

const WIDTH: i8 = 10;
// the regression shifts mass at `lower` from ~0.62 to ~0.39, so far fewer samples than BASE_N suffice
const N: usize = 10_000;

/// Histogram of offsets from `lower` when noising `shift = lower + offset` on `[lower, lower + WIDTH]`.
fn histogram<T: Integer>(lower: T, offset: i8) -> Fallible<Vec<u64>>
where
    usize: ExactIntCast<T>,
{
    let cast = |v: i8| T::exact_int_cast(v as i32);
    let (shift, upper) = (lower + cast(offset)?, lower + cast(WIDTH)?);
    let mut counts = vec![0u64; WIDTH as usize + 1];
    for _ in 0..N {
        let v = sample_discrete_laplace_linear::<T, f64>(shift, 2.0, (lower, upper))?;
        counts[usize::exact_int_cast(v - lower)?] += 1;
    }
    Ok(counts)
}

/// Two-sample chi-square test of homogeneity.
fn assert_same_distribution(a: Vec<u64>, b: Vec<u64>) -> Fallible<()> {
    let (mut statistic, mut df) = (0.0f64, -1.0f64);
    for (&a, &b) in a.iter().zip(&b) {
        if a + b > 0 {
            statistic += (a as f64 - b as f64).powi(2) / (a + b) as f64;
            df += 1.0;
        }
    }
    let crit = df + TEST_Z * (2.0 * df).sqrt();
    if statistic > crit {
        return fallible!(
            FailedFunction,
            "distributions differ: statistic={statistic} > crit={crit}\n{a:?}\n{b:?}"
        );
    }
    Ok(())
}

#[test]
fn test_discrete_laplace_linear_type_extremes_match_interior() -> Fallible<()> {
    // regression: a shift at the type minimum previously lost the mass of negative noise
    assert_same_distribution(histogram(0u8, 0)?, histogram(1u8, 0)?)?;
    assert_same_distribution(histogram(i8::MIN, 0)?, histogram(i8::MIN + 1, 0)?)?;
    // the type maximum is the mirror case
    let lower = i8::MAX - WIDTH;
    assert_same_distribution(histogram(lower, WIDTH)?, histogram(lower - 1, WIDTH)?)?;
    // shift at `lower` rejects positive zeros, shift at `upper` rejects negative zeros; they must mirror
    let mut mirrored = histogram(0i8, WIDTH)?;
    mirrored.reverse();
    assert_same_distribution(histogram(0i8, 0)?, mirrored)
}

#[test]
fn test_discrete_laplace_linear_extreme_bounds_infallible() -> Fallible<()> {
    // includes bounds wider than `i8::MAX`, whose width overflows `i8`
    for bounds in [
        (i8::MIN, -1),
        (0, i8::MAX),
        (-127, 0),
        (-100, 100),
        (i8::MIN, i8::MAX),
    ] {
        for shift in [i8::MIN, bounds.0, bounds.1, i8::MAX] {
            for _ in 0..1000 {
                let v = sample_discrete_laplace_linear::<i8, f64>(shift, 50.0, bounds)?;
                assert!(bounds.0 <= v && v <= bounds.1);
            }
        }
    }
    Ok(())
}

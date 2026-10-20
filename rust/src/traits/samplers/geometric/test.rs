//! Invariants tested
//! - the output distribution is invariant to translating `shift` and `bounds` together, including onto the type minimum
//! - a shift at `lower` mirrors a shift at `upper`
//! - outputs stay within `bounds` when `shift` and `bounds` sit at the type extremes

use crate::traits::samplers::test::TEST_Z;

use super::*;

const WIDTH: i8 = 10;
const N: usize = 10_000;

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
fn check_chi_square_homogeneity(a: &[u64], b: &[u64]) -> Fallible<()> {
    let (mut statistic, mut df) = (0.0f64, -1.0f64);
    for (&a, &b) in a.iter().zip(b) {
        if a + b > 0 {
            statistic += (a as f64 - b as f64).powi(2) / (a + b) as f64;
            df += 1.0;
        }
    }
    let crit = df + TEST_Z * (2.0 * df).sqrt();
    if statistic > crit {
        return fallible!(
            FailedFunction,
            "Chi-square rejected: statistic={statistic} > crit={crit}\n{a:?}\n{b:?}"
        );
    }
    Ok(())
}

#[test]
fn discrete_laplace_linear_type_min_matches_interior() -> Fallible<()> {
    check_chi_square_homogeneity(&histogram(i8::MIN, 0)?, &histogram(i8::MIN + 1, 0)?)
}

#[test]
fn discrete_laplace_linear_shift_at_bounds_is_mirrored() -> Fallible<()> {
    let mut mirrored = histogram(0i8, WIDTH)?;
    mirrored.reverse();
    check_chi_square_homogeneity(&histogram(0i8, 0)?, &mirrored)
}

#[test]
fn discrete_laplace_linear_extreme_bounds_in_range() -> Fallible<()> {
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

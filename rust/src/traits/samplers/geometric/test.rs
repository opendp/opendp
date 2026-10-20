//! Invariants tested
//! - the output distribution is invariant to translating `shift` and `bounds` together, including onto the type minimum

use crate::traits::samplers::test::{BASE_N, TEST_Z};

use super::*;

const WIDTH: u8 = 10;

/// Histogram of offsets from `lower` when noising `shift = lower`.
fn histogram(lower: u8) -> Fallible<Vec<u64>> {
    let mut counts = vec![0u64; WIDTH as usize + 1];
    for _ in 0..BASE_N {
        let v = sample_discrete_laplace_linear::<u8, f64>(lower, 2.0, (lower, lower + WIDTH))?;
        counts[(v - lower) as usize] += 1;
    }
    Ok(counts)
}

#[test]
fn test_discrete_laplace_linear_zero_matches_one() -> Fallible<()> {
    // regression: a shift at the type minimum previously lost the mass of negative noise
    let (a, b) = (histogram(0)?, histogram(1)?);

    // two-sample chi-square test of homogeneity
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

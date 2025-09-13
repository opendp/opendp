use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Div, Sub};

use num::traits::real::Real;
use statrs::function::erf;

use num::{NumCast, One};

use crate::error::Fallible;

/// returns z-statistic that satisfies p == ∫P(x)dx over (-∞, z),
///     where P is the standard normal distribution
pub fn normal_cdf_inverse(p: f64) -> f64 {
    std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * p)
}

macro_rules! c {
    ($expr:expr; $ty:ty) => {{
        let t: $ty = NumCast::from($expr).unwrap();
        t
    }};
}

pub fn test_proportion_parameters<T, FS: Fn() -> T>(
    sampler: FS,
    p_pop: T,
    alpha: f64,
    err_margin: T,
) -> bool
where
    T: Sum<T> + Sub<Output = T> + Div<Output = T> + Real + Debug + One,
{
    // |z_{alpha/2}|
    let z_stat = c!(normal_cdf_inverse(alpha / 2.).abs(); T);

    // derived sample size necessary to conduct the test
    let n: T = (p_pop * (T::one() - p_pop) * (z_stat / err_margin).powi(2)).ceil();

    // confidence interval for the mean
    let abs_p_tol = z_stat * (p_pop * (T::one() - p_pop) / n).sqrt(); // almost the same as err_margin

    println!(
        "sampling {:?} observations to detect a change in proportion with {:.4?}% confidence",
        c!(n; u32),
        (1. - alpha) * 100.
    );

    // take n samples from the distribution, compute average as empirical proportion
    let p_emp: T = (0..c!(n; u32)).map(|_| sampler()).sum::<T>() / n;

    let passed = (p_emp - p_pop).abs() < abs_p_tol;

    println!("stat: (tolerance, pop, emp, passed)");
    println!(
        "    proportion:     {:?}, {:?}, {:?}, {:?}",
        abs_p_tol, p_pop, p_emp, passed
    );
    println!();

    passed
}

#[allow(dead_code)]
/// Conduct a Kolmogorov-Smirnov (KS) test.
///
/// Since the critical values are difficult to compute in Rust,
/// this function hardcodes the critical value corresponding to a p-value of 1e-6 when 1000 samples are taken.
///
/// Assuming the samples are draws from the distribution specified by the cdf,
/// then the p-value is the false discovery rate,
/// or chance of this test failing even when the data is a sample from the distribution.
pub fn check_kolmogorov_smirnov(
    mut samples: [f64; 1000],
    cdf: impl Fn(f64) -> f64,
) -> Fallible<()> {
    // first, compute the test statistic. For a one-sample KS test,
    // this is the greatest distance between the empirical CDF of the samples and the expected CDF.
    samples.sort_by(|a, b| a.total_cmp(b));

    let n = samples.len() as f64;
    let statistic = samples
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let empirical_cdf = i as f64 / n;
            let idealized_cdf = cdf(s);
            (empirical_cdf - idealized_cdf).abs()
        })
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();

    // The KS-test is nonparametric,
    // so the critical value only changes in response to the number of samples (hardcoded at 1000),
    // not the distribution.
    //
    // The p-value corresponds to the mass of the tail of the KS distribution beyond the critical value.
    // The mass of the tail is the complement of the cumulative distribution function,
    // which is also called the survival function.
    // The inverse of the survival function `isf` tells us the critical value corresponding to a given mass of the tail.
    //
    // We therefore derive the critical value via the inverse survival function of the two-sided, one-sample KS distribution:
    // ```python
    // from scipy.stats import kstwo
    // CRIT_VALUE = kstwo(n=1000).isf(1e-6)
    // ```
    static CRIT_VALUE: f64 = 0.08494641956324511;
    if statistic > CRIT_VALUE {
        return fallible!(
            FailedFunction,
            "Statistic ({statistic}) exceeds critical value ({CRIT_VALUE})! This indicates that the data is not sampled from the same distribution specified by the cdf. There is a 1e-6 probability of this being a false positive."
        );
    }

    Ok(())
}

#[allow(dead_code)]
/// Conduct a chi-squared test.
///
/// Since the critical values are difficult to compute in Rust,
/// this function hardcodes the critical value corresponding to a p-value of 1e-6 for 9 degrees of freedom.
///
/// Assuming the samples are draws from the expected distribution,
/// then the p-value is the false discovery rate,
/// or chance of this test failing even when the data is a sample from the distribution.
pub fn check_chi_square(observed: [f64; 10], expected: [f64; 10]) -> Fallible<()> {
    let statistic = observed
        .iter()
        .zip(expected)
        .map(|(o, e)| (o - e).powi(2) / e)
        .sum::<f64>();

    // from scipy.stats import chi2
    // CRIT_VALUE = chi2(df=9).isf(1e-6)
    static CRIT_VALUE: f64 = 44.81093787068782;
    if statistic > CRIT_VALUE {
        return fallible!(
            FailedFunction,
            "Statistic ({statistic}) exceeds critical value ({CRIT_VALUE})! This indicates that the data is not sampled from the same distribution specified by the cdf. There is a 1e-6 probability of this being a false positive."
        );
    }
    Ok(())
}

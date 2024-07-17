use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Div, Sub};

use num::traits::real::Real;
use statrs::function::erf;

use num::{NumCast, One};

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

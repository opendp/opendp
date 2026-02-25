use statrs::function::erf;

use crate::error::Fallible;

pub const TEST_Z: f64 = 6.0;
pub const FP_SLOP: f64 = 5e-12;
pub const ALPHA: f64 = 1e-6;
pub const BASE_N: usize = 60_000;

pub fn sample_mean_bool(mut f: impl FnMut() -> bool, n: usize) -> f64 {
    (0..n).filter(|_| f()).count() as f64 / n as f64
}

// Φ^{-1}(p) = -sqrt(2)*erfc^{-1}(2p)
pub fn normal_cdf_inverse(p: f64) -> f64 {
    -std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * p)
}

// Wilson (two-sided) proportion acceptance test.
fn wilson_contains_p0(k: usize, n: usize, p0: f64, alpha: f64) -> bool {
    assert!((0.0..=1.0).contains(&p0));
    assert!(n > 0);
    assert!(alpha > 0.0 && alpha < 1.0);

    let z = normal_cdf_inverse(1.0 - alpha / 2.0);
    let n_f = n as f64;
    let phat = k as f64 / n_f;
    let z2 = z * z;

    let denom = 1.0 + z2 / n_f;
    let center = (phat + z2 / (2.0 * n_f)) / denom;
    let half = (z / denom) * (phat * (1.0 - phat) / n_f + z2 / (4.0 * n_f * n_f)).sqrt();

    let lo = (center - half).max(0.0);
    let hi = (center + half).min(1.0);

    lo - FP_SLOP <= p0 && p0 <= hi + FP_SLOP
}

pub fn run_wilson_test(
    mut sampler: impl FnMut() -> bool,
    p0: f64,
    n: usize,
    alpha: f64,
    label: &str,
) {
    let k = (0..n).filter(|_| sampler()).count();
    assert!(
        wilson_contains_p0(k, n, p0, alpha),
        "{}: Wilson rejected. n={}, k={}, p_hat={}, p0={}, alpha={}",
        label,
        n,
        k,
        k as f64 / n as f64,
        p0,
        alpha
    );
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
    mut samples: [f64; 5000],
    cdf: impl Fn(f64) -> f64,
) -> Fallible<()> {
    samples.sort_by(|a, b| a.total_cmp(b));
    let n = samples.len() as f64;

    let mut d_plus = 0f64;
    let mut d_minus = 0f64;

    for (idx0, &x) in samples.iter().enumerate() {
        let i = (idx0 + 1) as f64; // 1..=n
        let f = cdf(x).clamp(0.0, 1.0);

        // D+ = max_i (i/n - F(x_i))
        d_plus = d_plus.max(i / n - f);

        // D- = max_i (F(x_i) - (i-1)/n)
        d_minus = d_minus.max(f - (i - 1.0) / n);
    }

    let statistic = d_plus.max(d_minus);

    // Critical value must correspond to the SAME statistic definition above.
    // Derived in Python:
    //   from scipy.stats import kstwo
    //   kstwo(n=5000).isf(1e-6)
    static CRIT_VALUE: f64 = 0.038051617888080105;

    if statistic > CRIT_VALUE {
        return fallible!(
            FailedFunction,
            "KS statistic ({statistic}) exceeds critical value ({CRIT_VALUE}). \
             Under the KS assumptions (i.i.d. samples; continuous reference CDF), \
             Type I error is <= 1e-6."
        );
    }
    Ok(())
}

#[allow(dead_code)]
/// Conduct a Pearson chi-squared goodness-of-fit test.
///
/// - `observed[i]` are observed counts (integers).
/// - `expected[i]` are expected counts (floats) for the same bins.
///
/// This uses a conservative normal-approx bound for the chi-square critical value:
///     chi2 <= df + Z * sqrt(2*df)
/// where `df = #bins - 1`.
///
/// This avoids hardcoding a specific df/alpha table while keeping flakiness low.
///
/// Notes:
/// - Requires all expected counts >= 5 for the usual chi-square approximation.
/// - Assumes expected counts are fixed a priori (no data-dependent binning, unless you adjust df/alpha appropriately).
pub fn check_chi_square(observed: &[u64], expected: &[f64]) -> Fallible<()> {
    if observed.len() != expected.len() {
        return fallible!(
            FailedFunction,
            "observed/expected length mismatch: {} vs {}",
            observed.len(),
            expected.len()
        );
    }
    if observed.is_empty() {
        return fallible!(FailedFunction, "no bins");
    }

    // Preconditions: expected counts must be finite and >= 5.
    for (i, &e) in expected.iter().enumerate() {
        if !e.is_finite() || e <= 0.0 {
            return fallible!(
                FailedFunction,
                "expected[{i}] must be finite and > 0, got {e}"
            );
        }
        if e < 5.0 {
            return fallible!(
                FailedFunction,
                "expected[{i}] too small for chi-square approximation: expected={e} (<5)."
            );
        }
    }

    // df = k - 1 - params_estimated
    let k = observed.len();
    let df = (k - 1) as f64;

    // Statistic
    let mut statistic = 0.0f64;
    for (&o, &e) in observed.iter().zip(expected.iter()) {
        let o = o as f64;
        let diff = o - e;
        statistic += diff * diff / e;
    }

    // Conservative critical value (upper tail) without special functions:
    // chi2 ~ approx Normal(mean=df, var=2df)
    let crit_value = df + TEST_Z * (2.0 * df).sqrt();

    if statistic > crit_value {
        return fallible!(
            FailedFunction,
            "Chi-square rejected: statistic={statistic} > crit={crit_value} (df={df}, z={TEST_Z}). \
             Assumes iid samples and fixed expected counts."
        );
    }

    Ok(())
}

pub fn check_chi_square_from_probs(observed: &[u64], probs: &[f64]) -> Fallible<()> {
    if observed.len() != probs.len() {
        return fallible!(FailedFunction, "length mismatch");
    }
    let n: u64 = observed.iter().sum();
    let n_f = n as f64;
    let expected: Vec<f64> = probs.iter().map(|p| n_f * p).collect();
    check_chi_square(observed, &expected)
}

/// Conduct a wald z-test.
pub fn assert_close_normal(est: f64, target: f64, se: f64, what: &str) {
    let tol = TEST_Z * se + FP_SLOP;
    let err = (est - target).abs();
    assert!(
        err <= tol,
        "{}: est={} target={} err={} tol={} (se={})",
        what,
        est,
        target,
        err,
        tol,
        se
    );
}

/// Conducts a wald z-test on the mean of a binomial RV.
pub fn assert_close_binomial_mean(p_hat: f64, p: f64, n: usize, what: &str) {
    let se = (p * (1.0 - p) / (n as f64)).sqrt();
    assert_close_normal(p_hat, p, se, what);
}

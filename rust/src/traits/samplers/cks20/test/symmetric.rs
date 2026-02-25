use std::convert::TryFrom;

use dashu::integer::IBig;
use dashu::rbig;

use crate::traits::samplers::{
    sample_discrete_gaussian,
    test::{assert_close_binomial_mean, assert_close_normal},
};

use super::*;

pub const N_DEGENERATE: usize = 50_000;

#[derive(Debug)]
pub struct SignedHist {
    pub kmax: usize,
    /// counts for x in [-kmax..=kmax], followed by tail bin for |x|>kmax
    pub counts: Vec<u64>,
}

impl SignedHist {
    pub fn sample(n: usize, kmax: usize, mut sampler: impl FnMut() -> IBig) -> Self {
        let bins = 2 * kmax + 1;
        let mut counts = vec![0u64; bins + 1];
        let offset = kmax as i64;

        for _ in 0..n {
            let xi = i64::try_from(sampler()).unwrap();
            if (-offset..=offset).contains(&xi) {
                counts[(xi + offset) as usize] += 1;
            } else {
                counts[bins] += 1;
            }
        }

        Self { kmax, counts }
    }

    pub fn tail_count(&self) -> u64 {
        self.counts[2 * self.kmax + 1]
    }

    pub fn abs_counts(&self) -> Vec<u64> {
        let mut out = vec![0u64; self.kmax + 1];
        out[0] = self.counts[self.kmax];
        for k in 1..=self.kmax {
            out[k] = self.counts[self.kmax + k] + self.counts[self.kmax - k];
        }
        out
    }

    pub fn pos_neg_excl0(&self) -> (u64, u64) {
        let mut pos = 0u64;
        let mut neg = 0u64;
        for k in 1..=self.kmax {
            pos += self.counts[self.kmax + k];
            neg += self.counts[self.kmax - k];
        }
        (pos, neg)
    }

    /// Count of samples with |X| >= t, including the tail bin.
    pub fn tail_abs_ge(&self, t: usize) -> u64 {
        let abs = self.abs_counts();
        abs[t..].iter().sum::<u64>() + self.tail_count()
    }
}

/// Weighted log-ratio check:
/// compares ln(C[k+1]/C[k]) to ln_ratio_theory(k) over k_range,
/// weighting by delta-method variance ~ 1/C[k+1] + 1/C[k].
pub fn assert_log_ratio_matches(
    abs_counts: &[u64],
    k_range: std::ops::RangeInclusive<usize>,
    ln_ratio_theory: impl Fn(usize) -> f64,
    min_count: u64,
    label: &str,
    assert_close_normal: impl Fn(f64, f64, f64, &str),
) {
    let mut weighted_err_sum = 0.0;
    let mut weight_sum = 0.0;

    for k in k_range {
        let c_k = abs_counts[k];
        let c_k1 = abs_counts[k + 1];
        if c_k < min_count || c_k1 < min_count {
            continue;
        }
        let c_k_f = c_k as f64;
        let c_k1_f = c_k1 as f64;

        let ln_r_hat = (c_k1_f / c_k_f).ln();
        let ln_r_theory = ln_ratio_theory(k);

        let se = (1.0 / c_k1_f + 1.0 / c_k_f).sqrt();
        let w = 1.0 / (se * se);

        weighted_err_sum += w * (ln_r_hat - ln_r_theory);
        weight_sum += w;
    }

    assert!(
        weight_sum > 0.0,
        "insufficient counts for ratio test: {label} (increase N or adjust k-range/kmax)"
    );

    let avg_err = weighted_err_sum / weight_sum;
    let se_avg = (1.0 / weight_sum).sqrt();
    assert_close_normal(avg_err, 0.0, se_avg, label);
}

mod laplace {
    use crate::traits::samplers::{sample_discrete_laplace, test::check_chi_square_from_probs};

    use super::*;

    #[derive(Clone, Debug)]
    struct LaplaceTheory {
        a: f64,   // exp(-1/scale)
        p0: f64,  // P(X=0)
        var: f64, // Var(X)
    }

    impl LaplaceTheory {
        fn build(scale: f64) -> Self {
            assert!(scale.is_finite() && scale > 0.0);
            let a = (-(1.0 / scale)).exp();
            let p0 = (1.0 - a) / (1.0 + a);
            let var = 2.0 * a / ((1.0 - a) * (1.0 - a));
            Self { a, p0, var }
        }

        fn p_signed(&self, x: i64) -> f64 {
            self.p0 * self.a.powi(x.abs() as i32)
        }

        fn tail_abs_ge(&self, t: usize) -> f64 {
            if t == 0 {
                1.0
            } else {
                (2.0 * self.p0 * self.a.powi(t as i32) / (1.0 - self.a)).clamp(0.0, 1.0)
            }
        }

        fn ln_abs_ratio(&self, _k: usize) -> f64 {
            // for k>=1: P(|X|=k+1)/P(|X|=k) = a
            self.a.ln()
        }
    }

    #[test]
    fn scale_zero_is_always_zero() {
        let s0 = rbig!(0);
        for _ in 0..N_DEGENERATE {
            let x = sample_discrete_laplace(s0.clone()).unwrap();
            assert!(x.is_zero());
        }
    }

    #[test]
    fn p0_and_mean_match_theory_at_fixed_points() {
        let scales = vec![rbig!(1 / 2), rbig!(1), rbig!(2), rbig!(3)];

        for s in scales {
            if s.is_zero() {
                continue;
            }
            let th = LaplaceTheory::build(s.to_f64().value());

            let mut sum: i128 = 0;

            for _ in 0..N_LAPLACE {
                let x = sample_discrete_laplace(s.clone()).unwrap();
                let xi = i64::try_from(x).unwrap();
                sum += xi as i128;
            }

            let mean_hat = (sum as f64) / (N_LAPLACE as f64);

            let se_mean = (th.var / (N_LAPLACE as f64)).sqrt();
            assert_close_normal(mean_hat, 0.0, se_mean, "laplace mean");
        }
    }

    #[test]
    fn per_k_symmetry_matches_theory() {
        let s = rbig!(1);
        let th = LaplaceTheory::build(s.to_f64().value());
        let kmax = 8usize;

        let mut pos = vec![0u64; kmax + 1];
        let mut neg = vec![0u64; kmax + 1];

        for _ in 0..N_LAPLACE {
            let x = sample_discrete_laplace(s.clone()).unwrap();
            let xi = i64::try_from(x).unwrap();
            if xi > 0 && (xi as usize) <= kmax {
                pos[xi as usize] += 1;
            } else if xi < 0 && ((-xi) as usize) <= kmax {
                neg[(-xi) as usize] += 1;
            }
        }

        for k in 1..=kmax {
            let pk = th.p_signed(k as i64);
            let se = (pk * (1.0 - pk) / (N_LAPLACE as f64)).sqrt();

            let pk_hat = pos[k] as f64 / (N_LAPLACE as f64);
            let nk_hat = neg[k] as f64 / (N_LAPLACE as f64);

            assert_close_normal(pk_hat, pk, se, "P(+k)");
            assert_close_normal(nk_hat, pk, se, "P(-k)");

            let se_diff = (se * se + se * se).sqrt();
            assert_close_normal(pk_hat, nk_hat, se_diff, "laplace symmetry");
        }
    }

    #[test]
    fn goodness_of_fit_chi_square_binned() {
        let s = rbig!(1);
        let th = LaplaceTheory::build(s.to_f64().value());

        let kmax = 9usize;
        let hist = SignedHist::sample(N_LAPLACE, kmax, || {
            sample_discrete_laplace(s.clone()).unwrap()
        });

        let bins = 2 * kmax + 1;
        let mut probs = vec![0f64; bins + 1];

        for idx in 0..bins {
            let x = (idx as i64) - (kmax as i64);
            probs[idx] = th.p_signed(x);
        }
        probs[bins] = (1.0 - probs[..bins].iter().sum::<f64>()).max(0.0);

        check_chi_square_from_probs(&hist.counts, &probs).unwrap()
    }

    #[test]
    fn sharp_invariants() {
        let scales = vec![rbig!(1 / 2), rbig!(1), rbig!(2), rbig!(3)];
        let kmax = 60usize;

        for s in scales {
            if s.is_zero() {
                continue;
            }
            let th = LaplaceTheory::build(s.to_f64().value());

            let hist = SignedHist::sample(N_LAPLACE, kmax, || {
                sample_discrete_laplace(s.clone()).unwrap()
            });

            let abs = hist.abs_counts();
            let n = N_LAPLACE as f64;

            // P(0)
            let p0_hat = abs[0] as f64 / n;
            assert_close_binomial_mean(p0_hat, th.p0, N_LAPLACE, "laplace P(0)");

            // Tail checks P(|X|>=t)
            for &t in &[1usize, 2usize, 3usize, 5usize, 8usize] {
                let phat = hist.tail_abs_ge(t) as f64 / n;
                let p = th.tail_abs_ge(t);
                assert_close_binomial_mean(phat, p, N_LAPLACE, "laplace tail");
            }

            // Adjacent abs ratio on log scale (constant ln(a))
            assert_log_ratio_matches(
                &abs,
                2..=12,
                |k| th.ln_abs_ratio(k),
                50,
                "laplace log-ratio slope",
                assert_close_normal,
            );
        }
    }
}

mod gaussian {
    use crate::traits::samplers::test::check_chi_square_from_probs;

    use super::*;

    #[derive(Clone, Debug)]
    struct GaussTheory {
        sigma: f64,
        z: f64,
        e2: f64,
        e4: f64,
        k_trunc: i64,
    }

    impl GaussTheory {
        fn build(sigma: f64) -> Self {
            assert!(sigma.is_finite() && sigma > 0.0);

            let mut z = 1.0_f64;
            let mut m2 = 0.0_f64;
            let mut m4 = 0.0_f64;
            let mut k_trunc = 0_i64;

            for k in 1..=300_i64 {
                let kk = (k as f64) * (k as f64);
                let wk = (-(kk) / (2.0 * sigma * sigma)).exp();
                let pair = 2.0 * wk;

                z += pair;
                m2 += pair * kk;
                m4 += pair * kk * kk;

                k_trunc = k;

                if pair < 1e-16 * z {
                    break;
                }
            }

            Self {
                sigma,
                z,
                e2: m2 / z,
                e4: m4 / z,
                k_trunc,
            }
        }

        fn w(&self, k: i64) -> f64 {
            let kk = (k as f64) * (k as f64);
            (-(kk) / (2.0 * self.sigma * self.sigma)).exp()
        }

        fn p_signed(&self, k: i64) -> f64 {
            self.w(k) / self.z
        }

        fn tail_abs_ge(&self, t: usize) -> f64 {
            assert!(t >= 1);
            let mut tail = 0.0;
            for k in (t as i64)..=self.k_trunc {
                tail += self.w(k);
            }
            (2.0 * tail / self.z).clamp(0.0, 1.0)
        }

        fn ln_ratio_k(&self, k: usize) -> f64 {
            // ln(P(k+1)/P(k)) = - (2k+1)/(2 sigma^2)
            -((2 * k + 1) as f64) / (2.0 * self.sigma * self.sigma)
        }

        fn choose_kmax_for_expected(self: &Self, n: usize, min_exp: f64, hard_cap: usize) -> usize {
            let n_f = n as f64;
            for k in (0..=hard_cap).rev() {
                let pk = self.p_signed(k as i64);
                if n_f * pk >= min_exp {
                    return k;
                }
            }
            0
        }

        fn slope_theory(&self) -> f64 {
            -1.0 / (2.0 * self.sigma * self.sigma)
        }
    }

    #[test]
    fn scale_zero_is_always_zero() {
        let s0 = rbig!(0);
        for _ in 0..N_DEGENERATE {
            let x = sample_discrete_gaussian(s0.clone()).unwrap();
            assert!(x.is_zero());
        }
    }

    #[test]
    fn mean_and_second_moment_match_theory() {
        let scales = vec![rbig!(1), rbig!(2), rbig!(3)];

        for s in scales {
            if s.is_zero() {
                continue;
            }
            let th = GaussTheory::build(s.to_f64().value());

            // empirical mean + m2 in one pass
            let mut sum: i128 = 0;
            let mut sum2: f64 = 0.0;

            for _ in 0..N_GAUSS {
                let x = sample_discrete_gaussian(s.clone()).unwrap();
                let xi = i64::try_from(x).unwrap();
                sum += xi as i128;
                sum2 += (xi as f64) * (xi as f64);
            }

            let mean_hat = (sum as f64) / (N_GAUSS as f64);
            let m2_hat = sum2 / (N_GAUSS as f64);

            let se_mean = (m2_hat / (N_GAUSS as f64)).sqrt();
            assert_close_normal(mean_hat, 0.0, se_mean, "gauss mean");

            let var_x2 = (th.e4 - th.e2 * th.e2).max(0.0);
            let se_m2 = (var_x2 / (N_GAUSS as f64)).sqrt();
            assert_close_normal(m2_hat, th.e2, se_m2, "gauss E[X^2]");
        }
    }

    #[test]
    fn per_k_symmetry_matches_theory() {
        let s = rbig!(1);
        let th = GaussTheory::build(s.to_f64().value());
        let kmax = th.choose_kmax_for_expected(N_GAUSS, 10.0, 25).min(10);

        let mut pos = vec![0u64; kmax + 1];
        let mut neg = vec![0u64; kmax + 1];

        for _ in 0..N_GAUSS {
            let x = sample_discrete_gaussian(s.clone()).unwrap();
            let xi = i64::try_from(x).unwrap();
            if xi > 0 && (xi as usize) <= kmax {
                pos[xi as usize] += 1;
            } else if xi < 0 && ((-xi) as usize) <= kmax {
                neg[(-xi) as usize] += 1;
            }
        }

        for k in 1..=kmax {
            let pk = th.p_signed(k as i64);
            let se = (pk * (1.0 - pk) / (N_GAUSS as f64)).sqrt();

            let pk_hat = pos[k] as f64 / (N_GAUSS as f64);
            let nk_hat = neg[k] as f64 / (N_GAUSS as f64);

            assert_close_normal(pk_hat, pk, se, "P(+k)");
            assert_close_normal(nk_hat, pk, se, "P(-k)");

            let se_diff = (se * se + se * se).sqrt();
            assert_close_normal(pk_hat, nk_hat, se_diff, "gauss symmetry");
        }
    }

    #[test]
    fn discrete_gaussian_goodness_of_fit_binned_chi_square() {
        let s = rbig!(2);
        let th = GaussTheory::build(s.to_f64().value());

        // Choose kmax so edge expected counts are >= 10 (very safe).
        let kmax = th.choose_kmax_for_expected(N_GAUSS, 10.0, 60);
        assert!(
            kmax >= 3,
            "kmax too small; increase N_GAUSS or reduce min_exp"
        );

        let hist = SignedHist::sample(N_GAUSS, kmax, || {
            sample_discrete_gaussian(s.clone()).unwrap()
        });

        let bins = 2 * kmax + 1;
        let mut probs = vec![0f64; bins + 1];

        // Expected probs for x in [-kmax..kmax]
        for idx in 0..bins {
            let x = (idx as i64) - (kmax as i64);
            probs[idx] = th.p_signed(x);
        }

        // Tail = P(|X| > kmax)
        let mass = probs[..bins].iter().sum::<f64>();
        probs[bins] = (1.0 - mass).max(0.0);

        // Let the shared chi-square helper enforce expected>=5, compute statistic, and compare.
        check_chi_square_from_probs(&hist.counts, &probs).unwrap()
    }

    #[test]
    fn sharp_invariants() {
        let scales = vec![rbig!(1), rbig!(2), rbig!(3)];
        let kmax = 60usize;

        for s in scales {
            if s.is_zero() {
                continue;
            }
            let th = GaussTheory::build(s.to_f64().value());

            let hist = SignedHist::sample(N_GAUSS, kmax, || {
                sample_discrete_gaussian(s.clone()).unwrap()
            });

            let abs = hist.abs_counts();
            let n = N_GAUSS as f64;

            // Sign symmetry excluding zero
            let (pos, neg) = hist.pos_neg_excl0();
            let nonzero = (pos + neg) as usize;
            if nonzero > 0 {
                let phat = pos as f64 / nonzero as f64;
                assert_close_binomial_mean(phat, 0.5, nonzero, "gauss sign symmetry");
            }

            // Tail checks P(|X|>=t)
            for &t in &[1usize, 2usize, 3usize, 4usize, 6usize] {
                let phat = hist.tail_abs_ge(t) as f64 / n;
                let p = th.tail_abs_ge(t);
                assert_close_binomial_mean(phat, p, N_GAUSS, "gauss tail");
            }

            // Adjacent ratio check on log scale
            assert_log_ratio_matches(
                &abs,
                2..=12,
                |k| th.ln_ratio_k(k),
                50,
                "gauss log-ratio slope",
                assert_close_normal,
            );

            // Quadratic log-shape slope check (heuristic, low-flake)
            // log P(|X|=k) = const - k^2/(2 sigma^2) for k>=1
            let mut xs = Vec::<f64>::new(); // k^2
            let mut ys = Vec::<f64>::new(); // log(count)
            let mut ws = Vec::<f64>::new(); // weight ~ count

            for k in 1..=kmax {
                let c = abs[k];
                if c >= 80 {
                    xs.push((k as f64) * (k as f64));
                    ys.push((c as f64).ln());
                    ws.push(c as f64);
                }
            }

            if xs.len() >= 6 {
                let w_sum: f64 = ws.iter().sum();
                let x_bar: f64 = xs.iter().zip(ws.iter()).map(|(x, w)| x * w).sum::<f64>() / w_sum;
                let y_bar: f64 = ys.iter().zip(ws.iter()).map(|(y, w)| y * w).sum::<f64>() / w_sum;

                let mut num = 0.0;
                let mut den = 0.0;
                for ((x, y), w) in xs.iter().zip(ys.iter()).zip(ws.iter()) {
                    num += w * (x - x_bar) * (y - y_bar);
                    den += w * (x - x_bar) * (x - x_bar);
                }
                let slope_hat = num / den;
                let slope_theory = th.slope_theory();

                // Heuristic tolerance: catches large mistakes without flakiness.
                let tol = 0.15 * slope_theory.abs() + 1e-6;
                assert!(
                    (slope_hat - slope_theory).abs() <= tol,
                    "gauss log-shape slope mismatch: slope_hat={}, slope_theory={}, tol={}, sigma={}",
                    slope_hat,
                    slope_theory,
                    tol,
                    th.sigma
                );
            }
        }
    }
}

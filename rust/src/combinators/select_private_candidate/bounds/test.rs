use super::*;

fn tol(x: f64) -> f64 {
    1e-12 * x.abs().max(1.0)
}

fn assert_le_with_slack(lo: f64, exact: f64) {
    assert!(
        lo <= exact + tol(exact),
        "expected lower bound {lo} <= exact {exact}"
    );
}

fn assert_ge_with_slack(hi: f64, exact: f64) {
    assert!(
        hi + tol(exact) >= exact,
        "expected upper bound {hi} >= exact {exact}"
    );
}

fn exact_gamma(x: f64) -> f64 {
    1.0 - (-x).exp()
}

fn exact_log_inv_gamma(x: f64) -> f64 {
    -exact_gamma(x).ln()
}

fn exact_nb_mean(eta: f64, x: f64) -> f64 {
    let q = (-x).exp();
    let gamma = exact_gamma(x);

    if eta == 0.0 {
        q / (gamma * (-gamma.ln()))
    } else {
        eta * q / (gamma * (1.0 - gamma.powf(eta)))
    }
}

fn const_curve(eps: f64) -> Function<f64, f64> {
    Function::new_fallible(move |_| Ok(eps))
}

#[test]
fn test_gamma_bounds() -> Fallible<()> {
    for x in [0.05, 0.2, 1.0, 3.0, 10.0] {
        let lo = gamma_lo_from_x(x)?;
        let hi = gamma_hi_from_x(x)?;
        let exact = exact_gamma(x);

        assert!(0.0 < lo && lo < 1.0);
        assert!(0.0 < hi && hi < 1.0);
        assert!(lo <= hi + tol(hi), "expected lo <= hi");
        assert_le_with_slack(lo, exact);
        assert_ge_with_slack(hi, exact);
    }
    Ok(())
}

#[test]
fn test_log_inv_gamma_bounds() -> Fallible<()> {
    for x in [0.05, 0.2, 1.0, 3.0, 10.0] {
        let lo = log_inv_gamma_lo_from_x(x)?;
        let hi = log_inv_gamma_hi_from_x(x)?;
        let exact = exact_log_inv_gamma(x);

        assert!(lo > 0.0);
        assert!(hi > 0.0);
        assert!(lo <= hi + tol(hi), "expected lo <= hi");
        assert_le_with_slack(lo, exact);
        assert_ge_with_slack(hi, exact);
    }
    Ok(())
}

#[test]
fn test_gamma_pow_eta_hi_is_upper_bound() -> Fallible<()> {
    for eta in [0.1, 0.5, 1.0, 2.5, 5.0] {
        for x in [0.05, 0.2, 1.0, 3.0] {
            let hi = gamma_pow_eta_hi_from_x(eta, x)?;
            let exact = exact_gamma(x).powf(eta);
            assert_ge_with_slack(hi, exact);
        }
    }
    Ok(())
}

#[test]
fn test_expected_nb_mean_upper_bound() -> Fallible<()> {
    for eta in [0.0, 0.5, 1.0, 2.5, 5.0] {
        for x in [0.05, 0.2, 1.0, 3.0] {
            let hi = expected_nb_mean_from_x_hi(eta, x)?;
            let exact = exact_nb_mean(eta, x);
            assert!(hi.is_finite());
            assert_ge_with_slack(hi, exact);
        }
    }
    Ok(())
}

#[test]
fn test_expected_nb_mean_hi_decreases_in_x() -> Fallible<()> {
    let xs = [0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 4.0];

    for eta in [0.0, 0.5, 1.0, 2.0, 5.0] {
        let mut prev = expected_nb_mean_from_x_hi(eta, xs[0])?;
        for &x in &xs[1..] {
            let next = expected_nb_mean_from_x_hi(eta, x)?;
            assert!(
                prev + tol(prev) >= next,
                "expected upper mean bound to decrease in x: eta={eta}, prev={prev}, next={next}, x={x}"
            );
            prev = next;
        }
    }
    Ok(())
}

#[test]
fn test_log_inv_gamma_bounds_decrease_in_x() -> Fallible<()> {
    let xs = [0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 4.0];

    let mut prev_lo = log_inv_gamma_lo_from_x(xs[0])?;
    let mut prev_hi = log_inv_gamma_hi_from_x(xs[0])?;
    for &x in &xs[1..] {
        let next_lo = log_inv_gamma_lo_from_x(x)?;
        let next_hi = log_inv_gamma_hi_from_x(x)?;

        assert!(
            prev_lo + tol(prev_lo) >= next_lo,
            "expected lower bound on log(1/gamma) to decrease in x"
        );
        assert!(
            prev_hi + tol(prev_hi) >= next_hi,
            "expected upper bound on log(1/gamma) to decrease in x"
        );

        prev_lo = next_lo;
        prev_hi = next_hi;
    }
    Ok(())
}

#[test]
fn test_solve_nb_x_from_mean_is_conservative() -> Fallible<()> {
    for eta in [0.0, 0.5, 1.0, 2.0, 5.0] {
        for mean in [1.01, 1.1, 2.0, 5.0, 20.0, 100.0] {
            let x = solve_nb_x_from_mean(mean, eta)?;
            assert!(x.is_finite() && x > 0.0);

            let hi = expected_nb_mean_from_x_hi(eta, x)?;
            assert!(
                hi <= mean + tol(mean),
                "upper-evaluated mean {hi} should not exceed requested mean {mean}"
            );

            let exact = exact_nb_mean(eta, x);
            assert!(
                exact <= mean + tol(mean),
                "exact mean {exact} should not exceed requested mean {mean}"
            );
        }
    }
    Ok(())
}

#[test]
fn test_solve_nb_x_from_mean_boundary_cases() -> Fallible<()> {
    for (eta, mean) in [
        (0.0, 1.000001),
        (1.0, 1.000001),
        (0.0, 1.01),
        (1.0, 1.01),
        (2.0, 1.01),
    ] {
        let x = solve_nb_x_from_mean(mean, eta)?;
        assert!(x.is_finite() && x > 0.0);

        let hi = expected_nb_mean_from_x_hi(eta, x)?;
        assert!(hi <= mean + tol(mean));
    }
    Ok(())
}

#[test]
fn test_conditional_curve_guard() -> Fallible<()> {
    let curve = new_conditional_rdp_curve(const_curve(0.2), 0.7);
    assert!(curve.eval(&2.0)?.is_infinite());
    assert!(curve.eval(&1.5)?.is_infinite());
    Ok(())
}

#[test]
fn test_negative_binomial_curve_guard() -> Fallible<()> {
    let curve = new_negative_binomial_rdp_curve(const_curve(0.2), 1.5, 0.7, 5.0);
    assert!(curve.eval(&1.0)?.is_infinite());
    assert!(curve.eval(&0.8)?.is_infinite());
    Ok(())
}

#[test]
fn test_poisson_curve_guard() -> Fallible<()> {
    let curve = new_poisson_rdp_curve(const_curve(0.2), 2.0);
    assert!(curve.eval(&1.0)?.is_infinite());
    assert!(curve.eval(&0.8)?.is_infinite());
    Ok(())
}

#[test]
fn test_conditional_curve_upper_bounds_naive_formula() -> Fallible<()> {
    let eps = 0.2;
    let x = 0.7;
    let alpha = 3.5;
    let curve = new_conditional_rdp_curve(const_curve(eps), x);
    let got = curve.eval(&alpha)?;

    let gamma = exact_gamma(x);
    let exact =
        eps + ((alpha - 2.0) / (alpha - 1.0)) * eps + 2.0 * (1.0 / gamma).ln() / (alpha - 1.0);

    assert_ge_with_slack(got, exact);
    Ok(())
}

#[test]
fn test_negative_binomial_curve_upper_bounds_naive_formula() -> Fallible<()> {
    let eps = 0.2;
    let eta = 1.7;
    let x = 0.8;
    let mean = 4.0;
    let alpha = 3.0;
    let curve = new_negative_binomial_rdp_curve(const_curve(eps), eta, x, mean);
    let got = curve.eval(&alpha)?;

    let gamma = exact_gamma(x);
    let exact = eps
        + (1.0 + eta) * (1.0 - 1.0 / alpha) * eps
        + ((1.0 + eta) * (1.0 / gamma).ln()) / alpha
        + mean.ln() / (alpha - 1.0);

    assert_ge_with_slack(got, exact);
    Ok(())
}

#[test]
fn test_poisson_curve_zero_mean() -> Fallible<()> {
    let curve = new_poisson_rdp_curve(const_curve(0.2), 0.0);
    assert_eq!(curve.eval(&2.0)?, 0.0);
    assert_eq!(curve.eval(&10.0)?, 0.0);
    Ok(())
}

#[test]
fn test_poisson_curve_admissibility_guard() -> Fallible<()> {
    // For alpha = 3, log(alpha / (alpha - 1)) = log(3/2) ≈ 0.405.
    // Using eps > 0.405 should force +inf.
    let curve = new_poisson_rdp_curve(const_curve(0.5), 2.0);
    assert!(curve.eval(&3.0)?.is_infinite());
    Ok(())
}

#[test]
fn test_poisson_curve_upper_bounds_naive_formula() -> Fallible<()> {
    let eps = 0.2;
    let mean = 2.0;
    let alpha = 3.0;
    let curve = new_poisson_rdp_curve(const_curve(eps), mean);
    let got = curve.eval(&alpha)?;

    let delta = (1.0 / alpha) * (1.0 - 1.0 / alpha).powf(alpha - 1.0);
    let exact = eps + mean * delta + mean.ln() / (alpha - 1.0);

    assert_ge_with_slack(got, exact);
    Ok(())
}

use statrs::function::erf;


/// Algorithm to compute sigma for use in the analytic gaussian mechanism
/// Using Alg.1 and p.19 of [Balle (2018)](https://arxiv.org/pdf/1805.06530.pdf)
///
/// # Arguments
/// * `sensitivity` - Upper bound on the L2 sensitivity of the function you want to privatize.
/// * `epsilon` - Multiplicative privacy loss parameter.
/// * `delta` - Additive privacy loss parameter.
pub(super) fn get_analytic_gaussian_sigma(sensitivity: f64, epsilon: f64, delta: f64) -> f64 {
    // threshold to choose whether alpha is larger or smaller than one
    let delta_0 = b_neg(epsilon, 0.);

    // Branching cases are merged, and a new case added for when alpha exactly 1
    let alpha = if delta == delta_0 {
        1.
    } else {
        // depending on comparison with delta_0, alpha is either lt or gt 1
        // searching for either:
        //     v* = inf{u ∈ R≥0: B−ε(u)≤δ}  (where alpha > 1)
        //     u* = sup{v ∈ R≥0: B+ε(v)≤δ}  (where alpha < 1)
        // define s as a (B+/B-)-agnostic substitution for either u or v

        // use the doubling trick to bound the R≥0 region to the interval:
        let (s_inf, s_sup) = doubling_trick(epsilon, delta, delta_0);

        // run a binary search over either B+ or B- to find s*.
        // by Alg.1, if δ ≥ δ_0, then compute a proxy for u* or v* called s*.
        let tol: f64 = 1e-10f64;
        let s_final = binary_search(s_inf, s_sup, epsilon, delta, delta_0, tol);

        // differentiate s* between the u* and v* based on the sign
        let sign = if delta > delta_0 { -1. } else { 1. };
        // reverse second transform out of simplified optimization space (Alg.1 for finding alpha)
        (1. + s_final / 2.).sqrt() + sign * (s_final / 2.).sqrt()
    };

    // reverse first transform out of simplified optimization space
    // (Alg.1 let σ = α∆/√(2ε), and on p.19, below (11))
    alpha * sensitivity / (2. * epsilon).sqrt()
}
/// Find an s* (where s corresponds to either u or v based on the threshold delta_0),
///     such that B(s) lies within a positive tolerance of delta.
///
/// # Arguments
/// * `s_inf` - lower bound for valid values of s
/// * `s_sup` - upper bound for valid values of s
/// * `epsilon` - Multiplicative privacy loss parameter.
/// * `delta` - Additive privacy loss parameter.
/// * `delta_0` - threshold at which sign should be flipped
/// * `tol` - tolerance for error in delta
fn binary_search(
    mut s_inf: f64, mut s_sup: f64, epsilon: f64, delta: f64, delta_0: f64, tol: f64,
) -> f64 {
    // evaluate either B+ or B- on s
    let s_to_delta = |s: f64| if delta > delta_0 {
        b_neg(epsilon, s)
    } else {
        b_pos(epsilon, s)
    };

    loop {
        let s_mid = s_inf + (s_sup - s_inf) / 2.;
        let delta_prime = s_to_delta(s_mid);

        // stop iterating if tolerance is satisfied
        let diff = delta_prime - delta;
        if (diff.abs() <= tol) && (diff <= 0.) { return s_mid }

        // detect the side that the ideal delta falls into
        let is_left = if delta > delta_0 {
            delta_prime > delta
        } else {
            delta_prime < delta
        };

        // tighten bounds about ideal delta
        if is_left {
            s_sup = s_mid;
        } else {
            s_inf = s_mid;
        }
    }
}

/// Obtain an interval from which to start a binary search
/// Choice of B+ or B- is based on the sign determined by delta_0
/// The paper's example given for v* on B+ is to-- "Find the smallest k in N such that B+_eps(2^k) > delta"
///
/// Returns the interval (2^(k - 1), 2^k)
fn doubling_trick(
    epsilon: f64, delta: f64, delta_0: f64,
) -> (f64, f64) {
    // base case
    let mut s_inf: f64 = 0.;
    let mut s_sup: f64 = 1.;

    // return false when bounds should no longer be doubled
    let predicate = |s: f64| if delta > delta_0 {
        b_neg(epsilon, s) < delta
    } else {
        b_pos(epsilon, s) > delta
    };

    // continue doubling the bounds until Theorem 8's comparison with delta is not satisfied
    while predicate(s_sup) {
        s_inf = s_sup;
        s_sup = 2.0 * s_inf;
    }
    // return an interval of (2^(k - 1), 2^k) to search over
    (s_inf, s_sup)
}

/// B-: Reduced form of inequality (6) for optimization when alpha > 1.
/// Refer to p.19 Proof of Theorem 9.
/// 1. Substitute σ = α∆/sqrt(2ε) into inequality (6)
/// 2. Substitute u = (α−1/α)2/2
fn b_neg(epsilon: f64, s: f64) -> f64 {
    phi((epsilon * s).sqrt()) - epsilon.exp() * phi(-(epsilon * (s + 2.)).sqrt())
}

/// B+: Reduced form of inequality (6) for optimization when alpha < 1.
/// Refer to p.19 Proof of Theorem 9.
fn b_pos(epsilon: f64, s: f64) -> f64 {
    phi(-(epsilon * s).sqrt()) - epsilon.exp() * phi(-(epsilon * (s + 2.)).sqrt())
}

/// Integrate gaussian from -inf to t
/// P(N(0,1)≤t)
///
/// # Arguments
/// * `t` - upper bound for integration
fn phi(t: f64) -> f64 {
    0.5 * (1. + erf::erf(t / 2.0_f64.sqrt()))
}

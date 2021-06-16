use num::Float;

use crate::core::{PrivacyRelation};
use crate::dist::{L2Sensitivity, SmoothedMaxDivergence};
use statrs::function::erf;
use crate::meas::ADDITIVE_GAUSS_CONST;

pub(in crate::meas) fn make_analytic_gaussian_privacy_relation<T: 'static + Clone + Float>(scale: T) -> PrivacyRelation<L2Sensitivity<T>, SmoothedMaxDivergence<T>> {
    PrivacyRelation::new_fallible(move |&d_in: &T, &(eps, del): &(T, T)| {
        let _d_in = num_cast!(d_in.clone(); f64)?;
        let _eps = num_cast!(eps.clone(); f64)?;
        let _del = num_cast!(del.clone(); f64)?;
        let _scale = num_cast!(scale.clone(); f64)?;
        if get_analytic_gaussian_sigma(_eps, _del, _d_in) > _scale { return Ok(false) }

        let _2 = num_cast!(2.; T)?;
        let additive_gauss_const = num_cast!(ADDITIVE_GAUSS_CONST; T)?;

        if d_in.is_sign_negative() {
            return fallible!(InvalidDistance, "analytic gaussian mechanism: input sensitivity must be non-negative")
        }
        if eps.is_sign_negative() || eps.is_zero() {
            return fallible!(InvalidDistance, "analytic gaussian mechanism: epsilon must be positive")
        }
        if del.is_sign_negative() || del.is_zero() {
            return fallible!(InvalidDistance, "analytic gaussian mechanism: delta must be positive")
        }

        // TODO: should we error if epsilon > 1., or just waste the budget?
        Ok(eps.min(T::one()) >= (d_in / scale) * (additive_gauss_const + _2 * del.recip().ln()).sqrt())
    })
}


/// Integrate gaussian from -inf to t
/// P(N(0,1)≤t)
///
/// # Arguments
/// * `t` - upper bound for integration
fn phi(t: f64) -> f64 {
    0.5 * (1. + erf::erf(t / 2.0_f64.sqrt()))
}

/// B^-: Reduced form of inequality (6) for optimization when alpha > 1.
/// Refer to p.19 Proof of Theorem 9.
/// 1. Substitute σ = α∆/sqrt(2ε) into inequality (6)
/// 2. Substitute u = (α−1/α)2/2
fn case_a(epsilon: f64, s: f64) -> f64 {
    phi((epsilon * s).sqrt()) - epsilon.exp() * phi(-(epsilon * (s + 2.)).sqrt())
}

/// B^+: Reduced form of inequality (6) for optimization when alpha > 1.
/// Refer to p.19 Proof of Theorem 9.
fn case_b(epsilon: f64, s: f64) -> f64 {
    phi(-(epsilon * s).sqrt()) - epsilon.exp() * phi(-(epsilon * (s + 2.)).sqrt())
}

/// Obtain an interval from which to start a binary search
/// Choice of B^+ or B^- is based on the sign determined by delta_thr
/// The paper's example given for v* on B+ is to-- "Find the smallest k in N such that B^+_eps(2^k) > delta"
///
/// Returns the interval (2^(k - 1), 2^k)
fn doubling_trick(
    epsilon: f64, delta: f64, delta_thr: f64,
) -> (f64, f64) {
    // base case
    let mut s_inf: f64 = 0.;
    let mut s_sup: f64 = 1.;

    // return false when bounds should no longer be doubled
    let predicate = |s: f64| if delta > delta_thr {
        case_a(epsilon, s) < delta
    } else {
        case_b(epsilon, s) > delta
    };

    // continue doubling the bounds until Theorem 8's comparison with delta is not satisfied
    while predicate(s_sup) {
        s_inf = s_sup;
        s_sup = 2.0 * s_inf;
    }
    // return an interval of (2^(k - 1), 2^k) to search over
    (s_inf, s_sup)
}

/// Find an s* (where s corresponds to either u or v based on delta_threshold),
///     such that B(s) lies within a positive tolerance of delta.
///
/// # Arguments
/// * `s_inf` - lower bound for valid values of s
/// * `s_sup` - upper bound for valid values of s
/// * `epsilon` - Multiplicative privacy loss parameter.
/// * `delta` - Additive privacy loss parameter.
/// * `delta_thr` - threshold at which sign should be flipped
/// * `tol` - tolerance for error in delta
fn binary_search(
    mut s_inf: f64, mut s_sup: f64, epsilon: f64, delta: f64, delta_thr: f64, tol: f64,
) -> f64 {
    // evaluate either B+ or B- on s
    let s_to_delta = |s: f64| if delta > delta_thr {
        case_a(epsilon, s)
    } else {
        case_b(epsilon, s)
    };

    loop {
        let s_mid = s_inf + (s_sup - s_inf) / 2.;
        let delta_prime = s_to_delta(s_mid);

        // stop iterating if tolerance is satisfied
        let diff = delta_prime - delta;
        if (diff.abs() <= tol) && (diff <= 0.) { return s_mid }

        // detect the side that the ideal delta falls into
        let is_left = if delta > delta_thr {
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

/// Algorithm to compute sigma for use in the analytic gaussian mechanism
/// Using p.9, p.19 of [Balle (2018)](https://arxiv.org/pdf/1805.06530.pdf)
///
/// # Arguments
/// * `epsilon` - Multiplicative privacy loss parameter.
/// * `delta` - Additive privacy loss parameter.
/// * `sensitivity` - Upper bound on the L2 sensitivity of the function you want to privatize.
pub fn get_analytic_gaussian_sigma(epsilon: f64, delta: f64, sensitivity: f64) -> f64 {
    // threshold to choose whether alpha is larger or smaller than one
    let delta_thr = case_a(epsilon, 0.);

    // Algorithm 1
    let alpha = if delta == delta_thr {
        1.
    } else {
        // depending on comparison with delta_thr alpha is either negative or positive
        // searching for either:
        //     v* = inf{u ∈ R≥0: B−ε(u)≤δ}  (where alpha positive)
        //     u* = sup{v ∈ R≥0: B+ε(v)≤δ}  (where alpha negative)
        // let s be a B agnostic substitution for either u or v

        // use the doubling trick to bound the R≥0 region to the interval:
        let (s_inf, s_sup) = doubling_trick(epsilon, delta, delta_thr);

        // run a binary search over either B+ or B- to find s*.
        let tol: f64 = 1e-10f64;
        let s_final = binary_search(s_inf, s_sup, epsilon, delta, delta_thr, tol);

        // differentiate s between the u and v based on the sign
        let sign = if delta > delta_thr { -1. } else { 1. };
        // reverse second transform out of simplified optimization space (p.19)
        (1. + s_final / 2.).sqrt() + sign * (s_final / 2.).sqrt()
    };

    // reverse first transform out of simplified optimization space (p.19)
    alpha * sensitivity / (2. * epsilon).sqrt()
}

#[cfg(test)]
mod test_analytic_gaussian {
    use crate::meas::get_analytic_gaussian_sigma;

    #[test]
    fn test_analytic_gaussian_sigma() {
        println!("{:?}", get_analytic_gaussian_sigma(0.5, 1E-10, 1.))
    }
}

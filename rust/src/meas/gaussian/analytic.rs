use statrs::function::erf;

/// Algorithm to compute epsilon used by the analytic gaussian mechanism
/// A modification of Alg.1 and p.19 of [Balle (2018)](https://arxiv.org/pdf/1805.06530.pdf)
///
/// # Arguments
/// * `sensitivity` - Upper bound on the L2 sensitivity of the function you want to privatize.
/// * `sigma` - Noise scale.
/// * `delta` - Additive privacy loss parameter.
pub(super) fn get_analytic_gaussian_epsilon(sensitivity: f64, sigma: f64, delta: f64) -> f64 {
    if sigma == 0. {
        return f64::INFINITY
    }
    // threshold to choose whether alpha is larger or smaller than one
    let delta_0 = b_neg(sensitivity, sigma, 0.);

    // Branching cases are merged, and a new case added for when alpha exactly 1
    let alpha = if delta == delta_0 {
        1.
    } else {
        // depending on comparison with delta_0, alpha is either lt or gt 1
        // searching for either:
        //     v* = inf{u ∈ R≥0: B−σ(u)≤δ}  (where alpha > 1)
        //     u* = sup{v ∈ R≥0: B+σ(v)≤δ}  (where alpha < 1)
        // define s as a (B+/B-)-agnostic substitution for either u or v

        // use the doubling trick to bound the R≥0 region to the interval:
        let (s_inf, s_sup) = doubling_trick(sensitivity, sigma, delta, delta_0);

        // run a binary search over either B+ or B- to find s*.
        // by Alg.1, if δ ≥ δ_0, then compute a proxy for u* or v* called s*.
        let tol: f64 = 1e-10f64;
        let s_final = binary_search_s(s_inf, s_sup, sensitivity, sigma, delta, delta_0, tol);

        // differentiate s* between the u* and v* based on the sign
        let sign = if delta > delta_0 { -1. } else { 1. };

        // reverse second transform out of simplified optimization space (Alg.1 for finding alpha)
        1. + sign * 2. * s_final.sqrt()
    };

    // reverse first transform out of simplified optimization space
    // (Alg.1, but let ε = α(∆/σ)^2, and on p.19, below (11))
    alpha * (sensitivity / sigma).powi(2) / 2.
}


/// Find an s* (where s corresponds to either u or v based on the threshold delta_0),
///     such that B(s) lies within a positive tolerance of delta.
///
/// # Arguments
/// * `s_inf` - lower bound for valid values of s
/// * `s_sup` - upper bound for valid values of s
/// * `sensitivity` - Upper bound on the L2 sensitivity of the function you want to privatize.
/// * `sigma` - Noise scale.
/// * `delta` - Additive privacy loss parameter.
/// * `delta_0` - threshold at which sign should be flipped
/// * `tol` - tolerance for error in delta
fn binary_search_s(
    mut s_inf: f64, mut s_sup: f64, sensitivity: f64, sigma: f64, delta: f64, delta_0: f64, tol: f64,
) -> f64 {
    // evaluate either B+ or B- on s
    let s_to_delta = |s: f64| if delta > delta_0 {
        b_neg(sensitivity, sigma, s)
    } else {
        b_pos(sensitivity, sigma, s)
    };

    loop {
        let s_mid = s_inf + (s_sup - s_inf) / 2.;
        let delta_prime = s_to_delta(s_mid);
        // println!("all s: {} {} {}", s_inf, s_mid, s_sup);
        // println!("delta, delta': {} {}", delta, delta_prime);

        // stop iterating if tolerance is satisfied
        let diff = delta_prime - delta;
        if (diff.abs() <= tol) && (diff <= 0.) { return s_mid }

        // detect the side that the ideal delta falls into
        let is_left = if delta > delta_0 {
            delta_prime > delta // looking for sup
        } else {
            delta_prime < delta // looking for inf
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
/// We instead search on B+_σ because we are looking for epsilon.
///
/// Returns the interval (2^(k - 1), 2^k)
fn doubling_trick(
    sensitivity: f64, sigma: f64, delta: f64, delta_0: f64,
) -> (f64, f64) {
    // base case
    let mut s_inf: f64 = 0.;
    let mut s_sup: f64 = 1.;

    // return false when bounds should no longer be doubled
    let predicate = |s: f64| if delta > delta_0 {
        b_neg(sensitivity, sigma, s) < delta
    } else {
        b_pos(sensitivity, sigma, s) > delta
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
/// Refer to p.19 Proof of Theorem 9, but use alternate substitution for ε and v
/// 1. Substitute ε = α(∆/σ)^2 into inequality (6)
/// 2. Substitute v = (1-α)^2/4
fn b_pos(sens: f64, sigma: f64, s: f64) -> f64 {
    let t1 = (1. + 2. * s.sqrt()) * (sens / sigma).powi(2) / 2.;
    // println!("pos t1.exp: {}", t1.exp());
    // println!("pos t2.phi: {}", phi(-sens / sigma * (1. + 2. * s.sqrt())));
    phi(-sens * s.sqrt() / sigma) - t1.exp() * phi(-sens / sigma * (1. + s.sqrt()))
}

/// B+: Reduced form of inequality (6) for optimization when alpha < 1.
/// Refer to p.19 Proof of Theorem 9, but use alternate substitution for ε and v
/// 1. Substitute ε = α(∆/σ)^2 into inequality (6)
/// 2. Substitute u = (α−1)^2/4
fn b_neg(sens: f64, sigma: f64, s: f64) -> f64 {
    let t1 = (1. - 2. * s.sqrt()) * (sens / sigma).powi(2) / 2.;
    // println!("neg lt:     {}", phi(sens * s.sqrt() / sigma));
    // println!("neg t1.exp: {}", t1.exp());
    // println!("neg t2.phi: {}", phi(-sens / sigma * (1. + 2. * s.sqrt())));
    phi(sens * s.sqrt() / sigma) - t1.exp() * phi(-sens / sigma * (1. + s.sqrt()))
}

/// Integrate gaussian from -inf to t
/// P(N(0,1)≤t)
///
/// # Arguments
/// * `t` - upper bound for integration
fn phi(t: f64) -> f64 {
    0.5 * (1. + erf::erf(t / 2.0_f64.sqrt()))
}


#[cfg(test)]
mod tests {
    use crate::{meas::make_base_analytic_gaussian, error::Fallible, dom::AllDomain};

    use super::*;

    #[test]
    fn test_analytic_pos() {
        let tests = vec![
            (15.552795737560736, 7.5742261696821, (1.6623883316111547, 8.954557110260894e-05)),
            (25.927138353305395, 8.169823627211475, (1.0326333767062592, 7.250444741274898e-05)),
            (17.73263011499873, 7.133920366666498, (1.5064609686373784, 1.8051662152236247e-05)),
            (12.73179476133009, 8.267586303049015, (2.3162976621818494, 8.818012980703979e-05)),
            (23.325538382267975, 8.365186466138706, (1.1728633871867087, 9.047615077681805e-05)),
            (10.928764956912273, 7.512095797453234, (2.5791938326776607, 4.9301528493363514e-05)),
            (17.69906161633785, 9.34637235586042, (1.8325999308324243, 8.418141807908971e-05)),
            (5.805531923346137, 4.456851400832124, (2.9186379691180506, 5.2209334924349926e-05)),
            (20.468534936959735, 5.343654842958727, (0.8131681573358981, 9.856463867626194e-05)),
            (42.16583150573247, 9.788667202620704, (0.7739754028207885, 3.799040993351467e-05)),
        ];
        for (scale, d_in, d_out) in tests {
            harness_make_gaussian_mechanism_analytic(scale, d_in, d_out).unwrap();
        }
    }

    #[test]
    fn test_analytic_neg() {
        let tests = vec![
            (3.3784348369854293, 0.027475858897771484, (8.948891788456845e-09, 0.003244471792578877)),
            (6.4908971942181575, 0.02788850904671463, (9.783720228927065e-09, 0.0017140720016322816)),
            (4.003890346401021, 0.030326835266992945, (5.3561319817683206e-09, 0.003021715421715635)),
            (1.1966092399315538, 0.018879863988389083, (7.433842088070123e-09, 0.006294363437238998)),
            (0.7738839891267795, 0.01479344083543148, (9.658862563908917e-09, 0.007625995032569945)),
            (1.2795588439019825, 0.02531303440599442, (9.925119670547303e-09, 0.00789199242593334)),
            (0.767196419163509, 0.009930826555888781, (4.919811339384654e-09, 0.005163993154124557)),
            (1.8091709057918268, 0.026453124794319428, (9.917666906586758e-09, 0.005833150956649362)),
            (1.008954579159222, 0.024764506881924366, (4.290267472997025e-09, 0.009791678356943482)),
            (1.3672597854937922, 0.01543760099251167, (6.010415820922569e-09, 0.0045043926574913126)),
        ];
        for (scale, d_in, d_out) in tests {
            harness_make_gaussian_mechanism_analytic(scale, d_in, d_out).unwrap();
        }
    }

    fn harness_make_gaussian_mechanism_analytic(scale: f64, d_in: f64, d_out: (f64, f64)) -> Fallible<()> {
        let measurement = make_base_analytic_gaussian::<AllDomain<_>>(scale)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        let epsilon = measurement.map(&d_in)?.epsilon(&d_out.1)?;

        println!("epsilon {}, d_out.0 {}, diff {}", epsilon, d_out.0, epsilon - d_out.0);
        // upper bound is pretty loose. b_neg is suffering from instability
        assert!(epsilon <= d_out.0 + 1e-3);
        assert!(epsilon >= d_out.0);
        // use the simpler version of the check that suffers from catastrophic cancellation,
        // to check the more complicated algorithm for finding the analytic gaussian scale
        let wedge = 1e-7;
        assert!(catastrophic_analytic_check(scale * (1. + wedge), d_in, d_out));
        assert!(!catastrophic_analytic_check(scale * (1. - wedge), d_in, d_out));

        Ok(())
    }

    fn catastrophic_analytic_check(scale: f64, d_in: f64, d_out: (f64, f64)) -> bool {
        let (eps, del) = d_out;
        // simple shortcut to check the analytic gaussian.
        // suffers from catastrophic cancellation
        fn phi(t: f64) -> f64 {
            0.5 * (1. + erf::erf(t / 2.0_f64.sqrt()))
        }

        let prob_l_xy = phi(d_in / (2. * scale) - eps * scale / d_in);
        let prob_l_yx = phi(-d_in / (2. * scale) - eps * scale / d_in);
        del >= prob_l_xy - eps.exp() * prob_l_yx
    }
}

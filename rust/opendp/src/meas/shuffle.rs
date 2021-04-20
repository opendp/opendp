use std::cmp::Ordering;
use crate::error::*;
use crate::meas::MakeMeasurement4;
use crate::dom::AllDomain;
use crate::dist::{HammingDistance, SmoothedMaxDivergence, SymmetricDistance};
use crate::core::{Measurement, Function, PrivacyRelation, DatasetMetric};
use std::marker::PhantomData;


pub struct ShuffleAmplification<MI> {
    in_distance: PhantomData<MI>
}

pub trait ShuffleAmplificationConstant {
    fn get_stability_constant() -> f64;
}
impl ShuffleAmplificationConstant for ShuffleAmplification<HammingDistance> {
    fn get_stability_constant() -> f64 { 2. }
}
impl ShuffleAmplificationConstant for ShuffleAmplification<SymmetricDistance> {
    fn get_stability_constant() -> f64 { 1. }
}


pub enum ShuffleBound {
    Empirical, Theoretical
}

impl<MI> MakeMeasurement4<AllDomain<f64>, AllDomain<f64>, MI, SmoothedMaxDivergence<f64>, f64, f64, u64, ShuffleBound> for ShuffleAmplification<MI>
    where MI: DatasetMetric<Distance=u32>,
          Self: ShuffleAmplificationConstant {
    fn make4(step_epsilon: f64, step_delta: f64, n: u64, bound: ShuffleBound) -> Fallible<Measurement<AllDomain<f64>, AllDomain<f64>, MI, SmoothedMaxDivergence<f64>>> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(|_arg: &f64| unreachable!("this is not meant to be called")),
            MI::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new_fallible(move |d_in: &u32, (eps, delta): &(f64, f64)| {

                let step_epsilon = step_epsilon * *d_in as f64;
                let step_delta = step_delta * *d_in as f64;

                let epsilon_constraint = (n as f64 / (16. * (2. / delta).ln())).ln();
                if step_epsilon > epsilon_constraint {
                    return fallible!(RelationDebug, "step_epsilon ({:?}) must be <= ln(n / (16 ln(2 / delta))) ({:?})", step_epsilon, epsilon_constraint)
                }

                // TODO: double-check where d_in should be integrated
                let epoch_epsilon = match bound {
                    ShuffleBound::Empirical => compose_epsilon_empirical(step_epsilon, *delta, n)?,
                    ShuffleBound::Theoretical => compose_epsilon_theoretical(step_epsilon, *delta, n)
                } * Self::get_stability_constant();

                // theorem 3.8
                let epoch_delta = compose_delta(*delta, epoch_epsilon, step_epsilon, step_delta, n);
                println!("(eps, del): ({:?}, {:?})", epoch_epsilon, epoch_delta);

                Ok((epoch_epsilon, epoch_delta) < (*eps, *delta))
            }))
        )
    }
}

// theorem 3.8
fn compose_delta(delta: f64, epoch_epsilon: f64, step_epsilon: f64, step_delta: f64, n: u64 ) -> f64 {
    delta + (epoch_epsilon.exp() + 1.) * (1. + (-step_epsilon).exp() / 2.) * (n as f64) * step_delta
}

// theorem 3.1
fn compose_epsilon_theoretical(step_epsilon: f64, delta: f64, n: u64) -> f64 {
    let exp_eps = step_epsilon.exp();
    let x = (exp_eps * (4. / delta).ln() / (n as f64)).sqrt() + exp_eps / (n as f64);
    (1. + step_epsilon.exp_m1() / (exp_eps + 1.) * 8. * x).ln()
}

// Appendix E
pub fn compose_epsilon_empirical(
    step_epsilon: f64, delta: f64, n: u64,
) -> Fallible<f64> {
    let mut eps_inf = 0.;
    let mut eps_sup = step_epsilon;

    let mut delta_prior = f64::NAN;
    let tol = 1e-20f64;

    loop {
        let eps_candidate = eps_inf + (eps_sup - eps_inf) / 2.;
        // the smallest value such that P and Q are (εt,δt)-indistinguishable
        // S is number of steps to skip (for efficiency)
        let delta_candidate = m1(n, step_epsilon, delta, 5, eps_candidate);

        match delta_candidate.partial_cmp(&delta) {
            Some(Ordering::Less) => eps_sup = eps_candidate,
            Some(Ordering::Greater) => eps_inf = eps_candidate,
            Some(Ordering::Equal) => return Ok(eps_candidate),
            None => return fallible!(FailedRelation, "non-comparable delta")
        }

        let is_stuck = delta_prior == delta_candidate;
        let is_close = delta_candidate < delta && (delta - delta_candidate) <= tol;

        if is_close || is_stuck { return Ok(eps_sup) }
        delta_prior = delta_candidate;
    }
}

// Algorithm 5
fn m1(n: u64, step_epsilon: f64, delta_u: f64, skip: u64, epsilon: f64) -> f64 {
    let mut delta_p: f64 = 0.;
    let mut delta_q: f64 = 0.;
    let mut zeta_c: f64 = 0.;
    let p = (-step_epsilon).exp();
    let mut pr_prev = 0.;

    for t in 0..n / skip {

        // if max(δtP,δtQ) > δU then P and Q are not(ε,δU)-indistinguishable so exit.
        if delta_p.max(delta_q) > delta_u { return delta_u; }
        // any further contribution to either δP or δQ will not exceed 1−ζC, so if this is small we exit
        if 1. - zeta_c < delta_p.min(delta_q) { return delta_p.max(delta_q) + 1. - zeta_c; }

        // we estimate the contribution to δP and δQ from the next interval [Bt,Bt+S)
        let (c_min, c_max) = (t * skip, (t + 1) * skip - 1);
        let pr_next = binomial_cdf(c_max, n - 1, p);
        let pr_interval = pr_next - pr_prev;
        pr_prev = pr_next;

        // Step 3a Compute contribution to δP
        let c_p_max = integrate_hockey_stick(c_max, epsilon, step_epsilon, true);
        let c_p_min = integrate_hockey_stick(c_min, epsilon, step_epsilon, true);
        delta_p += pr_interval * c_p_max.max(c_p_min);

        // Step 3b Compute contribution to δQ
        let c_q_max = integrate_hockey_stick(c_max, epsilon, step_epsilon, false);
        let c_q_min = integrate_hockey_stick(c_min, epsilon, step_epsilon, false);
        delta_q += pr_interval * c_q_max.max(c_q_min);

        // Step 3c Compute contribution to ζC
        zeta_c += pr_interval
    }

    delta_p.max(delta_q)
}


/// referenced as B in paper, algorithm 4
fn integrate_hockey_stick(c: u64, epsilon: f64, step_epsilon: f64, b: bool) -> f64 {
    let q = step_epsilon.exp() / (step_epsilon.exp() + 1.);
    let exp_eps = epsilon.exp();
    let epsilon_q = (((exp_eps + 1.) * q - 1.) / ((exp_eps + 1.) * q - exp_eps)).ln();
    let beta = 1. / ((if b { 1. } else { -1. } * epsilon_q).exp() + 1.);
    let tau = beta * (c + 1) as f64;

    // tau truncation does not affect cdf
    let cdf_bin_tau = binomial_cdf(tau as u64, c, 0.5);
    // TODO: tau can be negative
    let cdf_bin_tau_m1 = binomial_cdf((tau - 1.) as u64, c, 0.5);

    let gamma_p = q * cdf_bin_tau + (1. - q) * cdf_bin_tau_m1;
    let gamma_q = (1. - q) * cdf_bin_tau + q * cdf_bin_tau_m1;

    if b {
        gamma_p - exp_eps * gamma_q
    } else {
        (1. - gamma_q) - exp_eps * (1. - gamma_p)
    }
}

// https://stackoverflow.com/a/45869209/10221612
// Pr(Bin(c, p) <= x)
fn binomial_cdf(x: u64, n: u64, p: f64) -> f64 {
    let mut cdf = 0.;
    let mut b = 0.;
    for k in 0..x + 1 {
        if k > 0 { b += ((n - k + 1) as f64).ln() - (k as f64).ln() }
        let log_pmf_k = b + k as f64 * p.ln() + ((n - k) as f64) * (1. - p).ln();
        cdf += log_pmf_k.exp()
    }
    cdf
}


#[cfg(test)]
mod compositor_tests {
    use super::*;
    use crate::dist::HammingDistance;
    use crate::meas::MakeMeasurement4;
    use crate::error::ExplainUnwrap;

    #[test]
    fn theoretical() {
        let amplifier = ShuffleAmplification::<HammingDistance>::make(
            1.0, 1e-8, 1000, ShuffleBound::Theoretical).unwrap_test();
        let check = amplifier.privacy_relation.eval(&1, &(10., 1e-6)).unwrap_test();
        println!("theoretical {:?}", check);
    }

    #[test]
    fn empirical() {
        let amplifier = ShuffleAmplification::<HammingDistance>::make(
            1.0, 1e-8, 1000, ShuffleBound::Empirical).unwrap_test();
        let check = amplifier.privacy_relation.eval(&1, &(10., 1e-6)).unwrap_test();
        println!("empirical {:?}", check);
    }
}
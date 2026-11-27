use opendp_derive::proven;

use crate::{
    error::Fallible,
    measures::curves::{check_alpha, check_delta, check_epsilon},
    traits::SInterval,
};

/// Apply the two-sided approximate-DP theorem directly to a fixed certificate.
/// No tradeoff representation is materialized: its rounded output must not be
/// fed into a convexity-dependent optimizer. Delta/epsilon queries instead use
/// the original certificate.
#[proven(proof_path = "measures/curves/approxdp_beta.tex")]
pub(super) fn beta_via_approx_dp(epsilon: f64, delta: f64, alpha: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;
    check_delta(delta)?;
    check_alpha(alpha)?;
    let one_minus_delta = (SInterval::point(1.0)? - SInterval::point(delta)?)?;
    if alpha == 0.0 {
        return one_minus_delta.lower_f64();
    }
    // Above this threshold neither branch has a positive representable
    // lower bound for nonzero f64 alpha.
    if alpha == 1.0 || epsilon >= 1024.0 {
        return Ok(0.0);
    }

    let epsilon = SInterval::point(epsilon)?;
    let alpha = SInterval::point(alpha)?;
    let zero = SInterval::point(0.0)?;
    let z = (epsilon.clone() + alpha.clone().ln()?)?;
    let t1 = if z.upper_f64()? >= 0.0 {
        zero.clone()
    } else {
        (one_minus_delta.clone() - z.exp()?)?
    };
    let base = (one_minus_delta - alpha)?.max(zero.clone());
    let t2 = ((-epsilon).exp()? * base)?;
    t1.max(t2).max(zero).lower_f64()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn large_epsilon_small_alpha() -> Fallible<()> {
        let beta = beta_via_approx_dp(710.0, 0.0, 1e-310)?;
        assert!(beta > 0.9 && beta < 1.0);
        Ok(())
    }
}

use dashu::{rational::RBig, rbig};
use opendp_derive::proven;

use crate::{
    error::Fallible,
    measures::curves::{check_alpha, check_delta, check_epsilon},
    traits::{InfCast, SInterval},
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
    if alpha == 1.0 {
        return Ok(0.0);
    }
    let one_minus_delta = RBig::ONE - RBig::try_from(delta)?;
    if alpha == 0.0 {
        return f64::neg_inf_cast(one_minus_delta);
    }
    let epsilon = SInterval::point(epsilon)?;
    let exp_eps_up = RBig::try_from(epsilon.clone().exp()?.upper_f64()?)?;
    let exp_neg_eps_down = RBig::try_from((-epsilon).exp()?.lower_f64()?)?;
    let alpha = RBig::try_from(alpha)?;
    let t1 = &one_minus_delta - exp_eps_up * &alpha;
    let base = (one_minus_delta - alpha).max(rbig!(0));
    let t2 = exp_neg_eps_down * base;
    Ok(f64::neg_inf_cast(t1.max(t2).max(rbig!(0)))?.clamp(0.0, 1.0))
}

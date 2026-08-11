use dashu::{rational::RBig, rbig};
use opendp_derive::proven;

use crate::{
    error::Fallible,
    measures::curves::{ApproxDPPoint, check_alpha, check_delta, check_epsilon},
    traits::{InfCast, SInterval, backend::Dashu},
};

impl ApproxDPPoint {
    /// Build a point while caching directed-rounding bounds for its exponentials.
    pub(crate) fn build((epsilon, delta): (f64, f64)) -> Fallible<Self> {
        check_epsilon(epsilon)?;
        if !epsilon.is_finite() {
            return fallible!(
                FailedMap,
                "epsilon values in privacy profile must be finite"
            );
        }
        check_delta(delta)?;

        let epsilon_cert = SInterval::<Dashu>::point(epsilon)?;
        let exp_eps_up = RBig::try_from(epsilon_cert.clone().exp()?.upper_f64()?)?;
        let exp_neg_eps_down = RBig::try_from((-epsilon_cert.clone())?.exp()?.lower_f64()?)?;

        Ok(Self {
            epsilon: epsilon_cert.upper_f64()?,
            delta,
            one_minus_delta: RBig::ONE - RBig::try_from(delta)?,
            exp_eps_up,
            exp_neg_eps_down,
        })
    }

    #[proven(proof_path = "measures/curves/approxdp_point_beta.tex")]
    #[inline]
    pub(crate) fn beta(&self, alpha: &RBig) -> RBig {
        let t1 = &self.one_minus_delta - &self.exp_eps_up * alpha;
        let base = (&self.one_minus_delta - alpha).max(rbig!(0));
        let t2 = &self.exp_neg_eps_down * base;

        t1.max(t2).max(rbig!(0))
    }
}

/// Convert point-backed ApproxDP information to a conservative symmetric
/// tradeoff. Each point gives a certified lower bound, so the aggregate is
/// their pointwise maximum.
#[allow(non_snake_case)]
pub(crate) fn beta_via_approxDP(points: &[ApproxDPPoint], alpha: f64) -> Fallible<f64> {
    check_alpha(alpha)?;
    let alpha = RBig::try_from(alpha)?;
    let best = points
        .iter()
        .map(|point| point.beta(&alpha))
        .max()
        .unwrap_or_default();

    Ok(f64::neg_inf_cast(best)?.clamp(0.0, 1.0))
}

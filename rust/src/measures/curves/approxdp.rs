use dashu::{rational::RBig, rbig};

use crate::{
    error::Fallible,
    measures::{
        ApproxDPPoint,
        curves::{check_delta, check_epsilon},
    },
    traits::InfCast,
    traits::{D, Interval},
};

impl ApproxDPPoint {
    pub fn build((epsilon, delta): (f64, f64)) -> Fallible<Self> {
        check_epsilon(epsilon)?;
        check_delta(delta)?;

        let epsilon = Interval::<D>::point(epsilon)?;
        let exp_eps = epsilon.clone().exp()?;
        let exp_neg_eps = epsilon.clone().neg()?.exp()?;

        let (_, exp_eps_up) = exp_eps.into_endpoints();
        let (exp_neg_eps_down, _) = exp_neg_eps.into_endpoints();

        Ok(Self {
            epsilon: epsilon.upper_f64()?,
            delta,
            one_minus_delta: RBig::ONE - RBig::try_from(delta)?,
            exp_eps_up: exp_eps_up.to_rbig()?,
            exp_neg_eps_down: exp_neg_eps_down.to_rbig()?,
        })
    }

    #[inline]
    pub fn beta(&self, alpha: &RBig) -> RBig {
        let t1 = &self.one_minus_delta - &self.exp_eps_up * alpha;
        let base = (&self.one_minus_delta - alpha).max(rbig!(0));
        let t2 = &self.exp_neg_eps_down * base;

        t1.max(t2).max(rbig!(0))
    }
}

pub fn beta_via_approxDP(points: &[ApproxDPPoint], alpha: f64) -> Fallible<f64> {
    let alpha = RBig::try_from(alpha)?;

    let best = points
        .iter()
        .map(|p| p.beta(&alpha))
        .max()
        .unwrap_or_default();

    Ok(f64::neg_inf_cast(best)?.clamp(0.0, 1.0))
}

pub fn delta_via_approxDP(points: &[ApproxDPPoint], epsilon: f64) -> Fallible<f64> {
    let idx = points.partition_point(|point| point.epsilon <= epsilon);
    Ok(if idx == 0 { 1.0 } else { points[idx - 1].delta })
}

pub fn epsilon_via_approxdp(points: &[ApproxDPPoint], delta: f64) -> Fallible<f64> {
    check_delta(delta)?;

    if delta == 1.0 {
        return Ok(0.0);
    }

    let idx = points.partition_point(|point| point.delta > delta);

    Ok(if idx == points.len() {
        f64::INFINITY
    } else {
        points[idx].epsilon
    })
}

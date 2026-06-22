use crate::{
    error::Fallible,
    measures::curves::{check_alpha, check_epsilon, check_mu},
    traits::{B, Interval},
};
use std::f64::consts::SQRT_2;

type I = Interval<B>;

#[inline]
fn nonnegative_sub(a: I, b: I) -> Fallible<I> {
    a.sub(b)?.max(I::point(0.0)?)
}

/// Best-effort conservative upper bound on delta_mu(epsilon)
/// under idealized assumptions about Gaussian special-function error.
pub fn delta_via_gaussianDP(mu: f64, epsilon: f64) -> Fallible<f64> {
    check_mu(mu)?;
    check_epsilon(epsilon)?;
    if mu == 0.0 {
        return Ok(0.0);
    }

    let epsilon = I::point(epsilon)?;
    let mu = I::point(mu)?;
    let two = I::point(2.0)?;
    let half = I::point(0.5)?;
    let sqrt2 = I::from_approx(SQRT_2)?;

    let u = epsilon.div(mu.clone())?;
    let h = mu.div(two)?;

    let x = u.clone().sub(h.clone())?.div(sqrt2.clone())?;
    let y = u.add(h)?.div(sqrt2)?;

    let delta = if x.is_nonnegative()? {
        // delta = 0.5 * exp(-x^2) * (erfcx(x) - erfcx(y))
        let x2 = x.clone().mul(x.clone())?;
        let common = half.clone().mul(x2.neg()?.exp()?)?;
        let diff = nonnegative_sub(x.erfcx()?, y.erfcx()?)?;
        common.mul(diff)?
    } else {
        // delta = 0.5 * erfc(x) - 0.5 * exp(-x^2) * erfcx(y)
        let a = half.clone().mul(x.clone().erfc()?)?;
        let x2 = x.clone().mul(x)?;
        let b = half.mul(x2.neg()?.exp()?)?.mul(y.erfcx()?)?;
        nonnegative_sub(a, b)?
    };

    delta.clamp01()?.upper_f64()
}

/// Best-effort conservative lower bound on beta_mu(alpha)
/// under idealized assumptions about Gaussian special-function error.
pub fn beta_via_gaussianDP(mu: f64, alpha: f64) -> Fallible<f64> {
    check_mu(mu)?;
    check_alpha(alpha)?;

    if alpha == 0.0 {
        return Ok(1.0);
    }
    if alpha == 1.0 {
        return Ok(0.0);
    }

    let mu = I::point(mu)?;
    let alpha = I::point(alpha)?;
    let one = I::point(1.0)?;
    let two = I::point(2.0)?;
    let half = I::point(0.5)?;
    let sqrt2 = I::from_approx(SQRT_2)?;

    // p = 2 * (1 - alpha), and beta is increasing in p.
    let p = two.mul(one.sub(alpha)?.clamp(0.0, 1.0)?)?.clamp(0.0, 2.0)?;

    if p.upper_f64()? <= 0.0 {
        return Ok(0.0);
    }
    if p.lower_f64()? >= 2.0 {
        return Ok(1.0);
    }

    // y = erfc_inv(p), and beta is decreasing in y.
    let y = p.erfc_inv()?;

    // q = mu / sqrt(2), and beta is decreasing in q.
    let q = mu.div(sqrt2)?;

    // t = y + q, and beta = 0.5 * erfc(t) is decreasing in t.
    let beta = half.mul(y.add(q)?.erfc()?)?.clamp01()?;

    beta.lower_f64()
}

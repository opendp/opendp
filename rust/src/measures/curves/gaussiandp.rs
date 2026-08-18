use crate::{
    error::Fallible,
    measures::curves::{check_alpha, check_epsilon, check_mu},
    traits::{BInterval, BestEffort, DirectedScalar, Direction, N64},
};
use errorfunctions::RealErrorFunctions;
use statrs::function::erf::{erfc as erfc_f64, erfc_inv as erfc_inv_f64};
use std::f64::consts::SQRT_2;

type I = BInterval;
type GaussianEndpoint = N64<BestEffort>;

fn erfc_round(value: GaussianEndpoint, direction: Direction) -> Fallible<GaussianEndpoint> {
    let value = value.to_f64(direction)?;
    let value = if value == f64::INFINITY {
        0.0
    } else if value == f64::NEG_INFINITY {
        2.0
    } else {
        erfc_f64(value).clamp(0.0, 2.0)
    };
    GaussianEndpoint::approx(value, direction)
}

fn erfcx_round(value: GaussianEndpoint, direction: Direction) -> Fallible<GaussianEndpoint> {
    let value = value.to_f64(direction)?.erfcx();
    if value.is_nan() || value < 0.0 {
        return fallible!(NumericBackend, "erfcx returned invalid value {value}");
    }
    GaussianEndpoint::approx(value, direction)
}

fn erfc_inv_round(value: GaussianEndpoint, direction: Direction) -> Fallible<GaussianEndpoint> {
    let value = value.to_f64(direction)?;
    if !(0.0..=2.0).contains(&value) {
        return fallible!(NumericDomain, "erfc_inv operand is outside [0, 2]");
    }
    let value = match value {
        0.0 => f64::INFINITY,
        2.0 => f64::NEG_INFINITY,
        value => erfc_inv_f64(value),
    };
    GaussianEndpoint::approx(value, direction)
}

fn erfc(value: I) -> Fallible<I> {
    let (lo, hi) = value.into_endpoints();
    I::new(
        erfc_round(hi, Direction::Down)?,
        erfc_round(lo, Direction::Up)?,
    )
}

fn erfcx(value: I) -> Fallible<I> {
    let (lo, hi) = value.into_endpoints();
    I::new(
        erfcx_round(hi, Direction::Down)?,
        erfcx_round(lo, Direction::Up)?,
    )
}

fn erfc_inv(value: I) -> Fallible<I> {
    if value.lower_f64()? < 0.0 {
        return fallible!(NumericDomain, "erfc_inv interval reaches below zero");
    }
    if value.upper_f64()? > 2.0 {
        return fallible!(NumericDomain, "erfc_inv interval reaches above two");
    }
    let (lo, hi) = value.into_endpoints();
    I::new(
        erfc_inv_round(hi, Direction::Down)?,
        erfc_inv_round(lo, Direction::Up)?,
    )
}

#[inline]
fn nonnegative_sub(a: I, b: I) -> Fallible<I> {
    (a - b)?.max(I::point(0.0)?)
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

    let u = (epsilon / mu.clone())?;
    let h = (mu / two)?;

    let x = ((u.clone() - h.clone())? / sqrt2.clone())?;
    let y = ((u + h)? / sqrt2)?;

    let delta = if x.is_nonnegative()? {
        // delta = 0.5 * exp(-x^2) * (erfcx(x) - erfcx(y))
        let x2 = (x.clone() * x.clone())?;
        let common = (half.clone() * ((-x2)?.exp()?))?;
        let diff = nonnegative_sub(erfcx(x)?, erfcx(y)?)?;
        (common * diff)?
    } else {
        // delta = 0.5 * erfc(x) - 0.5 * exp(-x^2) * erfcx(y)
        let a = (half.clone() * erfc(x.clone())?)?;
        let x2 = (x.clone() * x)?;
        let b = ((half * ((-x2)?.exp()?))? * erfcx(y)?)?;
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
    let p = (two * (one - alpha)?.clamp(0.0, 1.0)?)?.clamp(0.0, 2.0)?;

    if p.upper_f64()? <= 0.0 {
        return Ok(0.0);
    }
    if p.lower_f64()? >= 2.0 {
        return Ok(1.0);
    }

    // y = erfc_inv(p), and beta is decreasing in y.
    let y = erfc_inv(p)?;

    // q = mu / sqrt(2), and beta is decreasing in q.
    let q = (mu / sqrt2)?;

    // t = y + q, and beta = 0.5 * erfc(t) is decreasing in t.
    let beta = (half * erfc((y + q)?)?)?.clamp01()?;

    beta.lower_f64()
}

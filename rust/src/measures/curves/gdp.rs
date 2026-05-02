use crate::error::Fallible;
use statrs::function::erf::{erfc, erfc_inv};
use std::f64::consts::SQRT_2;

use crate::traits::{InfExp, NextFloat};
use errorfunctions::RealErrorFunctions;

#[inline]
fn next<T: NextFloat>(x: T, up: bool) -> T {
    if up { x.next_up_() } else { x.next_down_() }
}

#[inline]
fn add(a: f64, b: f64, up: bool) -> f64 {
    next(a + b, up)
}
#[inline]
fn sub(a: f64, b: f64, up: bool) -> f64 {
    next(a - b, up)
}
#[inline]
fn mul(a: f64, b: f64, up: bool) -> f64 {
    next(a * b, up)
}
#[inline]
fn div(a: f64, b: f64, up: bool) -> f64 {
    next(a / b, up)
}

#[inline]
fn nonneg_sub(a: f64, b: f64, up: bool) -> f64 {
    if a <= b { 0.0 } else { sub(a, b, up) }
}

#[inline]
fn to_f32(x: f64, up: bool) -> f32 {
    next(x as f32, up)
}

#[inline]
fn exp_(x: f64, up: bool) -> Fallible<f64> {
    debug_assert!(x <= 0.0 || x.is_infinite());

    if x == f64::NEG_INFINITY {
        return Ok(0.0);
    }

    if up { x.inf_exp() } else { x.neg_inf_exp() }
}

#[inline]
fn erfcx_(x: f64, up: bool) -> Fallible<f64> {
    debug_assert!(x >= 0.0 || (x.is_infinite() && x.is_sign_positive()));

    if x.is_infinite() && x.is_sign_positive() {
        return Ok(0.0);
    }

    let y = x.erfcx();
    let y = next(y, up).max(0.0);
    Ok(y as f64)
}

#[inline]
fn erfc_(x: f64, up: bool) -> Fallible<f64> {
    if x == f64::INFINITY {
        return Ok(0.0);
    }
    if x == f64::NEG_INFINITY {
        return Ok(2.0);
    }

    let y = erfc(x);
    let y = to_f32(y, up);
    let y = next(y, up).clamp(0.0, 2.0);
    Ok(y as f64)
}

#[inline]
fn erfc_inv_(p: f64, up: bool) -> Fallible<f64> {
    if !p.is_finite() || !(0.0..=2.0).contains(&p) {
        return fallible!(FailedMap, "p ({p}) must be finite and in [0, 2]");
    }
    if p == 0.0 {
        return Ok(f64::INFINITY);
    }
    if p == 2.0 {
        return Ok(f64::NEG_INFINITY);
    }

    let y = erfc_inv(p);
    let y = to_f32(y, up);
    let y = next(y, up);
    Ok(y as f64)
}

/// Conservative upper bound on delta_mu(epsilon).
pub fn delta_via_gdp(mu: f64, epsilon: f64) -> Fallible<f64> {
    let up = true;

    if !mu.is_finite() || mu < 0.0 {
        return fallible!(MakeMeasurement, "mu ({mu}) must be finite and non-negative");
    }
    if !epsilon.is_finite() || epsilon < 0.0 {
        return fallible!(
            MakeMeasurement,
            "epsilon ({epsilon}) must be finite and non-negative"
        );
    }
    if mu == 0.0 {
        return Ok(0.0);
    }

    let s2_lo = next(SQRT_2, !up);
    let s2_hi = next(SQRT_2, up);

    let u_lo = div(epsilon, mu, !up);
    let u_hi = div(epsilon, mu, up);
    let h_lo = div(mu, 2.0, !up);
    let h_hi = div(mu, 2.0, up);

    let x_lo = div(sub(u_lo, h_hi, !up), s2_hi, !up);
    let x_hi = div(sub(u_hi, h_lo, up), s2_lo, up);
    let y_hi = div(add(u_hi, h_hi, up), s2_lo, up);

    let delta_hi = if x_lo > 0.0 {
        // delta = 0.5 * exp(-x^2) * (erfcx(x) - erfcx(y))
        let x2_lo = mul(x_lo, x_lo, !up);
        let common_hi = mul(0.5, exp_(-x2_lo, up)?, up);
        let diff_hi = nonneg_sub(erfcx_(x_lo, up)?, erfcx_(y_hi, !up)?, up);
        mul(common_hi, diff_hi, up)
    } else {
        // delta = 0.5 * erfc(x) - 0.5 * exp(-x^2) * erfcx(y)
        let a_hi = mul(0.5, erfc_(x_lo, up)?, up);

        let z = x_lo.abs().max(x_hi.abs());
        let z2_hi = mul(z, z, up);
        let common_lo = mul(0.5, exp_(-z2_hi, !up)?, !up);
        let b_lo = mul(common_lo, erfcx_(y_hi, !up)?, !up);

        nonneg_sub(a_hi, b_lo, up)
    };

    Ok(delta_hi.clamp(0.0, 1.0))
}

/// Conservative lower bound on beta_mu(alpha).
pub fn beta_via_gdp(mu: f64, alpha: f64) -> Fallible<f64> {
    let up = false;
    if !mu.is_finite() || mu < 0.0 {
        return fallible!(
            MakeMeasurement,
            "mu ({mu}) must be a finite non-negative number"
        );
    }
    if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
        return fallible!(
            FailedMap,
            "alpha ({alpha}) must be a finite number in [0, 1]"
        );
    }

    if alpha == 0.0 {
        return Ok(1.0);
    }
    if alpha == 1.0 {
        return Ok(0.0);
    }

    let sqrt_2 = next(SQRT_2, !up);

    // p = 2 * (1 - alpha), and beta is increasing in p
    let p = mul(sub(1.0, alpha, up), 2.0, up).clamp(0.0, 2.0);

    // y = erfc_inv(p), and beta is decreasing in y
    let y = erfc_inv_(p, !up)?;

    // q = mu / sqrt(2), and beta is decreasing in q
    let q = div(mu, sqrt_2, !up);

    // t = y + q, and beta = 0.5 * erfc(t) is decreasing in t
    let t = add(y, q, !up);

    let beta_num = erfc_(t, up)?;
    let beta = div(beta_num, 2.0, up);

    Ok(beta.clamp(0.0, 1.0))
}

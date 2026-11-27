//! Direct optimization of the shared Balle--Canonne bound for linear RDP.

use crate::{error::Fallible, traits::SInterval};

use super::{Bound, Direction, certify, monotone};

#[cfg(test)]
pub(crate) mod test;

// Only choose a candidate order. The selected fixed-order theorem certifies
// the bound independently of the search.
fn search_order(upper_log: f64, derivative_negative: impl Fn(f64, f64) -> bool) -> f64 {
    let mut lo = -52.0_f64;
    let mut hi = upper_log;
    loop {
        let mid = lo + (hi - lo) / 2.0;
        if mid == lo || mid == hi {
            break;
        }
        let t = mid.exp2();
        if derivative_negative(t, 1.0 + t) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    1.0 + hi.exp2()
}

fn zcdp_order(rho: f64, epsilon: f64) -> f64 {
    // Bound the search to finite orders; also avoid unnecessarily large
    // log-delta intermediates when epsilon is close to f64::MAX.
    let cap = (f64::MAX / epsilon.max(1.0)).log2().min(1023.0);
    search_order(cap, |t, alpha| {
        // Half the derivative avoids overflow from 2 * alpha.
        (t * rho + rho / 2.0 - epsilon / 2.0 + (-alpha.recip()).ln_1p() / 2.0).is_sign_negative()
    })
}

fn zcdp_inverse_order(rho: f64, log_delta: f64) -> f64 {
    let sqrt_rho = rho.sqrt();
    search_order(1023.0, |t, _| {
        // The inverse objective has derivative rho + (ln(1+t)+log_delta)/t^2.
        // Compare square roots to avoid overflow in rho*t^2.
        t * sqrt_rho < (-log_delta - t.ln_1p()).max(0.0).sqrt()
    })
}

/// Log-native evaluation of the same fixed-order theorem as zcdp_delta.
pub(crate) fn zcdp_log_delta(rho: f64, epsilon: f64) -> Fallible<f64> {
    if rho.is_nan() || epsilon.is_nan() {
        return fallible!(NumericError, "invalid zCDP conversion parameters");
    }
    if rho < 0.0 {
        return fallible!(FailedMap, "rho ({rho}) must be non-negative");
    }
    if epsilon < 0.0 {
        return fallible!(FailedMap, "epsilon ({epsilon}) must be non-negative");
    }
    if rho.is_infinite() {
        return Ok(0.0);
    }
    if rho == 0.0 || epsilon.is_infinite() {
        return Ok(f64::NEG_INFINITY);
    }

    let order = SInterval::point(zcdp_order(rho, epsilon))?;
    let gamma = (order.clone() * SInterval::point(rho)?)?;
    certify(Direction::Delta, Bound::BalleCanonne, order, gamma, epsilon)
}

/// Invert the fixed-order zCDP theorem directly in log-delta space.
fn zcdp_epsilon_from_log_delta(rho: f64, log_delta: f64) -> Fallible<f64> {
    if rho.is_nan() || log_delta.is_nan() {
        return fallible!(NumericError, "invalid zCDP conversion parameters");
    }
    if rho < 0.0 || log_delta > 0.0 {
        return fallible!(FailedMap, "invalid rho or log-delta budget");
    }
    if rho == 0.0 || log_delta == 0.0 {
        return Ok(0.0);
    }
    if rho.is_infinite() || log_delta.is_infinite() {
        return Ok(f64::INFINITY);
    }

    let order = SInterval::point(zcdp_inverse_order(rho, log_delta))?;
    let gamma = (order.clone() * SInterval::point(rho)?)?;
    certify(
        Direction::Epsilon,
        Bound::BalleCanonne,
        order,
        gamma,
        -log_delta,
    )
}

/// Evaluate nondecreasing epsilon queries at independently certified orders.
pub(crate) fn zcdp_log_delta_batch(rho: f64, targets: &[f64]) -> Fallible<Vec<f64>> {
    let mut values = targets
        .iter()
        .map(|&target| zcdp_log_delta(rho, target))
        .collect::<Fallible<Vec<_>>>()?;
    monotone(&mut values);
    Ok(values)
}

/// Evaluate nondecreasing log-delta budgets at independently certified orders.
pub(crate) fn zcdp_epsilon_batch(rho: f64, targets: &[f64]) -> Fallible<Vec<f64>> {
    let mut values = targets
        .iter()
        .map(|&target| zcdp_epsilon_from_log_delta(rho, target))
        .collect::<Fallible<Vec<_>>>()?;
    monotone(&mut values);
    Ok(values)
}

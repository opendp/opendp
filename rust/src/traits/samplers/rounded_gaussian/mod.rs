//! Exact rounded continuous Gaussian sampling.
//!
//! Target law:
//!
//!     Y = round_T(mu + scale * Z),  Z ~ N(0, 1)
//!
//! where T is f32 or f64, and `mu` and `scale` are finite floats
//! interpreted as exact dyadic rationals.
//!
//! This module uses Karney's exact normal decomposition:
//!
//!     Z = S * (K + X)
//!
//! where:
//! - S is a fair random sign,
//! - K ~ D_{Z_{\ge 0}, 1}, so P[K = k] ∝ exp(-k^2 / 2),
//! - X ~ U(0, 1),
//! - X is accepted with probability exp(-X(2K + X) / 2).
//!
//! The accepted X is represented as a PSRN identity-uniform. The final
//! rounding step refines X until the exact interval for
//!
//!     mu + scale * S(K + X)
//!
//! is contained in one IEEE rounding cell.

use dashu::{integer::UBig, rational::RBig, rbig};

use crate::{
    error::Fallible,
    traits::{
        Float,
        samplers::{PartialSample, PartialUniform01, Uniform01RV, sample_discrete_half_gaussian},
    },
};

use super::sample_standard_bernoulli;

#[cfg(test)]
mod test;

mod native;

/// One Bernoulli trial with success probability `exp(-x)` for lazy `x`.
///
/// This is the generalized von Neumann decreasing-sequence test used by DFW20.
fn sample_bernoulli_exp_uniform(x: &mut PartialUniform01) -> Fallible<bool> {
    // First term: continue iff U_1 < x.
    let mut y = PartialSample::new(Uniform01RV);

    if !x.greater_than(&mut y)? {
        return Ok(true);
    }

    // We have seen one success, so the next failure returns false.
    let mut accept_on_failure = false;

    loop {
        let mut u = PartialSample::new(Uniform01RV);

        // Continue iff U_{n+1} < U_n.
        if !y.greater_than(&mut u)? {
            return Ok(accept_on_failure);
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

/// Return whether `u < x / 2` for lazy uniforms `u` and `x`.
fn sample_bernoulli_u_lt_x2(u: &mut PartialUniform01, x: &mut PartialUniform01) -> Fallible<bool> {
    loop {
        if u.upper().unwrap() * rbig!(2) < x.lower().unwrap() {
            return Ok(true);
        }

        if u.lower().unwrap() * rbig!(2) > x.upper().unwrap() {
            return Ok(false);
        }

        if u.refinements() <= x.refinements().saturating_add(1) {
            u.refine()?;
        } else {
            x.refine()?;
        }
    }
}

/// One Bernoulli trial with success probability `exp(-x^2 / 2)`.
///
/// This is DFW20's `B_{e^{-xy}}` algorithm with parameters `(x/2, x)`.
fn sample_bernoulli_exp_half_x_squared(x: &mut PartialUniform01) -> Fallible<bool> {
    // First continuation event:
    //     u_1 < x / 2
    //     v_1 < x
    let mut w = PartialSample::new(Uniform01RV);

    if !sample_bernoulli_u_lt_x2(&mut w, x)? {
        return Ok(true);
    }

    let mut v = PartialSample::new(Uniform01RV);
    if !x.greater_than(&mut v)? {
        return Ok(true);
    }

    // One completed continuation, so next failure returns false.
    let mut accept_on_failure = false;

    loop {
        let mut u = PartialSample::new(Uniform01RV);

        // Later continuation events use the decreasing-uniform chain:
        //     u_{i+1} < u_i
        if !w.greater_than(&mut u)? {
            return Ok(accept_on_failure);
        }

        let mut v = PartialSample::new(Uniform01RV);
        if !x.greater_than(&mut v)? {
            return Ok(accept_on_failure);
        }

        w = u;
        accept_on_failure = !accept_on_failure;
    }
}

/// Accept the fractional part x with probability
///
///     exp(-x * (2k + x) / 2).
///
/// DFW20 factorizes this as
///
///     exp(-k x) * exp(-x^2 / 2).
fn accept_fraction(k: &UBig, x: &mut PartialUniform01) -> Fallible<bool> {
    let mut remaining = k.clone();

    while !remaining.is_zero() {
        if !sample_bernoulli_exp_uniform(x)? {
            return Ok(false);
        }
        remaining -= UBig::ONE;
    }

    sample_bernoulli_exp_half_x_squared(x)
}

pub(super) struct ExactStdNormal {
    pub(super) negative: bool,
    pub(super) k: RBig,
    pub(super) x: PartialUniform01,
}

/// Sample an exact lazy standard normal in Karney form:
///
///     Z = S * (K + X).
fn sample_exact_std_normal() -> Fallible<ExactStdNormal> {
    loop {
        let k = sample_discrete_half_gaussian(RBig::ONE)?;
        let mut x = PartialSample::new(Uniform01RV);

        if !accept_fraction(&k, &mut x)? {
            continue;
        }

        let negative = sample_standard_bernoulli()?;
        let k = RBig::from(k);

        return Ok(ExactStdNormal { negative, k, x });
    }
}

fn midpoint(lhs: RBig, rhs: RBig) -> RBig {
    (lhs + rhs) / RBig::from(2)
}

fn half_gap(lhs: RBig, rhs: RBig) -> RBig {
    (rhs - lhs) / RBig::from(2)
}

/// Rounding cell for round-to-nearest over the real line.
///
/// Returns `(lower, upper)`, where `None` represents an unbounded side.
/// The returned interval is used only for containment tests; exact midpoint
/// tie events have probability zero under the continuous Gaussian.
fn rounding_cell<T: Float>(y: T) -> Fallible<(Option<RBig>, Option<RBig>)> {
    if y == T::infinity() {
        let max = T::max_value();
        let max_r = max.into_rational()?;
        let prev_r = max.next_down_().into_rational()?;

        let lower = max_r.clone() + half_gap(prev_r, max_r);
        return Ok((Some(lower), None));
    }

    if y == T::neg_infinity() {
        let min = T::min_value();
        let min_r = min.into_rational()?;
        let next_r = min.next_up_().into_rational()?;

        let upper = min_r.clone() - half_gap(min_r, next_r);
        return Ok((None, Some(upper)));
    }

    debug_assert!(y.is_finite());

    // Treat +0 and -0 as the same rounded value.
    if y == T::zero() {
        let lower = midpoint(y.next_down_().into_rational()?, RBig::ZERO);
        let upper = midpoint(RBig::ZERO, y.next_up_().into_rational()?);
        return Ok((Some(lower), Some(upper)));
    }

    let y_r = y.into_rational()?;

    let lower = {
        let down = y.next_down_();

        if down == T::neg_infinity() {
            let up_r = y.next_up_().into_rational()?;
            y_r.clone() - half_gap(y_r.clone(), up_r)
        } else {
            midpoint(down.into_rational()?, y_r.clone())
        }
    };

    let upper = {
        let up = y.next_up_();

        if up == T::infinity() {
            let down_r = y.next_down_().into_rational()?;
            y_r.clone() + half_gap(down_r, y_r.clone())
        } else {
            midpoint(y_r.clone(), up.into_rational()?)
        }
    };

    Ok((Some(lower), Some(upper)))
}

/// Exact rational interval containing
///
///     mu + scale * S(K + X)
///
/// where the current PSRN state gives `X ∈ [x_lo, x_hi]`.
fn normal_interval(z: &ExactStdNormal, mu: &RBig, scale: &RBig) -> (RBig, RBig) {
    let x_lo = z.x.lower().expect("Uniform01 inverse CDF is identity");
    let x_hi = z.x.upper().expect("Uniform01 inverse CDF is identity");

    if z.negative {
        (mu - scale * (&z.k + x_hi), mu - scale * (&z.k + x_lo))
    } else {
        (mu + scale * (&z.k + x_lo), mu + scale * (&z.k + x_hi))
    }
}

fn clip_value(value: RBig, range: &RBig) -> RBig {
    value.max(-range.clone()).min(range.clone())
}

fn clip_interval(interval: (RBig, RBig), range: &RBig) -> (RBig, RBig) {
    let (lo, hi) = interval;
    (clip_value(lo, range), clip_value(hi, range))
}

type RoundingCell = (Option<RBig>, Option<RBig>);

#[derive(Clone, Copy)]
enum Endpoint {
    Lower,
    Upper,
}

/// Exact affine endpoint for the current accepted normal trace prefix.
///
/// This mirrors the native cell-boundary certificate: the final proof step is
/// a comparison of affine endpoints against rounding-cell boundaries, not
/// rounded native interval arithmetic.
fn normal_endpoint(z: &ExactStdNormal, mu: &RBig, scale: &RBig, endpoint: Endpoint) -> RBig {
    let use_upper_fraction = matches!(
        (z.negative, endpoint),
        (false, Endpoint::Upper) | (true, Endpoint::Lower)
    );
    let x = if use_upper_fraction {
        z.x.upper().expect("Uniform01 inverse CDF is identity")
    } else {
        z.x.lower().expect("Uniform01 inverse CDF is identity")
    };

    if z.negative {
        mu - scale * (&z.k + x)
    } else {
        mu + scale * (&z.k + x)
    }
}

fn clipped_normal_endpoint(
    z: &ExactStdNormal,
    mu: &RBig,
    scale: &RBig,
    range: Option<&RBig>,
    endpoint: Endpoint,
) -> RBig {
    let endpoint = normal_endpoint(z, mu, scale, endpoint);
    range
        .map(|range| clip_value(endpoint.clone(), range))
        .unwrap_or(endpoint)
}

fn affine_trace_inside_cell(
    z: &ExactStdNormal,
    mu: &RBig,
    scale: &RBig,
    range: Option<&RBig>,
    cell: &RoundingCell,
) -> bool {
    let x_lo = clipped_normal_endpoint(z, mu, scale, range, Endpoint::Lower);
    let x_hi = clipped_normal_endpoint(z, mu, scale, range, Endpoint::Upper);
    let (c_lo, c_hi) = cell;

    let lower_in = c_lo.as_ref().is_none_or(|c_lo| &x_lo >= c_lo);
    let upper_in = c_hi.as_ref().is_none_or(|c_hi| &x_hi <= c_hi);
    lower_in && upper_in
}

/// Pick a candidate float whose rounding cell contains the rational midpoint
/// of the current interval. Correctness does not rely on this guess; the
/// containment test below is authoritative.
fn candidate_and_cell_from_interval<T: Float>(
    interval: &(RBig, RBig),
) -> Fallible<(T, RoundingCell)> {
    let mid = midpoint(interval.0.clone(), interval.1.clone());
    let y = T::from_rational(mid);
    Ok((y, rounding_cell(y)?))
}

/// Sample exactly from a continuous Gaussian rounded to the nearest f32/f64.
///
/// # Proof Definition
///
/// For finite `mu` and finite nonnegative `scale`, returns either `Err(e)` due
/// to lack of system entropy, or `Ok(y)`, where:
///
/// ```math
/// y = round_T(mu + scale * Z)
/// Z ~ N(0, 1)
/// ```
///
/// The float inputs are interpreted as exact dyadic rationals. The only
/// floating-point operation in the proof path is the final deterministic
/// rounding to type `T`, represented by exact rational rounding-cell tests.
pub fn sample_rounded_gaussian<T: Float>(mu: T, scale: T) -> Fallible<T> {
    if !mu.is_finite() || !scale.is_finite() {
        return fallible!(FailedFunction, "mu and scale must be finite");
    }

    if scale < T::zero() {
        return fallible!(FailedFunction, "scale must be nonnegative");
    }

    if scale == T::zero() {
        return Ok(mu);
    }

    let mu = mu.into_rational()?;
    let scale = scale.into_rational()?;
    let mut z = sample_exact_std_normal()?;

    loop {
        let interval = normal_interval(&z, &mu, &scale);
        let (y, cell) = candidate_and_cell_from_interval::<T>(&interval)?;

        if affine_trace_inside_cell(&z, &mu, &scale, None, &cell) {
            return Ok(y);
        }

        // The current interval straddles a rounding boundary. Refine only the
        // accepted fractional component X; K and sign are already exact.
        z.x.refine()?;
    }
}

/// Sample exactly from a clipped continuous Gaussian rounded to f32/f64.
///
/// For finite `mu`, finite nonnegative `scale`, and finite nonnegative `range`,
/// this implements
///
///     round_T(clip_R(clip_R(mu) + scale * Z)).
///
/// Clipping is deterministic post-processing; values outside the clipping
/// range are not rejected and resampled.
pub fn sample_rounded_gaussian_clipped<T: Float>(mu: T, scale: T, range: T) -> Fallible<T> {
    if !mu.is_finite() || !scale.is_finite() || !range.is_finite() {
        return fallible!(FailedFunction, "mu, scale and range must be finite");
    }

    if scale < T::zero() {
        return fallible!(FailedFunction, "scale must be nonnegative");
    }

    if range < T::zero() {
        return fallible!(FailedFunction, "range must be nonnegative");
    }

    let range = range.into_rational()?;
    let mu = clip_value(mu.into_rational()?, &range);

    if scale == T::zero() {
        return Ok(T::from_rational(mu));
    }

    let scale = scale.into_rational()?;
    let mut z = sample_exact_std_normal()?;

    loop {
        let interval = clip_interval(normal_interval(&z, &mu, &scale), &range);
        let (y, cell) = candidate_and_cell_from_interval::<T>(&interval)?;

        if affine_trace_inside_cell(&z, &mu, &scale, Some(&range), &cell) {
            return Ok(y);
        }

        z.x.refine()?;
    }
}

/// Sample from a continuous Gaussian with f64 input parameters, rounded as a
/// real affine value directly into the extended f32 output lattice.
///
/// This path does not construct a native floating-point noise value and add it
/// to `mu`; it certifies which extended f32 rounding cell contains the exact
/// real value `mu +/- scale32 * (k + x)`, where `scale32` is the smallest
/// finite positive f32 at least as large as `scale`.
///
/// If the native prefix cannot certify one extended f32 cell, the accepted trace is
/// rejected as an unresolved rounding-boundary comb and the sampler restarts.
/// This intentionally avoids exact rational finalization at the cost of the
/// small conditioning term accounted for by the comb probability.
/// Pre-accept sampler caps in the native specialization are declared
/// sampler-side resampling events. For positive scale, the hybrid native sampler
/// profile is proof-grade only for the clipped wrapper below; this unclipped
/// convenience wrapper returns an error unless `scale == 0`.
pub fn sample_rounded_gaussian_f64_to_f32_native(mu: f64, scale: f64) -> Fallible<f32> {
    sample_rounded_gaussian_f64_to_f32_native_clipped(mu, scale, None)
}

/// Sample from a continuous Gaussian with f64 input parameters, optionally
/// clipping the input location and noisy real value to `[-range, range]`
/// before extended f32 rounding.
///
/// When `range` is `None`, this path is available only for `scale == 0`.
/// When `Some(R)`, this implements
///
///     round_ext_f32(clip_R(clip_R(mu) + scale32 * Z)).
///
/// Here `scale32` is the upward-snapped f32 scale used by the native
/// specialization. Privacy accounting for this path must use `scale32`, and
/// the default hybrid profile requires `R < 524288 * scale32`. If `R` is at or
/// below the f32 finite-output threshold, infinities are excluded from the
/// support.
pub fn sample_rounded_gaussian_f64_to_f32_native_clipped(
    mu: f64,
    scale: f64,
    range: Option<f64>,
) -> Fallible<f32> {
    if !mu.is_finite() || !scale.is_finite() {
        return fallible!(FailedFunction, "mu and scale must be finite");
    }
    if scale < 0.0 {
        return fallible!(FailedFunction, "scale must be nonnegative");
    }
    if let Some(range) = range
        && (!range.is_finite() || range < 0.0)
    {
        return fallible!(FailedFunction, "range must be finite and nonnegative");
    }

    let mu = range.map(|range| mu.max(-range).min(range)).unwrap_or(mu);

    if scale == 0.0 {
        return Ok(mu as f32);
    }
    if range.is_none() {
        return fallible!(
            FailedFunction,
            "positive-scale native f64-to-f32 sampler requires a finite clipping range"
        );
    }

    loop {
        match native::sample_f64_to_f32_clipped(mu, scale, range)? {
            native::NativeF32Sample::Output(out) => return Ok(out),
            native::NativeF32Sample::RejectedSampler => continue,
            native::NativeF32Sample::RejectedComb => continue,
            native::NativeF32Sample::ResourceLimit => {
                return fallible!(
                    FailedFunction,
                    "native arithmetic resource limit outside declared resampling events"
                );
            }
        }
    }
}

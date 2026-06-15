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

use dashu::{integer::UBig, rational::RBig, rbig, ubig};

use crate::{
    error::Fallible,
    traits::{
        Float,
        samplers::{
            PartialSample, PartialUniform01, Uniform01, sample_bernoulli_exp,
            sample_geometric_exp_fast,
        },
    },
};

use super::{sample_standard_bernoulli, sample_uniform_ubig_below};

/// Karney's integer primitive:
///
///     P[K = k] ∝ exp(-k^2 / 2), k ∈ {0, 1, 2, ...}.
fn sample_k() -> Fallible<UBig> {
    let half = rbig!(1 / 2);

    loop {
        // Proposal mass is proportional to exp(-candidate / 2).
        let candidate = sample_geometric_exp_fast(half.clone())?;

        // Unit-scale CKS correction term for the one-sided proposal:
        //     (candidate - 1/2)^2 / 2
        let centered = RBig::from(candidate.clone()) - half.clone();
        let bias = centered.clone() * centered / rbig!(2);

        if sample_bernoulli_exp(bias)? {
            return Ok(candidate);
        }
    }
}

/// One Bernoulli trial with success probability
///
///     exp(-x * (2k + x) / (2k + 2)).
///
/// This is Karney's Algorithm B, using the T/C selector transformation
/// to avoid arithmetic on the real-valued lazy uniform `x`.
fn sample_karney_b(k: &UBig, x: &mut PartialUniform01) -> Fallible<bool> {
    let m = k * ubig!(2) + ubig!(2);

    let mut accept_on_failure = true;
    let mut y: Option<PartialUniform01> = None;

    loop {
        // Step B2(a): sample z and require z < y.
        // Initially y is x; after a successful loop, y is the previous z.
        let mut z = PartialSample::new(Uniform01);

        let z_lt_y = match y.as_mut() {
            Some(y_prev) => y_prev.greater_than(&mut z)?,
            None => x.greater_than(&mut z)?,
        };

        if !z_lt_y {
            return Ok(accept_on_failure);
        }

        // Steps B2(b,c), using Karney's T/C selector:
        //
        // r < (2k + x)/(2k + 2)
        //
        // is implemented by:
        // - fail with probability 1/m,
        // - succeed with probability 1 - 2/m,
        // - otherwise compare a fresh r < x.
        debug_assert!(m >= ubig!(2));
        let u = sample_uniform_ubig_below(m.clone())?;

        if u.is_zero() {
            return Ok(accept_on_failure);
        }
        if u.is_one() {
            let mut r = PartialSample::new(Uniform01);

            if !x.greater_than(&mut r)? {
                return Ok(accept_on_failure);
            }
        }

        y = Some(z);
        accept_on_failure = !accept_on_failure;
    }
}

/// Accept the fractional part x with probability
///
///     exp(-x * (2k + x) / 2).
///
/// Karney obtains this by running Algorithm B exactly k + 1 times.
fn accept_fraction(k: &UBig, x: &mut PartialUniform01) -> Fallible<bool> {
    let mut remaining = k.clone();

    loop {
        if !sample_karney_b(k, x)? {
            return Ok(false);
        }

        if remaining.is_zero() {
            return Ok(true);
        }

        remaining -= UBig::ONE;
    }
}

struct ExactStdNormal {
    negative: bool,
    k: RBig,
    x: PartialUniform01,
}

/// Sample an exact lazy standard normal in Karney form:
///
///     Z = S * (K + X).
fn sample_exact_std_normal() -> Fallible<ExactStdNormal> {
    loop {
        let k = sample_k()?;
        let mut x = PartialSample::new(Uniform01);

        if !accept_fraction(&k, &mut x)? {
            continue;
        }

        let negative = sample_standard_bernoulli()?;
        let k = RBig::from(k.as_ibig().clone());

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

type RoundingCell = (Option<RBig>, Option<RBig>);

fn interval_inside_cell(interval: &(RBig, RBig), cell: &RoundingCell) -> bool {
    let (x_lo, x_hi) = interval;
    let (c_lo, c_hi) = cell;

    let lower_in = c_lo.as_ref().is_none_or(|c_lo| x_lo >= c_lo);
    let upper_in = c_hi.as_ref().is_none_or(|c_hi| x_hi <= c_hi);
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

        if interval_inside_cell(&interval, &cell) {
            return Ok(y);
        }

        // The current interval straddles a rounding boundary. Refine only the
        // accepted fractional component X; K and sign are already exact.
        z.x.refine()?;
    }
}

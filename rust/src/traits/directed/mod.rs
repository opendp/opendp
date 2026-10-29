//! Directed scalar arithmetic for OpenDP conservative numerics.

/// Directed rounding requested from a numerical operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Down,
    Up,
}

mod scalar;
mod soft_float;

pub use scalar::{Approximate, Certified, DirectedScalar, DirectedTranscendental, N64};
pub use soft_float::SoftFloat;

use crate::error::Fallible;
use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[cfg(test)]
mod test;

pub type AInterval = Interval<N64<Approximate>>;
pub type CInterval = Interval<N64<Certified>>;
/// Closed interval over the software directed scalar.
pub type SInterval = Interval<SoftFloat>;

/// Closed endpoint interval `[lo, hi]` over a directed scalar.
///
/// Ordered endpoints alone do not guarantee a conservative enclosure: arithmetic
/// is conservative only when the scalar's directed operations are certified.
/// In particular, [`AInterval`] uses approximate native arithmetic.
///
/// Transcendental operations require [`DirectedTranscendental`]; certified
/// native scalars support arithmetic and square root, but not native exp/ln.
///
/// ```
/// use opendp::{error::Fallible, traits::{DirectedScalar, Interval, SoftFloat}};
///
/// fn root<S: DirectedScalar>(x: Interval<S>) -> Fallible<Interval<S>> {
///     x.sqrt()
/// }
/// let result = root(Interval::<SoftFloat>::point(4.0)?)?;
/// assert_eq!(result.upper_f64()?, 2.0);
/// # Ok::<(), opendp::error::Error>(())
/// ```
///
/// ```compile_fail
/// use opendp::traits::CInterval;
/// CInterval::point(1.0)?.exp()?;
/// # Ok::<(), opendp::error::Error>(())
/// ```
///
/// ```compile_fail
/// use opendp::traits::{Certified, Interval, N64};
/// Interval::<N64<Certified>>::point(1.0)?.ln()?;
/// # Ok::<(), opendp::error::Error>(())
/// ```
#[derive(Clone)]
pub struct Interval<S: DirectedScalar> {
    lo: S,
    hi: S,
}

impl<S: DirectedScalar + fmt::Debug> fmt::Debug for Interval<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interval")
            .field("lo", &self.lo)
            .field("hi", &self.hi)
            .finish()
    }
}

impl<S: DirectedScalar> Interval<S> {
    pub fn new(lo: S, hi: S) -> Fallible<Self> {
        if lo > hi {
            return fallible!(
                FailedFunction,
                "interval lower endpoint exceeds upper endpoint"
            );
        }
        Ok(Self { lo, hi })
    }

    pub fn point(value: f64) -> Fallible<Self> {
        let value = S::exact(value)?;
        Self::new(value.clone(), value)
    }

    pub fn between(lo: f64, hi: f64) -> Fallible<Self> {
        Self::new(S::exact(lo)?, S::exact(hi)?)
    }

    pub fn lower(&self) -> &S {
        &self.lo
    }

    pub fn upper(&self) -> &S {
        &self.hi
    }

    pub fn into_endpoints(self) -> (S, S) {
        (self.lo, self.hi)
    }

    pub fn lower_f64(&self) -> Fallible<f64> {
        self.lo.to_f64(Direction::Down)
    }

    pub fn upper_f64(&self) -> Fallible<f64> {
        self.hi.to_f64(Direction::Up)
    }

    pub fn contains_zero(&self) -> bool {
        // Numerically, x <= -x iff x <= 0, without constructing a scalar zero.
        self.lo <= -self.lo.clone() && self.hi >= -self.hi.clone()
    }

    pub fn is_nonnegative(&self) -> bool {
        self.lo >= -self.lo.clone()
    }

    pub fn is_nonpositive(&self) -> bool {
        self.hi <= -self.hi.clone()
    }

    pub fn recip(self) -> Fallible<Self> {
        if self.contains_zero() {
            return fallible!(NumericIndeterminate, "reciprocal interval contains zero");
        }
        let one = S::exact(1.0)?;
        Self::new(
            one.clone().div_round(self.hi, Direction::Down)?,
            one.div_round(self.lo, Direction::Up)?,
        )
    }

    pub fn abs(self) -> Fallible<Self> {
        if self.is_nonnegative() {
            return Ok(self);
        }
        if self.is_nonpositive() {
            return Self::new(-self.hi, -self.lo);
        }
        let hi = self.lo.abs().max(self.hi.abs());
        Self::new(S::exact(0.0)?, hi)
    }

    pub fn abs_upper(self) -> S {
        if self.is_nonnegative() {
            return self.hi;
        }
        if self.is_nonpositive() {
            return -self.lo;
        }
        self.lo.abs().max(self.hi.abs())
    }

    pub fn max(self, rhs: Self) -> Self {
        Self {
            lo: self.lo.max(rhs.lo),
            hi: self.hi.max(rhs.hi),
        }
    }

    pub fn min(self, rhs: Self) -> Self {
        Self {
            lo: self.lo.min(rhs.lo),
            hi: self.hi.min(rhs.hi),
        }
    }

    pub fn clamp(self, min: f64, max: f64) -> Fallible<Self> {
        if min > max {
            return fallible!(FailedFunction, "clamp minimum exceeds maximum");
        }
        Ok(self.max(Self::point(min)?).min(Self::point(max)?))
    }

    pub fn clamp01(self) -> Fallible<Self> {
        self.clamp(0.0, 1.0)
    }

    pub fn sqrt(self) -> Fallible<Self> {
        if !self.is_nonnegative() {
            return fallible!(NumericDomain, "interval extends below the sqrt domain");
        }
        Self::new(
            self.lo.sqrt_round(Direction::Down)?,
            self.hi.sqrt_round(Direction::Up)?,
        )
    }
}

impl<S: DirectedScalar> Add for Interval<S> {
    type Output = Fallible<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.lo.add_round(rhs.lo, Direction::Down)?,
            self.hi.add_round(rhs.hi, Direction::Up)?,
        )
    }
}

impl<S: DirectedScalar> Sub for Interval<S> {
    type Output = Fallible<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.lo.sub_round(rhs.hi, Direction::Down)?,
            self.hi.sub_round(rhs.lo, Direction::Up)?,
        )
    }
}

impl<S: DirectedScalar> Neg for Interval<S> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            lo: -self.hi,
            hi: -self.lo,
        }
    }
}

impl<S: DirectedScalar> Mul for Interval<S> {
    type Output = Fallible<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs_nonnegative = self.is_nonnegative();
        let lhs_nonpositive = self.is_nonpositive();
        let rhs_nonnegative = rhs.is_nonnegative();
        let rhs_nonpositive = rhs.is_nonpositive();

        if lhs_nonnegative && rhs_nonnegative {
            return Self::new(
                self.lo.mul_round(rhs.lo, Direction::Down)?,
                self.hi.mul_round(rhs.hi, Direction::Up)?,
            );
        }
        if lhs_nonpositive && rhs_nonpositive {
            return Self::new(
                self.hi.mul_round(rhs.hi, Direction::Down)?,
                self.lo.mul_round(rhs.lo, Direction::Up)?,
            );
        }
        if lhs_nonnegative && rhs_nonpositive {
            return Self::new(
                self.hi.mul_round(rhs.lo, Direction::Down)?,
                self.lo.mul_round(rhs.hi, Direction::Up)?,
            );
        }
        if lhs_nonpositive && rhs_nonnegative {
            return Self::new(
                self.lo.mul_round(rhs.hi, Direction::Down)?,
                self.hi.mul_round(rhs.lo, Direction::Up)?,
            );
        }

        let lo = min4(
            self.lo.clone().mul_round(rhs.lo.clone(), Direction::Down)?,
            self.lo.clone().mul_round(rhs.hi.clone(), Direction::Down)?,
            self.hi.clone().mul_round(rhs.lo.clone(), Direction::Down)?,
            self.hi.clone().mul_round(rhs.hi.clone(), Direction::Down)?,
        );
        let hi = max4(
            self.lo.clone().mul_round(rhs.lo.clone(), Direction::Up)?,
            self.lo.mul_round(rhs.hi.clone(), Direction::Up)?,
            self.hi.clone().mul_round(rhs.lo, Direction::Up)?,
            self.hi.mul_round(rhs.hi, Direction::Up)?,
        );
        Self::new(lo, hi)
    }
}

impl<S: DirectedScalar> Div for Interval<S> {
    type Output = Fallible<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.contains_zero() {
            return fallible!(NumericIndeterminate, "divisor interval contains zero");
        }
        if rhs.is_nonpositive() {
            return -self / -rhs;
        }
        // For a positive divisor, endpoints toward zero use its upper bound;
        // endpoints away from zero use its lower bound. Divide directly to avoid
        // reciprocal overflow and unnecessary infinity/infinity corner quotients.
        let lower_divisor = if self.is_nonnegative() {
            rhs.hi.clone()
        } else {
            rhs.lo.clone()
        };
        let upper_divisor = if self.is_nonpositive() {
            rhs.hi
        } else {
            rhs.lo
        };
        Self::new(
            self.lo.div_round(lower_divisor, Direction::Down)?,
            self.hi.div_round(upper_divisor, Direction::Up)?,
        )
    }
}

impl<S: DirectedTranscendental> Interval<S> {
    pub fn exp(self) -> Fallible<Self> {
        Self::new(
            self.lo.exp_round(Direction::Down)?,
            self.hi.exp_round(Direction::Up)?,
        )
    }

    pub fn exp_m1(self) -> Fallible<Self> {
        Self::new(
            self.lo.exp_m1_round(Direction::Down)?,
            self.hi.exp_m1_round(Direction::Up)?,
        )
    }

    pub fn ln(self) -> Fallible<Self> {
        if !self.is_nonnegative() {
            return fallible!(NumericDomain, "interval extends below the ln domain");
        }
        Self::new(
            self.lo.ln_round(Direction::Down)?,
            self.hi.ln_round(Direction::Up)?,
        )
    }
}

fn min4<T: Ord>(a: T, b: T, c: T, d: T) -> T {
    a.min(b).min(c.min(d))
}

fn max4<T: Ord>(a: T, b: T, c: T, d: T) -> T {
    a.max(b).max(c.max(d))
}

//! Directed scalar arithmetic for OpenDP conservative numerics.

pub mod backend;
mod scalar;

pub use backend::{Dashu, Direction, SoftFloat};
pub use scalar::{Approximate, BestEffort, Certified, DirectedScalar, DirectedTranscendental, N64};

use crate::error::Fallible;
use std::{
    cmp::Ordering,
    fmt,
    marker::PhantomData,
    ops::{Add, Div, Mul, Neg, Sub},
};

mod interval_private {
    pub trait Sealed {}
}

#[cfg(test)]
mod test;

trait ScalarExt: DirectedScalar {
    fn compare_f64(&self, rhs: f64) -> Fallible<Ordering> {
        self.compare(&Self::exact(rhs)?)
    }

    fn lt_f64(&self, rhs: f64) -> Fallible<bool> {
        Ok(self.compare_f64(rhs)? == Ordering::Less)
    }

    fn max(self, rhs: Self) -> Fallible<Self> {
        Ok(if self.compare(&rhs)? == Ordering::Less {
            rhs
        } else {
            self
        })
    }

    fn min(self, rhs: Self) -> Fallible<Self> {
        Ok(if self.compare(&rhs)? == Ordering::Greater {
            rhs
        } else {
            self
        })
    }
}

impl<T: DirectedScalar> ScalarExt for T {}

/// Marker selecting an interval scalar implementation.
///
/// Certified native scalars intentionally provide arithmetic only:
///
/// ```compile_fail
/// use opendp::traits::CInterval;
///
/// fn certified_interval_transcendental_is_unavailable() -> Result<(), opendp::error::Error> {
///     CInterval::point(1.0)?.exp()?;
///     Ok(())
/// }
/// ```
pub trait IntervalBackend: interval_private::Sealed + Clone + Copy + 'static {
    type Scalar: DirectedScalar;
}

#[derive(Clone, Copy, Debug)]
pub enum A {}
#[derive(Clone, Copy, Debug)]
pub enum B {}
#[derive(Clone, Copy, Debug)]
pub enum C {}
#[derive(Clone, Copy, Debug)]
pub struct S<P>(PhantomData<fn() -> P>);

impl interval_private::Sealed for A {}
impl interval_private::Sealed for B {}
impl interval_private::Sealed for C {}
impl interval_private::Sealed for S<Dashu> {}

impl IntervalBackend for A {
    type Scalar = N64<Approximate>;
}

impl IntervalBackend for B {
    type Scalar = N64<BestEffort>;
}

impl IntervalBackend for C {
    type Scalar = N64<Certified>;
}

impl IntervalBackend for S<Dashu> {
    type Scalar = SoftFloat<Dashu>;
}

pub type AInterval = Interval<A>;
pub type BInterval = Interval<B>;
pub type CInterval = Interval<C>;
pub type SInterval<P> = Interval<S<P>>;

/// Closed endpoint interval `[lo, hi]`.
#[derive(Clone)]
pub struct Interval<Bk: IntervalBackend> {
    lo: Bk::Scalar,
    hi: Bk::Scalar,
}

impl<Bk: IntervalBackend> fmt::Debug for Interval<Bk> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interval")
            .field("lo", &self.lower_f64())
            .field("hi", &self.upper_f64())
            .finish()
    }
}

impl<Bk: IntervalBackend> Interval<Bk> {
    pub fn new(lo: Bk::Scalar, hi: Bk::Scalar) -> Fallible<Self> {
        if lo.compare(&hi)? == Ordering::Greater {
            return fallible!(
                FailedFunction,
                "interval lower endpoint exceeds upper endpoint"
            );
        }
        Ok(Self { lo, hi })
    }

    pub fn point(value: f64) -> Fallible<Self> {
        let value = Bk::Scalar::exact(value)?;
        Self::new(value.clone(), value)
    }

    pub fn from_approx(value: f64) -> Fallible<Self> {
        Self::new(
            Bk::Scalar::approx(value, Direction::Down)?,
            Bk::Scalar::approx(value, Direction::Up)?,
        )
    }

    pub fn between(lo: f64, hi: f64) -> Fallible<Self> {
        Self::new(Bk::Scalar::exact(lo)?, Bk::Scalar::exact(hi)?)
    }

    pub fn between_approx(lo: f64, hi: f64) -> Fallible<Self> {
        Self::new(
            Bk::Scalar::approx(lo, Direction::Down)?,
            Bk::Scalar::approx(hi, Direction::Up)?,
        )
    }

    pub fn lower(&self) -> &Bk::Scalar {
        &self.lo
    }

    pub fn upper(&self) -> &Bk::Scalar {
        &self.hi
    }

    pub fn into_endpoints(self) -> (Bk::Scalar, Bk::Scalar) {
        (self.lo, self.hi)
    }

    pub fn lower_f64(&self) -> Fallible<f64> {
        self.lo.to_f64(Direction::Down)
    }

    pub fn upper_f64(&self) -> Fallible<f64> {
        self.hi.to_f64(Direction::Up)
    }

    pub fn contains_zero(&self) -> Fallible<bool> {
        Ok(self.lo.compare_f64(0.0)? != Ordering::Greater
            && self.hi.compare_f64(0.0)? != Ordering::Less)
    }

    pub fn is_nonnegative(&self) -> Fallible<bool> {
        Ok(self.lo.compare_f64(0.0)? != Ordering::Less)
    }

    pub fn is_nonpositive(&self) -> Fallible<bool> {
        Ok(self.hi.compare_f64(0.0)? != Ordering::Greater)
    }

    pub fn recip(self) -> Fallible<Self> {
        if self.contains_zero()? {
            return fallible!(NumericIndeterminate, "reciprocal interval contains zero");
        }
        let one = Bk::Scalar::exact(1.0)?;
        Self::new(
            one.clone().div_round(self.hi, Direction::Down)?,
            one.div_round(self.lo, Direction::Up)?,
        )
    }

    pub fn abs(self) -> Fallible<Self> {
        if self.is_nonnegative()? {
            return Ok(self);
        }
        if self.is_nonpositive()? {
            return Self::new(self.hi.neg()?, self.lo.neg()?);
        }
        let hi = self.lo.abs()?.max(self.hi.abs()?)?;
        Self::new(Bk::Scalar::exact(0.0)?, hi)
    }

    pub fn abs_upper(self) -> Fallible<Bk::Scalar> {
        if self.is_nonnegative()? {
            return Ok(self.hi);
        }
        if self.is_nonpositive()? {
            return self.lo.neg();
        }
        self.lo.abs()?.max(self.hi.abs()?)
    }

    pub fn max(self, rhs: Self) -> Fallible<Self> {
        Self::new(self.lo.max(rhs.lo)?, self.hi.max(rhs.hi)?)
    }

    pub fn min(self, rhs: Self) -> Fallible<Self> {
        Self::new(self.lo.min(rhs.lo)?, self.hi.min(rhs.hi)?)
    }

    pub fn clamp(self, min: f64, max: f64) -> Fallible<Self> {
        if min > max {
            return fallible!(FailedFunction, "clamp minimum exceeds maximum");
        }
        self.max(Self::point(min)?)?.min(Self::point(max)?)
    }

    pub fn clamp01(self) -> Fallible<Self> {
        self.clamp(0.0, 1.0)
    }
}

impl<Bk: IntervalBackend> Add for Interval<Bk> {
    type Output = Fallible<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.lo.add_round(rhs.lo, Direction::Down)?,
            self.hi.add_round(rhs.hi, Direction::Up)?,
        )
    }
}

impl<Bk: IntervalBackend> Sub for Interval<Bk> {
    type Output = Fallible<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.lo.sub_round(rhs.hi, Direction::Down)?,
            self.hi.sub_round(rhs.lo, Direction::Up)?,
        )
    }
}

impl<Bk: IntervalBackend> Neg for Interval<Bk> {
    type Output = Fallible<Self>;

    fn neg(self) -> Self::Output {
        Self::new(self.hi.neg()?, self.lo.neg()?)
    }
}

impl<Bk: IntervalBackend> Mul for Interval<Bk> {
    type Output = Fallible<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs_nonnegative = self.is_nonnegative()?;
        let lhs_nonpositive = self.is_nonpositive()?;
        let rhs_nonnegative = rhs.is_nonnegative()?;
        let rhs_nonpositive = rhs.is_nonpositive()?;

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
        )?;
        let hi = max4(
            self.lo.clone().mul_round(rhs.lo.clone(), Direction::Up)?,
            self.lo.mul_round(rhs.hi.clone(), Direction::Up)?,
            self.hi.clone().mul_round(rhs.lo, Direction::Up)?,
            self.hi.mul_round(rhs.hi, Direction::Up)?,
        )?;
        Self::new(lo, hi)
    }
}

impl<Bk: IntervalBackend> Div for Interval<Bk> {
    type Output = Fallible<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.recip()?
    }
}

impl<Bk> Interval<Bk>
where
    Bk: IntervalBackend,
    Bk::Scalar: DirectedTranscendental,
{
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
        if self.lo.lt_f64(0.0)? {
            return fallible!(NumericDomain, "interval extends below the ln domain");
        }
        Self::new(
            self.lo.ln_round(Direction::Down)?,
            self.hi.ln_round(Direction::Up)?,
        )
    }

    pub fn sqrt(self) -> Fallible<Self> {
        if self.lo.lt_f64(0.0)? {
            return fallible!(NumericDomain, "interval extends below the sqrt domain");
        }
        Self::new(
            self.lo.sqrt_round(Direction::Down)?,
            self.hi.sqrt_round(Direction::Up)?,
        )
    }
}

fn min4<T: ScalarExt>(a: T, b: T, c: T, d: T) -> Fallible<T> {
    a.min(b)?.min(c.min(d)?)
}

fn max4<T: ScalarExt>(a: T, b: T, c: T, d: T) -> Fallible<T> {
    a.max(b)?.max(c.max(d)?)
}

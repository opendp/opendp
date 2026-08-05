//! Provider-neutral arbitrary-precision floating-point values.
//!
//! `SoftFloat<B>` is an opaque OpenDP-owned scalar. `B` selects a provider,
//! while provider types remain private to this module.

use crate::error::Fallible;
use core::{cmp::Ordering, fmt};

mod dashu;
pub use dashu::Dashu;

/// Directed rounding requested from a numerical operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Down,
    Up,
}

mod private {
    pub trait Sealed {}
}

/// OpenDP-owned adapter for a numerical provider.
///
/// The provider representation is consumed by arithmetic operations so an
/// implementation can reuse allocations instead of cloning by default.
pub trait Backend: private::Sealed + Clone + Copy + fmt::Debug + 'static {
    type Repr: Clone + fmt::Debug;

    fn is_nan(value: &Self::Repr) -> bool;
    fn from_f64(value: f64) -> Fallible<Self::Repr>;
    fn to_f64(value: &Self::Repr, direction: Direction) -> Fallible<f64>;
    fn compare(lhs: &Self::Repr, rhs: &Self::Repr) -> Fallible<Ordering>;
    fn is_zero(value: &Self::Repr) -> bool;
    fn is_infinite(value: &Self::Repr) -> bool;

    fn add(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
    fn sub(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
    fn mul(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
    fn div(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
    fn neg(value: Self::Repr) -> Fallible<Self::Repr>;
    fn abs(value: Self::Repr) -> Fallible<Self::Repr>;
    fn exp(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
    fn exp_m1(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
    fn ln(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
    fn sqrt(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr>;
}

/// Backend-parameterized software floating-point value.
///
/// Successful construction guarantees that the wrapped provider value is not
/// NaN. Native provider infinities remain values rather than errors.
#[derive(Clone)]
pub struct SoftFloat<B: Backend> {
    repr: B::Repr,
}

impl<B: Backend> fmt::Debug for SoftFloat<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SoftFloat").field(&self.repr).finish()
    }
}

impl<B: Backend> SoftFloat<B> {
    fn from_repr(repr: B::Repr) -> Fallible<Self> {
        if B::is_nan(&repr) {
            return fallible!(NumericBackend, "numerical backend produced NaN");
        }
        Ok(Self { repr })
    }

    pub fn from_f64(value: f64) -> Fallible<Self> {
        Self::from_repr(B::from_f64(value)?)
    }

    pub fn to_f64(&self, direction: Direction) -> Fallible<f64> {
        B::to_f64(&self.repr, direction)
    }

    pub fn compare(&self, rhs: &Self) -> Fallible<Ordering> {
        B::compare(&self.repr, &rhs.repr)
    }

    pub fn is_zero(&self) -> bool {
        B::is_zero(&self.repr)
    }

    pub fn is_infinite(&self) -> bool {
        B::is_infinite(&self.repr)
    }

    pub fn add(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::add(self.repr, rhs.repr, direction)?)
    }

    pub fn sub(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::sub(self.repr, rhs.repr, direction)?)
    }

    pub fn mul(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::mul(self.repr, rhs.repr, direction)?)
    }

    pub fn div(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::div(self.repr, rhs.repr, direction)?)
    }

    pub fn neg(self) -> Fallible<Self> {
        Self::from_repr(B::neg(self.repr)?)
    }

    pub fn abs(self) -> Fallible<Self> {
        Self::from_repr(B::abs(self.repr)?)
    }

    pub fn exp(self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::exp(self.repr, direction)?)
    }

    pub fn exp_m1(self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::exp_m1(self.repr, direction)?)
    }

    pub fn ln(self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::ln(self.repr, direction)?)
    }

    pub fn sqrt(self, direction: Direction) -> Fallible<Self> {
        Self::from_repr(B::sqrt(self.repr, direction)?)
    }
}

//! Directed scalar arithmetic for OpenDP conservative numerics.

use crate::error::Fallible;
use std::{cmp::Ordering, fmt, marker::PhantomData};

use super::backend::{Dashu, Direction, SoftFloat};

#[cfg(test)]
mod test;

mod private {
    pub trait Sealed {}
}

/// Approximate native arithmetic, with no directed guarantee.
#[derive(Clone, Copy, Debug)]
pub enum Approximate {}

/// Best-effort directed native arithmetic.
#[derive(Clone, Copy, Debug)]
pub enum BestEffort {}

/// Certified native arithmetic for simple f64 operations.
#[derive(Clone, Copy, Debug)]
pub enum Certified {}

impl private::Sealed for Approximate {}
impl private::Sealed for BestEffort {}
impl private::Sealed for Certified {}
impl private::Sealed for SoftFloat<Dashu> {}

/// Native arithmetic regime.
///
/// Best-effort and certified basic arithmetic share the same outward widening;
/// their important distinction is that only best-effort arithmetic implements
/// [`DirectedTranscendental`].
pub trait NativeRegime: private::Sealed + Clone + Copy + 'static {
    const OUTWARD: bool;
}

impl NativeRegime for Approximate {
    const OUTWARD: bool = false;
}

impl NativeRegime for BestEffort {
    const OUTWARD: bool = true;
}

impl NativeRegime for Certified {
    const OUTWARD: bool = true;
}

/// Native f64 scalar parameterized only by its guarantee regime.
#[derive(Clone, Copy)]
pub struct N64<R = Certified> {
    value: f64,
    marker: PhantomData<fn() -> R>,
}

impl<R: NativeRegime> private::Sealed for N64<R> {}

impl<R> fmt::Debug for N64<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("N64").field(&self.value).finish()
    }
}

impl<R> N64<R> {
    fn raw(value: f64) -> Fallible<Self> {
        if value.is_nan() {
            return fallible!(NumericIndeterminate, "native operation produced NaN");
        }
        Ok(Self {
            value,
            marker: PhantomData,
        })
    }
}

impl<R: NativeRegime> N64<R> {
    /// Construct from the native approximation to a mathematically finite
    /// result. `value` may be infinite only because the finite result overflowed
    /// the f64 range.
    fn from_native_result(value: f64, direction: Direction) -> Fallible<Self> {
        if !R::OUTWARD {
            return Self::raw(value);
        }

        let value = match (value, direction) {
            (f64::INFINITY, Direction::Down) => f64::MAX,
            (f64::INFINITY, Direction::Up) => f64::INFINITY,
            (f64::NEG_INFINITY, Direction::Down) => f64::NEG_INFINITY,
            (f64::NEG_INFINITY, Direction::Up) => -f64::MAX,
            (value, Direction::Down) => value.next_down(),
            (value, Direction::Up) => value.next_up(),
        };
        Self::raw(value)
    }
}

/// Scalar operations with runtime-directed rounding.
pub trait DirectedScalar: private::Sealed + Sized + Clone {
    fn exact(value: f64) -> Fallible<Self>;
    fn approx(value: f64, direction: Direction) -> Fallible<Self>;
    fn to_f64(&self, direction: Direction) -> Fallible<f64>;
    fn compare(&self, rhs: &Self) -> Fallible<Ordering>;

    fn add_round(self, rhs: Self, direction: Direction) -> Fallible<Self>;
    fn sub_round(self, rhs: Self, direction: Direction) -> Fallible<Self>;
    fn mul_round(self, rhs: Self, direction: Direction) -> Fallible<Self>;
    fn div_round(self, rhs: Self, direction: Direction) -> Fallible<Self>;
    fn neg(self) -> Fallible<Self>;
    fn abs(self) -> Fallible<Self>;
}

/// Increasing elementary functions supported by a scalar.
///
/// `N64<Certified>` intentionally does not implement this trait because
/// native transcendental functions do not provide certified rounding.
///
/// ```compile_fail
/// use opendp::traits::{Certified, Direction, DirectedScalar, DirectedTranscendental, N64};
///
/// fn certified_transcendental_is_unavailable() -> Result<(), opendp::error::Error> {
///     let value = N64::<Certified>::exact(1.0)?;
///     value.exp_round(Direction::Up)?;
///     Ok(())
/// }
/// ```
pub trait DirectedTranscendental: DirectedScalar {
    fn exp_round(self, direction: Direction) -> Fallible<Self>;
    fn exp_m1_round(self, direction: Direction) -> Fallible<Self>;
    fn ln_round(self, direction: Direction) -> Fallible<Self>;
    fn sqrt_round(self, direction: Direction) -> Fallible<Self>;
}

impl<R: NativeRegime> DirectedScalar for N64<R> {
    fn exact(value: f64) -> Fallible<Self> {
        Self::raw(value)
    }

    fn approx(value: f64, direction: Direction) -> Fallible<Self> {
        if value.is_finite() {
            Self::from_native_result(value, direction)
        } else {
            // An approximate extended-real input is already a genuine infinity
            // (or an invalid NaN), not finite overflow from an operation.
            Self::raw(value)
        }
    }

    fn to_f64(&self, _: Direction) -> Fallible<f64> {
        Ok(self.value)
    }

    fn compare(&self, rhs: &Self) -> Fallible<Ordering> {
        self.value
            .partial_cmp(&rhs.value)
            .ok_or_else(|| err!(NumericBackend, "native comparison was unordered"))
    }

    fn add_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        let finite = self.value.is_finite() && rhs.value.is_finite();
        let value = self.value + rhs.value;
        if finite {
            Self::from_native_result(value, direction)
        } else {
            // Genuine infinities remain values; `raw` rejects indeterminate NaN.
            Self::raw(value)
        }
    }

    fn sub_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        let finite = self.value.is_finite() && rhs.value.is_finite();
        let value = self.value - rhs.value;
        if finite {
            Self::from_native_result(value, direction)
        } else {
            Self::raw(value)
        }
    }

    fn mul_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        let finite = self.value.is_finite() && rhs.value.is_finite();
        let value = self.value * rhs.value;
        if finite {
            Self::from_native_result(value, direction)
        } else {
            Self::raw(value)
        }
    }

    fn div_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if rhs.value == 0.0 {
            return fallible!(NumericIndeterminate, "division by zero is indeterminate");
        }
        let finite = self.value.is_finite() && rhs.value.is_finite();
        let value = self.value / rhs.value;
        if finite {
            Self::from_native_result(value, direction)
        } else {
            Self::raw(value)
        }
    }

    fn neg(self) -> Fallible<Self> {
        Self::raw(-self.value)
    }

    fn abs(self) -> Fallible<Self> {
        Self::raw(self.value.abs())
    }
}

trait NativeTranscendental: NativeRegime {}

impl NativeTranscendental for Approximate {}
impl NativeTranscendental for BestEffort {}

impl<R: NativeTranscendental> DirectedTranscendental for N64<R> {
    fn exp_round(self, direction: Direction) -> Fallible<Self> {
        if self.value == f64::NEG_INFINITY {
            return Self::raw(0.0);
        }
        if self.value == f64::INFINITY {
            return Self::raw(f64::INFINITY);
        }
        if self.value == 0.0 {
            return Self::raw(1.0);
        }
        Self::from_native_result(self.value.exp(), direction)
    }

    fn exp_m1_round(self, direction: Direction) -> Fallible<Self> {
        if self.value == f64::NEG_INFINITY {
            return Self::raw(-1.0);
        }
        if self.value == f64::INFINITY {
            return Self::raw(f64::INFINITY);
        }
        if self.value == 0.0 {
            return Self::raw(0.0);
        }
        Self::from_native_result(self.value.exp_m1(), direction)
    }

    fn ln_round(self, direction: Direction) -> Fallible<Self> {
        if self.value < 0.0 {
            return fallible!(NumericDomain, "ln operand is below zero");
        }
        if self.value == 0.0 {
            return Self::raw(f64::NEG_INFINITY);
        }
        if self.value == 1.0 {
            return Self::raw(0.0);
        }
        if self.value == f64::INFINITY {
            return Self::raw(f64::INFINITY);
        }
        Self::from_native_result(self.value.ln(), direction)
    }

    fn sqrt_round(self, direction: Direction) -> Fallible<Self> {
        if self.value < 0.0 {
            return fallible!(NumericDomain, "sqrt operand is below zero");
        }
        if self.value == 0.0 {
            return Self::raw(0.0);
        }
        if self.value == 1.0 {
            return Self::raw(1.0);
        }
        if self.value == f64::INFINITY {
            return Self::raw(f64::INFINITY);
        }
        Self::from_native_result(self.value.sqrt(), direction)
    }
}

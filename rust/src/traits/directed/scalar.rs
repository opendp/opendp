//! Directed scalar arithmetic for OpenDP conservative numerics.

use crate::error::Fallible;
use std::{cmp::Ordering, fmt, marker::PhantomData};

use super::backend::{Backend, Direction, SoftFloat};

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

/// Rounding policy for native f64 scalar operations.
pub trait NativeRegime: private::Sealed + Clone + Copy + 'static {
    fn round_simple(value: f64, direction: Direction) -> Fallible<N64<Self>>
    where
        Self: Sized;

    fn round_finite(value: f64, direction: Direction) -> Fallible<N64<Self>>
    where
        Self: Sized,
    {
        Self::round_simple(value, direction)
    }
}

impl NativeRegime for Approximate {
    fn round_simple(value: f64, _: Direction) -> Fallible<N64<Self>> {
        N64::raw(value)
    }

    fn round_finite(value: f64, direction: Direction) -> Fallible<N64<Self>> {
        if value.is_finite() {
            N64::raw(value)
        } else {
            N64::next_finite(value, direction)
        }
    }
}

impl NativeRegime for BestEffort {
    fn round_simple(value: f64, direction: Direction) -> Fallible<N64<Self>> {
        N64::next(value, direction)
    }

    fn round_finite(value: f64, direction: Direction) -> Fallible<N64<Self>> {
        N64::next_finite(value, direction)
    }
}

impl NativeRegime for Certified {
    fn round_simple(value: f64, direction: Direction) -> Fallible<N64<Self>> {
        N64::next(value, direction)
    }

    fn round_finite(value: f64, direction: Direction) -> Fallible<N64<Self>> {
        N64::next_finite(value, direction)
    }
}

/// Native f64 scalar parameterized only by its guarantee regime.
#[derive(Clone, Copy)]
pub struct N64<R = Certified> {
    value: f64,
    marker: PhantomData<fn() -> R>,
}

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

    fn next(value: f64, direction: Direction) -> Fallible<Self> {
        if !value.is_finite() {
            return Self::raw(value);
        }
        Self::raw(match direction {
            Direction::Down => value.next_down(),
            Direction::Up => value.next_up(),
        })
    }

    fn next_finite(value: f64, direction: Direction) -> Fallible<Self> {
        if value.is_nan() {
            return Self::raw(value);
        }
        let value = match (value, direction) {
            (f64::INFINITY, Direction::Down) => f64::MAX,
            (f64::INFINITY, Direction::Up) => f64::INFINITY,
            (f64::NEG_INFINITY, Direction::Down) => f64::NEG_INFINITY,
            (f64::NEG_INFINITY, Direction::Up) => -f64::MAX,
            (value, direction) => match direction {
                Direction::Down => value.next_down(),
                Direction::Up => value.next_up(),
            },
        };
        Self::raw(value)
    }

    fn round(value: f64, direction: Direction) -> Fallible<Self>
    where
        R: NativeRegime,
    {
        R::round_simple(value, direction)
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
pub trait DirectedTranscendental: DirectedScalar {
    fn exp_round(self, direction: Direction) -> Fallible<Self>;
    fn exp_m1_round(self, direction: Direction) -> Fallible<Self>;
    fn ln_round(self, direction: Direction) -> Fallible<Self>;
    fn sqrt_round(self, direction: Direction) -> Fallible<Self>;
}

impl<R: NativeRegime> private::Sealed for N64<R> {}

impl<R: NativeRegime> DirectedScalar for N64<R> {
    fn exact(value: f64) -> Fallible<Self> {
        Self::raw(value)
    }

    fn approx(value: f64, direction: Direction) -> Fallible<Self> {
        Self::round(value, direction)
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
            R::round_finite(value, direction)
        } else {
            Self::round(value, direction)
        }
    }

    fn sub_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        let finite = self.value.is_finite() && rhs.value.is_finite();
        let value = self.value - rhs.value;
        if finite {
            R::round_finite(value, direction)
        } else {
            Self::round(value, direction)
        }
    }

    fn mul_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        let finite = self.value.is_finite() && rhs.value.is_finite();
        let value = self.value * rhs.value;
        if finite {
            R::round_finite(value, direction)
        } else {
            Self::round(value, direction)
        }
    }

    fn div_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if rhs.value == 0.0 {
            return fallible!(NumericIndeterminate, "division by zero is indeterminate");
        }
        let finite = self.value.is_finite() && rhs.value.is_finite();
        let value = self.value / rhs.value;
        if finite {
            R::round_finite(value, direction)
        } else {
            Self::round(value, direction)
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
        let value = self.value.exp();
        if self.value.is_finite() {
            R::round_finite(value, direction)
        } else {
            Self::round(value, direction)
        }
    }

    fn exp_m1_round(self, direction: Direction) -> Fallible<Self> {
        let value = self.value.exp_m1();
        if self.value.is_finite() {
            R::round_finite(value, direction)
        } else {
            Self::round(value, direction)
        }
    }

    fn ln_round(self, direction: Direction) -> Fallible<Self> {
        if self.value < 0.0 {
            return fallible!(NumericDomain, "ln operand is below zero");
        }
        R::round_simple(self.value.ln(), direction)
    }

    fn sqrt_round(self, direction: Direction) -> Fallible<Self> {
        if self.value < 0.0 {
            return fallible!(NumericDomain, "sqrt operand is below zero");
        }
        R::round_simple(self.value.sqrt(), direction)
    }
}

impl<B: Backend> private::Sealed for SoftFloat<B> {}

impl<B: Backend> DirectedScalar for SoftFloat<B> {
    fn exact(value: f64) -> Fallible<Self> {
        SoftFloat::from_f64(value)
    }

    fn approx(value: f64, _: Direction) -> Fallible<Self> {
        // Every finite f64 is exact in the software scalar.
        SoftFloat::from_f64(value)
    }

    fn to_f64(&self, direction: Direction) -> Fallible<f64> {
        SoftFloat::to_f64(self, direction)
    }

    fn compare(&self, rhs: &Self) -> Fallible<Ordering> {
        SoftFloat::compare(self, rhs)
    }

    fn add_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        SoftFloat::add(self, rhs, direction)
    }

    fn sub_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        SoftFloat::sub(self, rhs, direction)
    }

    fn mul_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        SoftFloat::mul(self, rhs, direction)
    }

    fn div_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        SoftFloat::div(self, rhs, direction)
    }

    fn neg(self) -> Fallible<Self> {
        SoftFloat::neg(self)
    }

    fn abs(self) -> Fallible<Self> {
        SoftFloat::abs(self)
    }
}

impl<B: Backend> DirectedTranscendental for SoftFloat<B> {
    fn exp_round(self, direction: Direction) -> Fallible<Self> {
        SoftFloat::exp(self, direction)
    }

    fn exp_m1_round(self, direction: Direction) -> Fallible<Self> {
        SoftFloat::exp_m1(self, direction)
    }

    fn ln_round(self, direction: Direction) -> Fallible<Self> {
        SoftFloat::ln(self, direction)
    }

    fn sqrt_round(self, direction: Direction) -> Fallible<Self> {
        SoftFloat::sqrt(self, direction)
    }
}

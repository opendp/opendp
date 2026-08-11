use super::{Direction, SoftFloat, SoftFloatBackend};
use crate::{
    error::Fallible,
    traits::{DirectedScalar, DirectedTranscendental},
};
use core::cmp::Ordering;
use rug::{
    Float,
    float::Round,
    ops::{AddAssignRound, DivAssignRound, MulAssignRound},
};

const PRECISION: u32 = 256;
type Scalar = SoftFloat<Rug>;

/// Test-only marker selecting Rug's MPFR-backed scalar implementation.
#[derive(Clone, Copy, Debug)]
pub struct Rug;

impl SoftFloatBackend for Rug {
    type Repr = Float;
}

#[derive(Clone, Copy)]
enum Class {
    NegInfinity,
    Finite,
    PosInfinity,
}

fn round(direction: Direction) -> Round {
    match direction {
        Direction::Down => Round::Down,
        Direction::Up => Round::Up,
    }
}

fn classify(value: &Float) -> Class {
    if !value.is_infinite() {
        return Class::Finite;
    }
    if value.is_sign_negative() {
        Class::NegInfinity
    } else {
        Class::PosInfinity
    }
}

fn infinity(negative: bool) -> Float {
    Float::with_val(
        PRECISION,
        if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        },
    )
}

impl Scalar {
    fn from_f64(value: f64) -> Fallible<Self> {
        if value.is_nan() {
            return fallible!(
                NumericIndeterminate,
                "NaN cannot be represented by SoftFloat"
            );
        }
        Ok(Self {
            // SoftFloat models Dashu, which has no signed zero.
            repr: Float::with_val(PRECISION, if value == 0.0 { 0.0 } else { value }),
        })
    }

    fn from_repr(repr: Float) -> Self {
        Self { repr }
    }

    fn narrow(&self, direction: Direction) -> Fallible<f64> {
        match classify(&self.repr) {
            Class::NegInfinity => return Ok(f64::NEG_INFINITY),
            Class::PosInfinity => return Ok(f64::INFINITY),
            Class::Finite => {}
        }
        let max = Float::with_val(PRECISION, f64::MAX);
        if self.repr > max {
            return match direction {
                Direction::Down => Ok(f64::MAX),
                Direction::Up => fallible!(
                    NumericRangeAbove,
                    "finite software value exceeds the f64 range"
                ),
            };
        }
        if self.repr < -max {
            return match direction {
                Direction::Down => fallible!(
                    NumericRangeBelow,
                    "finite software value is below the f64 range"
                ),
                Direction::Up => Ok(-f64::MAX),
            };
        }
        Ok(self.repr.to_f64_round(round(direction)))
    }
}

impl DirectedScalar for Scalar {
    fn exact(value: f64) -> Fallible<Self> {
        Self::from_f64(value)
    }

    fn approx(value: f64, _: Direction) -> Fallible<Self> {
        Self::from_f64(value)
    }

    fn to_f64(&self, direction: Direction) -> Fallible<f64> {
        self.narrow(direction)
    }

    fn compare(&self, rhs: &Self) -> Fallible<Ordering> {
        self.repr
            .partial_cmp(&rhs.repr)
            .ok_or_else(|| err!(NumericBackend, "Rug comparison was unordered"))
    }

    fn add_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        match (classify(&self.repr), classify(&rhs.repr)) {
            (Class::PosInfinity, Class::NegInfinity) | (Class::NegInfinity, Class::PosInfinity) => {
                fallible!(NumericIndeterminate, "opposite infinities cannot be added")
            }
            (Class::PosInfinity, _) | (_, Class::PosInfinity) => {
                Ok(Self::from_repr(infinity(false)))
            }
            (Class::NegInfinity, _) | (_, Class::NegInfinity) => {
                Ok(Self::from_repr(infinity(true)))
            }
            (Class::Finite, Class::Finite) => {
                let mut repr = self.repr;
                repr.add_assign_round(rhs.repr, round(direction));
                Ok(Self::from_repr(repr))
            }
        }
    }

    fn sub_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        self.add_round(rhs.neg()?, direction)
    }

    fn mul_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if (self.repr.is_zero() && rhs.repr.is_infinite())
            || (self.repr.is_infinite() && rhs.repr.is_zero())
        {
            return fallible!(NumericIndeterminate, "zero times infinity is indeterminate");
        }
        if self.repr.is_infinite() || rhs.repr.is_infinite() {
            return Ok(Self::from_repr(infinity(
                self.repr.is_sign_negative() ^ rhs.repr.is_sign_negative(),
            )));
        }
        let mut repr = self.repr;
        repr.mul_assign_round(rhs.repr, round(direction));
        Ok(Self::from_repr(repr))
    }

    fn div_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if rhs.repr.is_zero() {
            return fallible!(NumericIndeterminate, "division by zero is indeterminate");
        }
        if self.repr.is_infinite() && rhs.repr.is_infinite() {
            return fallible!(
                NumericIndeterminate,
                "infinity divided by infinity is indeterminate"
            );
        }
        if self.repr.is_infinite() {
            return Ok(Self::from_repr(infinity(
                self.repr.is_sign_negative() ^ rhs.repr.is_sign_negative(),
            )));
        }
        if rhs.repr.is_infinite() {
            return Self::from_f64(0.0);
        }
        let mut repr = self.repr;
        repr.div_assign_round(rhs.repr, round(direction));
        Ok(Self::from_repr(repr))
    }

    fn neg(self) -> Fallible<Self> {
        Ok(Self::from_repr(-self.repr))
    }

    fn abs(mut self) -> Fallible<Self> {
        self.repr.abs_mut();
        Ok(self)
    }
}

impl DirectedTranscendental for Scalar {
    fn exp_round(self, direction: Direction) -> Fallible<Self> {
        match classify(&self.repr) {
            Class::NegInfinity => return Self::from_f64(0.0),
            Class::PosInfinity => return Ok(Self::from_repr(infinity(false))),
            Class::Finite => {}
        }
        let mut repr = self.repr;
        repr.exp_round(round(direction));
        Ok(Self::from_repr(repr))
    }

    fn exp_m1_round(self, direction: Direction) -> Fallible<Self> {
        match classify(&self.repr) {
            Class::NegInfinity => return Self::from_f64(-1.0),
            Class::PosInfinity => return Ok(Self::from_repr(infinity(false))),
            Class::Finite => {}
        }
        let mut repr = self.repr;
        repr.exp_m1_round(round(direction));
        Ok(Self::from_repr(repr))
    }

    fn ln_round(self, direction: Direction) -> Fallible<Self> {
        if self.repr < 0 {
            return fallible!(NumericDomain, "ln operand is below zero");
        }
        if self.repr.is_zero() {
            return Ok(Self::from_repr(infinity(true)));
        }
        if self.repr.is_infinite() {
            return Ok(Self::from_repr(infinity(false)));
        }
        let mut repr = self.repr;
        repr.ln_round(round(direction));
        Ok(Self::from_repr(repr))
    }

    fn sqrt_round(self, direction: Direction) -> Fallible<Self> {
        if self.repr < 0 {
            return fallible!(NumericDomain, "sqrt operand is below zero");
        }
        if self.repr.is_infinite() {
            return Ok(Self::from_repr(infinity(false)));
        }
        let mut repr = self.repr;
        repr.sqrt_round(round(direction));
        Ok(Self::from_repr(repr))
    }
}

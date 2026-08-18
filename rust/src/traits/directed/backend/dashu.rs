use super::{Direction, SoftFloat, SoftFloatBackend};
use crate::{
    error::Fallible,
    traits::{DirectedScalar, DirectedTranscendental, ToFloatRounded},
};
use core::cmp::Ordering;
use dashu::{
    base::{Abs, SquareRoot},
    float::{
        FBig as DashuFBig,
        round::{
            Round,
            mode::{Down, Up, Zero},
        },
    },
    rational::RBig,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

type Native = DashuFBig<Zero>;
type Scalar = SoftFloat<Dashu>;

/// OpenDP marker selecting the Dashu software scalar implementation.
#[derive(Clone, Copy, Debug)]
pub struct Dashu;

impl SoftFloatBackend for Dashu {
    type Repr = Native;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Class {
    NegInfinity,
    Finite,
    PosInfinity,
}

fn classify(value: &Native) -> Class {
    if value.repr().is_infinite() {
        if value < &Native::ZERO {
            Class::NegInfinity
        } else {
            Class::PosInfinity
        }
    } else {
        Class::Finite
    }
}

fn is_zero(value: &Native) -> bool {
    value == &Native::ZERO
}

fn is_infinite(value: &Native) -> bool {
    value.repr().is_infinite()
}

fn is_negative(value: &Native) -> bool {
    value < &Native::ZERO
}

fn infinity(negative: bool) -> Native {
    if negative {
        Native::NEG_INFINITY
    } else {
        Native::INFINITY
    }
}

fn neutralize<R: Round>(value: DashuFBig<R>) -> Native {
    value.with_rounding::<Zero>()
}

// Provider exhaustion is not classified as a range error here. Only final
// narrowing knows which f64 boundary, if any, a finite value crossed.
fn backend_call<T>(operation: &str, f: impl FnOnce() -> T) -> Fallible<T> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|_| {
        err!(
            NumericBackend,
            "Dashu panicked while evaluating {operation}"
        )
    })
}

fn conversion_error(error: impl core::fmt::Display) -> crate::error::Error {
    err!(NumericBackend, "Dashu conversion failed: {error}")
}

impl Scalar {
    fn from_repr(repr: Native) -> Self {
        // Dashu FBig has no NaN representation.
        Self { repr }
    }

    fn from_f64(value: f64) -> Fallible<Self> {
        if value.is_nan() {
            return fallible!(
                NumericIndeterminate,
                "NaN cannot be represented by SoftFloat"
            );
        }
        if value == f64::INFINITY {
            return Ok(Self::from_repr(Native::INFINITY));
        }
        if value == f64::NEG_INFINITY {
            return Ok(Self::from_repr(Native::NEG_INFINITY));
        }
        Native::try_from(value)
            .map(|value| neutralize(value.with_precision(256).value()))
            .map(Self::from_repr)
            .map_err(conversion_error)
    }

    fn narrow(&self, direction: Direction) -> Fallible<f64> {
        match classify(&self.repr) {
            Class::NegInfinity => return Ok(f64::NEG_INFINITY),
            Class::PosInfinity => return Ok(f64::INFINITY),
            Class::Finite => {}
        }

        let rounded = match direction {
            Direction::Down => self.repr.clone().with_rounding::<Down>().to_f64_rounded(),
            Direction::Up => self.repr.clone().with_rounding::<Up>().to_f64_rounded(),
        };

        // Dashu can report an inexact subnormal conversion as an unchanged
        // rounded value. Compare against the exact rational at every finite
        // boundary, not just the first two subnormal values.
        let exact = RBig::try_from(self.repr.clone()).map_err(conversion_error)?;
        match rounded {
            f64::INFINITY => match direction {
                Direction::Down => Ok(f64::MAX),
                Direction::Up => fallible!(
                    NumericRangeAbove,
                    "finite software value exceeds the f64 range"
                ),
            },
            f64::NEG_INFINITY => match direction {
                Direction::Down => fallible!(
                    NumericRangeBelow,
                    "finite software value is below the f64 range"
                ),
                Direction::Up => Ok(-f64::MAX),
            },
            value => {
                let rounded_exact = RBig::try_from(value).map_err(conversion_error)?;
                let adjusted = match direction {
                    Direction::Down if rounded_exact > exact => value.next_down(),
                    Direction::Down => {
                        let next = value.next_up();
                        if next.is_finite()
                            && RBig::try_from(next).map_err(conversion_error)? <= exact
                        {
                            next
                        } else {
                            value
                        }
                    }
                    Direction::Up if rounded_exact < exact => value.next_up(),
                    Direction::Up => {
                        let next = value.next_down();
                        if next.is_finite()
                            && exact <= RBig::try_from(next).map_err(conversion_error)?
                        {
                            next
                        } else {
                            value
                        }
                    }
                };
                match adjusted {
                    f64::INFINITY => fallible!(
                        NumericRangeAbove,
                        "finite software value exceeds the f64 range"
                    ),
                    f64::NEG_INFINITY => fallible!(
                        NumericRangeBelow,
                        "finite software value is below the f64 range"
                    ),
                    value if value == 0.0 => Ok(if exact < RBig::ZERO { -0.0 } else { 0.0 }),
                    value => Ok(value),
                }
            }
        }
    }
}

impl DirectedScalar for Scalar {
    fn exact(value: f64) -> Fallible<Self> {
        Self::from_f64(value)
    }

    fn approx(value: f64, _: Direction) -> Fallible<Self> {
        // Every finite f64 is exact in the software scalar.
        Self::from_f64(value)
    }

    fn to_f64(&self, direction: Direction) -> Fallible<f64> {
        self.narrow(direction)
    }

    fn compare(&self, rhs: &Self) -> Fallible<Ordering> {
        self.repr
            .partial_cmp(&rhs.repr)
            .ok_or_else(|| err!(NumericBackend, "Dashu comparison was unordered"))
    }

    fn add_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        let repr = match (classify(&self.repr), classify(&rhs.repr)) {
            (Class::PosInfinity, Class::NegInfinity) | (Class::NegInfinity, Class::PosInfinity) => {
                return fallible!(NumericIndeterminate, "opposite infinities cannot be added");
            }
            (Class::PosInfinity, _) | (_, Class::PosInfinity) => Native::INFINITY,
            (Class::NegInfinity, _) | (_, Class::NegInfinity) => Native::NEG_INFINITY,
            (Class::Finite, Class::Finite) => backend_call("addition", || match direction {
                Direction::Down => {
                    neutralize(self.repr.with_rounding::<Down>() + rhs.repr.with_rounding::<Down>())
                }
                Direction::Up => {
                    neutralize(self.repr.with_rounding::<Up>() + rhs.repr.with_rounding::<Up>())
                }
            })?,
        };
        Ok(Self::from_repr(repr))
    }

    fn sub_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        self.add_round(rhs.neg()?, direction)
    }

    fn mul_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if (is_zero(&self.repr) && is_infinite(&rhs.repr))
            || (is_infinite(&self.repr) && is_zero(&rhs.repr))
        {
            return fallible!(NumericIndeterminate, "zero times infinity is indeterminate");
        }
        let repr = if is_infinite(&self.repr) || is_infinite(&rhs.repr) {
            infinity(is_negative(&self.repr) ^ is_negative(&rhs.repr))
        } else {
            backend_call("multiplication", || match direction {
                Direction::Down => {
                    neutralize(self.repr.with_rounding::<Down>() * rhs.repr.with_rounding::<Down>())
                }
                Direction::Up => {
                    neutralize(self.repr.with_rounding::<Up>() * rhs.repr.with_rounding::<Up>())
                }
            })?
        };
        Ok(Self::from_repr(repr))
    }

    fn div_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if is_zero(&rhs.repr) {
            return fallible!(NumericIndeterminate, "division by zero is indeterminate");
        }
        if is_infinite(&self.repr) && is_infinite(&rhs.repr) {
            return fallible!(
                NumericIndeterminate,
                "infinity divided by infinity is indeterminate"
            );
        }
        if is_infinite(&self.repr) {
            return Ok(Self::from_repr(infinity(
                is_negative(&self.repr) ^ is_negative(&rhs.repr),
            )));
        }
        if is_infinite(&rhs.repr) {
            return Self::from_f64(0.0);
        }
        let repr = backend_call("division", || match direction {
            Direction::Down => {
                neutralize(self.repr.with_rounding::<Down>() / rhs.repr.with_rounding::<Down>())
            }
            Direction::Up => {
                neutralize(self.repr.with_rounding::<Up>() / rhs.repr.with_rounding::<Up>())
            }
        })?;
        Ok(Self::from_repr(repr))
    }

    fn neg(self) -> Fallible<Self> {
        backend_call("negation", || neutralize(-self.repr)).map(Self::from_repr)
    }

    fn abs(self) -> Fallible<Self> {
        backend_call("absolute value", || neutralize(self.repr.abs())).map(Self::from_repr)
    }
}

impl DirectedTranscendental for Scalar {
    fn exp_round(self, direction: Direction) -> Fallible<Self> {
        match classify(&self.repr) {
            Class::NegInfinity => return Self::from_f64(0.0),
            Class::PosInfinity => return Ok(Self::from_repr(Native::INFINITY)),
            Class::Finite => {}
        }

        // Dashu cannot reduce an input whose exponent is beyond its internal
        // bounds. Return a conservative extended-real enclosure instead.
        let input = self.repr.clone().with_rounding::<Up>().to_f64_rounded();
        if input < -(i32::MAX as f64) {
            return Self::from_f64(match direction {
                Direction::Down => 0.0,
                Direction::Up => f64::from_bits(1),
            });
        }
        if input > i32::MAX as f64 {
            return match direction {
                Direction::Down => Self::from_f64(f64::MAX),
                Direction::Up => Ok(Self::from_repr(Native::INFINITY)),
            };
        }

        // Transcendentals need more working precision than simple arithmetic.
        let precision = self.repr.precision().max(256);
        let value = self.repr.with_precision(precision).value();
        let repr = backend_call("exp", || match direction {
            Direction::Down => neutralize(value.with_rounding::<Down>().exp()),
            Direction::Up => neutralize(value.with_rounding::<Up>().exp()),
        })?;
        Ok(Self::from_repr(repr))
    }

    fn exp_m1_round(self, direction: Direction) -> Fallible<Self> {
        match classify(&self.repr) {
            Class::NegInfinity => return Self::from_f64(-1.0),
            Class::PosInfinity => return Ok(Self::from_repr(Native::INFINITY)),
            Class::Finite => {}
        }
        let input = self.repr.clone().with_rounding::<Up>().to_f64_rounded();
        if input < -(i32::MAX as f64) {
            return Self::from_f64(match direction {
                Direction::Down => -1.0,
                Direction::Up => (-1.0f64).next_up(),
            });
        }
        if input > i32::MAX as f64 {
            return match direction {
                Direction::Down => Self::from_f64(f64::MAX),
                Direction::Up => Ok(Self::from_repr(Native::INFINITY)),
            };
        }
        let precision = self.repr.precision().max(256);
        let value = self.repr.with_precision(precision).value();
        let repr = backend_call("expm1", || match direction {
            Direction::Down => neutralize(value.with_rounding::<Down>().exp_m1()),
            Direction::Up => neutralize(value.with_rounding::<Up>().exp_m1()),
        })?;
        Ok(Self::from_repr(repr))
    }

    fn ln_round(self, direction: Direction) -> Fallible<Self> {
        if self.repr < Native::ZERO {
            return fallible!(NumericDomain, "ln operand is below zero");
        }
        if is_zero(&self.repr) {
            return Ok(Self::from_repr(Native::NEG_INFINITY));
        }
        if is_infinite(&self.repr) {
            return Ok(Self::from_repr(Native::INFINITY));
        }
        let repr = backend_call("ln", || match direction {
            Direction::Down => neutralize(self.repr.with_rounding::<Down>().ln()),
            Direction::Up => neutralize(self.repr.with_rounding::<Up>().ln()),
        })?;
        Ok(Self::from_repr(repr))
    }

    fn sqrt_round(self, direction: Direction) -> Fallible<Self> {
        if self.repr < Native::ZERO {
            return fallible!(NumericDomain, "sqrt operand is below zero");
        }
        if is_infinite(&self.repr) {
            return Ok(Self::from_repr(Native::INFINITY));
        }
        let repr = backend_call("sqrt", || match direction {
            Direction::Down => neutralize(self.repr.with_rounding::<Down>().sqrt()),
            Direction::Up => neutralize(self.repr.with_rounding::<Up>().sqrt()),
        })?;
        Ok(Self::from_repr(repr))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unexpected_provider_panic_is_numeric_backend() {
        let error =
            backend_call::<()>("test", || panic!("unexpected provider failure")).unwrap_err();
        assert_eq!(error.variant, crate::error::ErrorVariant::NumericBackend);
    }
}

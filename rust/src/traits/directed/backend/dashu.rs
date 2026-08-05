use super::{Backend, Direction, private::Sealed};
use crate::{error::Fallible, traits::ToFloatRounded};
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

/// OpenDP marker selecting the Dashu implementation.
#[derive(Clone, Copy, Debug)]
pub struct Dashu;

impl Sealed for Dashu {}

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

impl Backend for Dashu {
    type Repr = Native;

    fn is_nan(_: &Self::Repr) -> bool {
        // Dashu FBig has no NaN representation.
        false
    }

    fn from_f64(value: f64) -> Fallible<Self::Repr> {
        if value.is_nan() {
            return fallible!(
                NumericIndeterminate,
                "NaN cannot be represented by SoftFloat"
            );
        }
        if value == f64::INFINITY {
            return Ok(Native::INFINITY);
        }
        if value == f64::NEG_INFINITY {
            return Ok(Native::NEG_INFINITY);
        }
        Native::try_from(value)
            .map(neutralize)
            .map_err(conversion_error)
    }

    fn to_f64(value: &Self::Repr, direction: Direction) -> Fallible<f64> {
        match classify(value) {
            Class::NegInfinity => return Ok(f64::NEG_INFINITY),
            Class::PosInfinity => return Ok(f64::INFINITY),
            Class::Finite => {}
        }

        let rounded = match direction {
            Direction::Down => value.clone().with_rounding::<Down>().to_f64_rounded(),
            Direction::Up => value.clone().with_rounding::<Up>().to_f64_rounded(),
        };

        // Dashu reports some subnormal conversions as an inexact no-op.
        // Compare exact rational values at the first two subnormal boundaries.
        let minimum = f64::from_bits(1);
        let next_minimum = f64::from_bits(2);
        if rounded.abs() <= next_minimum {
            let exact = RBig::try_from(value.clone()).map_err(conversion_error)?;
            let minimum = RBig::try_from(minimum).map_err(conversion_error)?;
            let next_minimum = RBig::try_from(next_minimum).map_err(conversion_error)?;
            let negative_minimum = -minimum.clone();
            let negative_next_minimum = -next_minimum.clone();

            if exact > RBig::ZERO && exact < minimum {
                return Ok(match direction {
                    Direction::Down => 0.0,
                    Direction::Up => f64::from_bits(1),
                });
            }
            if exact > RBig::ZERO && exact <= next_minimum {
                return Ok(match direction {
                    Direction::Down if exact == next_minimum => f64::from_bits(2),
                    Direction::Down => f64::from_bits(1),
                    Direction::Up if exact == minimum => f64::from_bits(1),
                    Direction::Up => f64::from_bits(2),
                });
            }
            if exact < RBig::ZERO && exact > negative_minimum {
                return Ok(match direction {
                    Direction::Down => -f64::from_bits(1),
                    Direction::Up => -0.0,
                });
            }
            if exact < RBig::ZERO && exact >= negative_next_minimum {
                return Ok(match direction {
                    Direction::Down if exact == negative_minimum => -f64::from_bits(1),
                    Direction::Down => -f64::from_bits(2),
                    Direction::Up if exact == negative_next_minimum => -f64::from_bits(2),
                    Direction::Up => -f64::from_bits(1),
                });
            }
        }
        match rounded {
            f64::INFINITY => fallible!(
                NumericRangeAbove,
                "finite software value exceeds the f64 range"
            ),
            f64::NEG_INFINITY => fallible!(
                NumericRangeBelow,
                "finite software value is below the f64 range"
            ),
            value => Ok(value),
        }
    }

    fn compare(lhs: &Self::Repr, rhs: &Self::Repr) -> Fallible<Ordering> {
        lhs.partial_cmp(rhs)
            .ok_or_else(|| err!(NumericBackend, "Dashu comparison was unordered"))
    }

    fn is_zero(value: &Self::Repr) -> bool {
        value == &Native::ZERO
    }

    fn is_infinite(value: &Self::Repr) -> bool {
        value.repr().is_infinite()
    }

    fn add(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        match (classify(&lhs), classify(&rhs)) {
            (Class::PosInfinity, Class::NegInfinity) | (Class::NegInfinity, Class::PosInfinity) => {
                fallible!(NumericIndeterminate, "opposite infinities cannot be added")
            }
            (Class::PosInfinity, _) | (_, Class::PosInfinity) => Ok(Native::INFINITY),
            (Class::NegInfinity, _) | (_, Class::NegInfinity) => Ok(Native::NEG_INFINITY),
            (Class::Finite, Class::Finite) => backend_call("addition", || match direction {
                Direction::Down => {
                    neutralize(lhs.with_rounding::<Down>() + rhs.with_rounding::<Down>())
                }
                Direction::Up => neutralize(lhs.with_rounding::<Up>() + rhs.with_rounding::<Up>()),
            }),
        }
    }

    fn sub(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        Self::add(lhs, Self::neg(rhs)?, direction)
    }

    fn mul(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        if (Self::is_zero(&lhs) && Self::is_infinite(&rhs))
            || (Self::is_infinite(&lhs) && Self::is_zero(&rhs))
        {
            return fallible!(NumericIndeterminate, "zero times infinity is indeterminate");
        }
        if Self::is_infinite(&lhs) || Self::is_infinite(&rhs) {
            return Ok(infinity(is_negative(&lhs) ^ is_negative(&rhs)));
        }
        backend_call("multiplication", || match direction {
            Direction::Down => {
                neutralize(lhs.with_rounding::<Down>() * rhs.with_rounding::<Down>())
            }
            Direction::Up => neutralize(lhs.with_rounding::<Up>() * rhs.with_rounding::<Up>()),
        })
    }

    fn div(lhs: Self::Repr, rhs: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        if Self::is_zero(&rhs) {
            return fallible!(NumericIndeterminate, "division by zero is indeterminate");
        }
        if Self::is_infinite(&lhs) && Self::is_infinite(&rhs) {
            return fallible!(
                NumericIndeterminate,
                "infinity divided by infinity is indeterminate"
            );
        }
        if Self::is_infinite(&lhs) {
            return Ok(infinity(is_negative(&lhs) ^ is_negative(&rhs)));
        }
        if Self::is_infinite(&rhs) {
            return Self::from_f64(0.0);
        }
        backend_call("division", || match direction {
            Direction::Down => {
                neutralize(lhs.with_rounding::<Down>() / rhs.with_rounding::<Down>())
            }
            Direction::Up => neutralize(lhs.with_rounding::<Up>() / rhs.with_rounding::<Up>()),
        })
    }

    fn neg(value: Self::Repr) -> Fallible<Self::Repr> {
        backend_call("negation", || neutralize(-value))
    }

    fn abs(value: Self::Repr) -> Fallible<Self::Repr> {
        backend_call("absolute value", || neutralize(value.abs()))
    }

    fn exp(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        match classify(&value) {
            Class::NegInfinity => return Self::from_f64(0.0),
            Class::PosInfinity => return Ok(Native::INFINITY),
            Class::Finite => {}
        }

        // Transcendentals need more working precision than simple arithmetic.
        let precision = value.precision().max(256);
        let value = value.with_precision(precision).value();
        backend_call("exp", || match direction {
            Direction::Down => neutralize(value.with_rounding::<Down>().exp()),
            Direction::Up => neutralize(value.with_rounding::<Up>().exp()),
        })
    }

    fn exp_m1(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        match classify(&value) {
            Class::NegInfinity => return Self::from_f64(-1.0),
            Class::PosInfinity => return Ok(Native::INFINITY),
            Class::Finite => {}
        }
        backend_call("expm1", || match direction {
            Direction::Down => neutralize(value.with_rounding::<Down>().exp_m1()),
            Direction::Up => neutralize(value.with_rounding::<Up>().exp_m1()),
        })
    }

    fn ln(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        if value < Native::ZERO {
            return fallible!(NumericDomain, "ln operand is below zero");
        }
        if Self::is_zero(&value) {
            return Ok(Native::NEG_INFINITY);
        }
        if Self::is_infinite(&value) {
            return Ok(Native::INFINITY);
        }
        backend_call("ln", || match direction {
            Direction::Down => neutralize(value.with_rounding::<Down>().ln()),
            Direction::Up => neutralize(value.with_rounding::<Up>().ln()),
        })
    }

    fn sqrt(value: Self::Repr, direction: Direction) -> Fallible<Self::Repr> {
        if value < Native::ZERO {
            return fallible!(NumericDomain, "sqrt operand is below zero");
        }
        if Self::is_infinite(&value) {
            return Ok(Native::INFINITY);
        }
        backend_call("sqrt", || match direction {
            Direction::Down => neutralize(value.with_rounding::<Down>().sqrt()),
            Direction::Up => neutralize(value.with_rounding::<Up>().sqrt()),
        })
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

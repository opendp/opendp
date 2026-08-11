use super::{SoftFloat, SoftFloatBackend};
use crate::{
    error::Fallible,
    traits::{DirectedScalar, DirectedTranscendental, Direction},
};
use core::{cmp::Ordering, ops::Neg};
use dashu::{
    base::Abs,
    float::{
        Context, FBig, FpResult,
        round::{
            Round,
            mode::{Down, Up, Zero},
        },
    },
};

const PRECISION: usize = 256;
type Native = FBig<Zero>;
type Scalar = SoftFloat<Dashu>;

/// OpenDP marker selecting the Dashu software scalar implementation.
#[derive(Clone, Copy, Debug)]
pub struct Dashu;

impl SoftFloatBackend for Dashu {
    type Repr = Native;
}

impl PartialEq for Scalar {
    fn eq(&self, rhs: &Self) -> bool {
        self.repr == rhs.repr
    }
}

impl Eq for Scalar {}

impl PartialOrd for Scalar {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for Scalar {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.repr.cmp(&rhs.repr)
    }
}

impl Neg for Scalar {
    type Output = Self;

    fn neg(self) -> Self {
        Self { repr: -self.repr }
    }
}

// Apply the requested rounding to the full operands, without pre-rounding them.
// A fixed positive precision also works for exact constants with precision zero.
macro_rules! directed {
    ($direction:expr, $method:ident($($arg:expr),* $(,)?)) => {
        match $direction {
            Direction::Down => Scalar::from_result(Context::<Down>::new(PRECISION).$method($($arg),*)),
            Direction::Up => Scalar::from_result(Context::<Up>::new(PRECISION).$method($($arg),*)),
        }
    };
}

impl Scalar {
    fn from_result<R: Round>(result: FpResult<FBig<R>>) -> Fallible<Self> {
        // Provider exhaustion is not an f64 range error. Only final narrowing
        // classifies finite results relative to the f64 boundaries.
        let value =
            result.map_err(|error| err!(NumericBackend, "Dashu operation failed: {error}"))?;
        Ok(Self {
            repr: value.value().with_rounding::<Zero>(),
        })
    }
}

impl DirectedScalar for Scalar {
    fn exact(value: f64) -> Fallible<Self> {
        // Binary FBig accepts every f64 except NaN, including signed zeros and infinities.
        Native::try_from(value)
            .map(|repr| Self { repr })
            .map_err(|_| {
                err!(
                    NumericIndeterminate,
                    "NaN cannot be represented by SoftFloat"
                )
            })
    }

    fn to_f64(&self, direction: Direction) -> Fallible<f64> {
        let value = match direction {
            Direction::Down => self.repr.clone().with_rounding::<Down>().to_f64().value(),
            Direction::Up => self.repr.clone().with_rounding::<Up>().to_f64().value(),
        };
        if self.repr.repr().is_infinite() {
            return Ok(value);
        }
        match value {
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

    fn add_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        // Dashu arithmetic rejects infinite inputs; OpenDP uses extended reals.
        if self.repr.repr().is_infinite() {
            if self.repr == -&rhs.repr {
                return fallible!(NumericIndeterminate, "opposite infinities cannot be added");
            }
            return Ok(self);
        }
        if rhs.repr.repr().is_infinite() {
            return Ok(rhs);
        }
        directed!(direction, add(self.repr.repr(), rhs.repr.repr()))
    }

    fn sub_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        self.add_round(-rhs, direction)
    }

    fn mul_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if self.repr.repr().is_infinite() || rhs.repr.repr().is_infinite() {
            if self.repr == Native::ZERO || rhs.repr == Native::ZERO {
                return fallible!(NumericIndeterminate, "zero times infinity is indeterminate");
            }
            let negative = self.repr.repr().sign() != rhs.repr.repr().sign();
            return Self::exact(if negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            });
        }
        directed!(direction, mul(self.repr.repr(), rhs.repr.repr()))
    }

    fn div_round(self, rhs: Self, direction: Direction) -> Fallible<Self> {
        if rhs.repr == Native::ZERO {
            return fallible!(NumericIndeterminate, "division by zero is indeterminate");
        }
        if self.repr.repr().is_infinite() && rhs.repr.repr().is_infinite() {
            return fallible!(
                NumericIndeterminate,
                "infinity divided by infinity is indeterminate"
            );
        }
        let negative = self.repr.repr().sign() != rhs.repr.repr().sign();
        if self.repr.repr().is_infinite() {
            return Self::exact(if negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            });
        }
        if rhs.repr.repr().is_infinite() {
            return Self::exact(if negative { -0.0 } else { 0.0 });
        }
        directed!(direction, div(self.repr.repr(), rhs.repr.repr()))
    }

    fn sqrt_round(self, direction: Direction) -> Fallible<Self> {
        if self.repr < Native::ZERO {
            return fallible!(NumericDomain, "sqrt operand is below zero");
        }
        if self.repr == Native::INFINITY {
            return Ok(self);
        }
        // Dashu 0.6 subtracts up to 2 * precision from the exponent before
        // halving it. Reject this provider edge before that subtraction wraps.
        if self.repr.repr().exponent() < isize::MIN + 2 * PRECISION as isize {
            return fallible!(
                NumericBackend,
                "sqrt exceeds Dashu's working exponent range"
            );
        }
        directed!(direction, sqrt(self.repr.repr()))
    }

    fn abs(self) -> Self {
        Self {
            repr: self.repr.abs(),
        }
    }
}

impl DirectedTranscendental for Scalar {
    fn exp_round(self, direction: Direction) -> Fallible<Self> {
        directed!(direction, exp(self.repr.repr(), None))
    }

    fn exp_m1_round(self, direction: Direction) -> Fallible<Self> {
        directed!(direction, exp_m1(self.repr.repr(), None))
    }

    fn ln_round(self, direction: Direction) -> Fallible<Self> {
        if self.repr < Native::ZERO {
            return fallible!(NumericDomain, "ln operand is below zero");
        }
        if self.repr == Native::INFINITY {
            return Ok(self);
        }
        directed!(direction, ln(self.repr.repr(), None))
    }
}

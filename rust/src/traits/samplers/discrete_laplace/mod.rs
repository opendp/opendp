use num::One;
use rug::Rational;
use std::convert::TryFrom;
use std::ops::MulAssign;

// stands for Big Integer, an integer with unlimited precision, from gmp
use rug::{Complete, Integer as BInt};

use crate::error::Fallible;
use crate::traits::Float;

use super::SampleTwoSidedGeometric;

trait SampleDiscreteLaplace: Sized {
    fn sample_discrete_laplace(shift: Self, scale: Self, gran_pow: u32) -> Fallible<Self>;
}

impl<T> SampleDiscreteLaplace for T
where
    Rational: TryFrom<T>,
    BInt: SampleTwoSidedGeometric<T>,
    T: 'static + From<u32> + Float + MulAssign + CastInternalRational,
{
    fn sample_discrete_laplace(shift: Self, mut scale: Self, gran_pow: u32) -> Fallible<Self> {
        let (mut sx, sy): (BInt, BInt) = shift.into_rational()?.into_numer_denom();

        //     shift + l           where l ~ Lap(scale)
        //          shift = sx/sy = sx'/(gx/gy) -> sx' = sx * g /_r sy
        //
        //  ~= (sx' / g + i) * g  where i ~ 2SGeo(scale / g)

        // rewrite the rationals
        // change shift denominator to gran by multiplying shift top by 2^gran_pow / sy
        sx <<= gran_pow;
        sx += (&sy - 1u8).complete() / 2; // divide by sy with rounding towards nearest
        sx /= sy;

        // increase scale by gran
        scale *= T::exp2(gran_pow.into());

        // noise the shift numerator
        sx = rug::Integer::sample_two_sided_geometric(sx, scale, None)?;

        let rational = Rational::from((sx, BInt::one() << (gran_pow + 1)));
        Ok(Self::from_rational(rational))
    }
}

pub trait CastInternalRational: {
    fn from_rational(v: rug::Rational) -> Self;
    fn into_rational(self) -> Fallible<rug::Rational>;
}

macro_rules! impl_cast_internal_rational {
    ($ty:ty, $method:ident) => (impl CastInternalRational for $ty {
        fn from_rational(v: rug::Rational) -> Self {
            v.$method()
        }
        fn into_rational(self) -> Fallible<rug::Rational> {
            Rational::try_from(self)
                .map_err(|_| err!(FailedFunction, "shift must be finite"))
        }
    })
}
impl_cast_internal_rational!(f32, to_f32);
impl_cast_internal_rational!(f64, to_f64);

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_sample_discrete_laplace() -> Fallible<()> {
        let dgeo: f64 = f64::sample_discrete_laplace(0f64, 1f64, 50)?;
        println!("final: {:?}", dgeo);

        // let dgeo: f64 = f64::sample_discrete_laplace(0f64, 20f64, 14)?;
        // println!("final: {:?}", dgeo);
        Ok(())
    }
}
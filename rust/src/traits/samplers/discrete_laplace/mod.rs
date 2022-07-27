use num::One;
use std::convert::TryFrom;
use std::ops::MulAssign;

// stands for Big Integer, an integer with unlimited precision, from gmp
#[cfg(feature = "use-mpfr")]
use rug::{Complete, Integer, Rational};

use crate::{error::Fallible, traits::ExactIntCast};
use crate::traits::Float;

use super::{SampleTwoSidedGeometric, Tail};

pub trait SampleDiscreteLaplace: Sized {
    fn sample_discrete_laplace(shift: Self, scale: Self, gran_pow: i32) -> Fallible<Self>;
}

#[cfg(feature = "use-mpfr")]
impl<T> SampleDiscreteLaplace for T
where
    Rational: TryFrom<T>,
    Integer: SampleTwoSidedGeometric<T>,
    T: 'static + Float + MulAssign + CastInternalRational + ExactIntCast<i32>,
{
    fn sample_discrete_laplace(shift: Self, mut scale: Self, gran_pow: i32) -> Fallible<Self> {
        let (mut sx, mut sy): (Integer, Integer) = shift.into_rational()?.into_numer_denom();

        //     shift + l           where l ~ Lap(scale)
        //          shift = sx/sy = sx'/(gx/gy) -> sx' = sx * g /_r sy
        //
        //  ~= (sx' / g + i) * g  where i ~ 2SGeo(scale / g)

        // 1. Exactly multiply shift by 2^-gran_pow
        if gran_pow > 0 {
            sy <<= gran_pow;
        } else {
            sx <<= -gran_pow;
        }
        // 2. Rewrite the shift numer (sx) to implicitly have a denom of 2^-gran_pow.
        sx += (&sy - 1u8).complete() / 2; // divide by sy with rounding towards nearest
        sx /= sy;

        // adjust scale by gran
        scale *= T::exp2(T::exact_int_cast(-gran_pow)?);
        
        // noise the shift numerator
        sx = Integer::sample_two_sided_geometric(sx, scale, Tail::Modular)?;

        let mut rx = Integer::one();
        if gran_pow > 0 {
            sx <<= gran_pow;
        } else {
            rx <<= -gran_pow;
        }

        let rational = Rational::from((sx, rx));
        Ok(Self::from_rational(rational))
    }
}

#[cfg(feature = "use-mpfr")]
pub trait CastInternalRational {
    fn from_rational(v: Rational) -> Self;
    fn into_rational(self) -> Fallible<Rational>;
}
#[cfg(feature = "use-mpfr")]
macro_rules! impl_cast_internal_rational {
    ($ty:ty, $method:ident) => {
        impl CastInternalRational for $ty {
            fn from_rational(v: Rational) -> Self {
                v.$method()
            }
            fn into_rational(self) -> Fallible<Rational> {
                Rational::try_from(self).map_err(|_| err!(FailedFunction, "shift must be finite"))
            }
        }
    };
}
#[cfg(feature = "use-mpfr")]
impl_cast_internal_rational!(f32, to_f32);
#[cfg(feature = "use-mpfr")]
impl_cast_internal_rational!(f64, to_f64);

#[cfg(not(feature = "use-mpfr"))]
impl<T> SampleDiscreteLaplace for T
where
    T: num::Float
        + rand::distributions::uniform::SampleUniform
        + crate::traits::samplers::SampleRademacher,
{
    fn sample_discrete_laplace(shift: Self, scale: Self, _gran_pow: usize) -> Fallible<Self> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut u: T = T::zero();
        while u.abs().is_zero() {
            u = rng.gen_range(T::from(-1.).unwrap(), T::from(1.).unwrap())
        }
        let value = shift + u.signum() * u.abs().ln() * scale;
        Ok(super::censor_neg_zero(value))
    }
}

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

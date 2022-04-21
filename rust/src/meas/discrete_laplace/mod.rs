#[cfg(feature = "ffi")]
mod ffi;

use std::ops::Mul;

use num::{Float, Integer};

use crate::core::{Measurement, Metric, SensitivityMetric};
use crate::dist::{AbsoluteDistance, L1Distance, MaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::meas::make_base_geometric;
use crate::samplers::SampleTwoSidedGeometric;
use crate::traits::{
    AlertingAbs, CheckNull, DistanceConstant, InfAdd, InfCast, RoundCast, TotalOrd,
};
use crate::trans::{
    make_lipschitz_cast, make_lipschitz_mul, GreatestDifference, LipschitzCastDomain,
    LipschitzMulDomain, LipschitzMulMetric, SameMetric,
};

use super::GeometricDomain;

// Helper trait to obscure trait bounds on Atom
pub trait BoundedLipschitzMulDomain<I>: LipschitzMulDomain<Atom = Self::BoundedAtom>
where
    I: InfCast<Self::BoundedAtom>,
{
    type BoundedAtom: Float
        + AlertingAbs
        + DistanceConstant<Self::Atom>
        + DistanceConstant<I>
        + DefaultGranularity
        + GreatestDifference<Self::Atom>
        + InfAdd;
}
impl<T, I> BoundedLipschitzMulDomain<I> for T
where
    T: LipschitzMulDomain,
    I: InfCast<Self::Atom>,
    Self::Atom: Float
        + AlertingAbs
        + DistanceConstant<Self::Atom>
        + DistanceConstant<I>
        + DefaultGranularity
        + GreatestDifference<Self::Atom>
        + InfAdd,
{
    type BoundedAtom = Self::Atom;
}

pub trait DiscreteLaplaceDomain<I>:
    'static + BoundedLipschitzMulDomain<I> + LipschitzCastDomain<Self::IntegerDomain> + Default
where
    I: InfCast<Self::BoundedAtom>,
{
    type Metric: SensitivityMetric<Distance = Self::Atom> + Default + LipschitzMulMetric;
    type IntegerMetric: Metric<Distance = I>;
    type IntegerDomain: GeometricDomain<Self::Atom, Atom = I, InputMetric = Self::IntegerMetric>
        + LipschitzCastDomain<Self>
        + Default;
}

impl<T, I> DiscreteLaplaceDomain<I> for AllDomain<T>
where
    T: 'static + Float + CheckNull + for<'a> Mul<&'a T, Output = T> + RoundCast<I>
    + AlertingAbs
    + DistanceConstant<Self::Atom>
    + DistanceConstant<I>
    + DefaultGranularity
    + GreatestDifference<Self::Atom>
    + InfAdd,
    I: 'static
        + RoundCast<T>
        + CheckNull
        + Integer
        + Clone
        + SampleTwoSidedGeometric<T>
        + InfCast<T>,
{
    type Metric = AbsoluteDistance<T>;
    type IntegerMetric = AbsoluteDistance<I>;
    type IntegerDomain = AllDomain<I>;
}

impl<T, I> DiscreteLaplaceDomain<I> for VectorDomain<AllDomain<T>>
where
    T: 'static + Float + CheckNull + for<'a> Mul<&'a T, Output = T> + RoundCast<I>
    + AlertingAbs
    + DistanceConstant<Self::Atom>
    + DistanceConstant<I>
    + DefaultGranularity
    + GreatestDifference<Self::Atom>
    + InfAdd,
    I: 'static
        + RoundCast<T>
        + CheckNull
        + Integer
        + Clone
        + SampleTwoSidedGeometric<T>
        + InfCast<T>,
{
    type Metric = L1Distance<T>;
    type IntegerMetric = L1Distance<I>;
    type IntegerDomain = VectorDomain<AllDomain<I>>;
}

pub trait DefaultGranularity {
    const GRAN: Self;
}
macro_rules! impl_granularity {
    ($ty:ty) => {
        impl DefaultGranularity for $ty {
            // 2^{-14}
            const GRAN: Self = 0.00006103515625;
        }
    };
}
impl_granularity!(f32);
impl_granularity!(f64);

pub fn make_base_discrete_laplace<D, I>(
    scale: D::Atom,
    bounds: Option<(D::Atom, D::Atom)>,
    granularity: Option<D::Atom>,
) -> Fallible<Measurement<D, D, D::Metric, MaxDivergence<D::Atom>>>
where
    D: DiscreteLaplaceDomain<I>,
    I: 'static
        + InfCast<D::Atom>
        + RoundCast<D::Atom>
        + Integer
        + Clone
        + TotalOrd
        + GreatestDifference<D::Atom>
        + InfAdd,
    // metrics match, but associated distance types may vary
    (D::Metric, D::IntegerMetric): SameMetric<D::Metric, D::IntegerMetric>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    // unwrap granularity or fill with default
    let granularity: D::Atom = granularity.unwrap_or_else(|| scale * D::Atom::GRAN);
    // derive the constant to convert to int space
    let c: D::Atom = (-granularity.log2().ceil()).exp2();
    // translate bounds to int space
    let bounds: Option<(I, I)> = bounds
        .map(|(l, u): (_, _)| {
            Result::<_, Error>::Ok((I::round_cast(l * c)?, I::round_cast(u * c)?))
        })
        .transpose()?;

    let scale: D::Atom = scale * c;

    make_lipschitz_mul::<D, D::Metric>(c)?
        >> make_lipschitz_cast::<D, D::IntegerDomain, D::Metric, D::IntegerMetric>()?
        >> make_base_geometric::<D::IntegerDomain, D::Atom>(scale, bounds)?
        // these metrics are ignored. Arbitrarily chosen to minimize the number of necessary trait bounds
        >> make_lipschitz_cast::<D::IntegerDomain, D, AbsoluteDistance<D::Atom>, AbsoluteDistance<D::Atom>>()?
        >> make_lipschitz_mul::<D, AbsoluteDistance<D::Atom>>(c.recip())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dom::VectorDomain, trans::make_sized_bounded_mean};

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (make_sized_bounded_mean(3, (10.0, 12.0))?
            >> make_base_discrete_laplace::<_, i64>(1.0f64, None, None)?)?;
        let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
        Ok(())
    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace::<AllDomain<_>, i64>(1.0, None, None)?;
        let _ret = measurement.invoke(&0.0)?;

        assert!(measurement.check(&1., &1.0001)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace::<VectorDomain<_>, i64>(1.0, None, None)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&1., &1.0001)?);
        Ok(())
    }
}

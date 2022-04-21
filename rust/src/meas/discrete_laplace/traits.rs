use std::ops::Mul;

use num::Float;

use crate::core::{Metric, SensitivityMetric};
use crate::dist::{AbsoluteDistance, L1Distance};
use crate::dom::{AllDomain, VectorDomain};
use crate::samplers::SampleTwoSidedGeometric;
use crate::traits::{AlertingAbs, CheckNull, DistanceConstant, InfAdd, InfCast, RoundCast};
use crate::trans::{
    GreatestDifference, LipschitzCastDomain, LipschitzMulDomain, LipschitzMulMetric,
};

use crate::meas::GeometricDomain;

impl<T, I> DiscreteLaplaceDomain<I> for AllDomain<T>
where
    T: 'static
        + Float
        + CheckNull
        + for<'a> Mul<&'a T, Output = T>
        + RoundCast<I>
        + AlertingAbs
        + DistanceConstant<Self::Atom>
        + DistanceConstant<I>
        + DefaultGranularity
        + GreatestDifference<Self::Atom>
        + InfAdd,
    I: 'static + RoundCast<T> + CheckNull + Clone + SampleTwoSidedGeometric<T> + InfCast<T>,
{
    type Metric = AbsoluteDistance<T>;
    type IntegerMetric = AbsoluteDistance<I>;
    type IntegerDomain = AllDomain<I>;
}

impl<T, I> DiscreteLaplaceDomain<I> for VectorDomain<AllDomain<T>>
where
    T: 'static
        + Float
        + CheckNull
        + for<'a> Mul<&'a T, Output = T>
        + RoundCast<I>
        + AlertingAbs
        + DistanceConstant<Self::Atom>
        + DistanceConstant<I>
        + DefaultGranularity
        + GreatestDifference<Self::Atom>
        + InfAdd,
    I: 'static + RoundCast<T> + CheckNull + Clone + SampleTwoSidedGeometric<T> + InfCast<T>,
{
    type Metric = L1Distance<T>;
    type IntegerMetric = L1Distance<I>;
    type IntegerDomain = VectorDomain<AllDomain<I>>;
}

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

use num::Signed;

use crate::{
    core::{Domain, Function, Metric, StabilityMap, Transformation},
    dist::{AbsoluteDistance, LpDistance},
    dom::{AllDomain, VectorDomain},
    error::Fallible,
    traits::{CheckNull, InfAdd, InfCast, RoundCast},
};


/// Create a data transformation that rounds a float to the nearest integer on an arbitrary input domain.
pub fn make_lipschitz_cast<DI, DO, MI, MO>() -> Fallible<Transformation<DI, DO, MI, MO>>
where
    DI: Domain + Default,
    DO: LipschitzCastDomain<DI>,
    MI: Metric,
    MO: Metric,
    MI::Distance: 'static + Clone,
    MO::Distance: 'static
        + Clone
        + InfCast<MI::Distance>
        + GreatestDifference<MI::Distance>
        + InfAdd
        + PartialOrd,
    (MI, MO): SameMetric<MI, MO>,
{
    Ok(Transformation::new(
        DI::default(),
        DO::default(),
        Function::new(|arg: &DI::Carrier| DO::transform(arg)),
        MI::default(),
        MO::default(),
        StabilityMap::new_fallible(|d_in: &MI::Distance| {
            MO::Distance::inf_cast(d_in.clone())?.inf_add(&MO::Distance::C)
        }),
    ))
}

pub trait LipschitzCastDomain<DI>: Domain + Default
where
    DI: Domain,
{
    fn transform(v: &DI::Carrier) -> Self::Carrier;
}

impl<TI, TO> LipschitzCastDomain<AllDomain<TI>> for AllDomain<TO>
where
    TI: CheckNull + Clone + Signed,
    TO: RoundCast<TI> + CheckNull + FiniteBounds,
{
    fn transform(v: &TI) -> TO {
        TO::round_cast(v.clone())
            .unwrap_or_else(|_| if v.is_negative() {TO::MIN_FINITE} else {TO::MAX_FINITE})
    }
}

impl<DI, DO> LipschitzCastDomain<VectorDomain<DI>> for VectorDomain<DO>
where
    DI: Domain,
    DO: LipschitzCastDomain<DI>,
{
    fn transform(v: &Vec<DI::Carrier>) -> Vec<DO::Carrier> {
        v.iter().map(DO::transform).collect()
    }
}

/// Consts representing the maximum and minimum finite representable values.
pub trait FiniteBounds {
    const MAX_FINITE: Self;
    const MIN_FINITE: Self;
}
macro_rules! impl_finite_bounds {
    ($($ty:ty)+) => ($(impl FiniteBounds for $ty {
        const MAX_FINITE: Self = Self::MAX;
        const MIN_FINITE: Self = Self::MIN;
    })+)
}
impl_finite_bounds!(f64 f32 i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);


/// Allow the associated type to change, but restrict the metric
pub trait SameMetric<MI, MO> {}
impl<MI: Metric, MO: Metric> SameMetric<MI, MO>
    for (
        AbsoluteDistance<MI::Distance>,
        AbsoluteDistance<MO::Distance>,
    )
{
}
impl<MI: Metric, MO: Metric, const P: usize> SameMetric<MI, MO>
    for (LpDistance<MI::Distance, P>, LpDistance<MO::Distance, P>)
{
}

/// A const C = max(dist(cast(v), cast(v'))) - |v - v'| where v and v' are adjacent inputs in TI.
pub trait GreatestDifference<TI> {
    const C: Self;
}

macro_rules! impl_greatest_cast_0 {
    ($tyi:ty, $tyo:ty) => {
        impl GreatestDifference<$tyi> for $tyo {
            const C: Self = 0;
        }
    };
}
macro_rules! impl_greatest_cast_1 {
    ($tyi:ty, $tyo:ty) => {
        impl GreatestDifference<$tyi> for $tyo {
            const C: Self = 1;
        }
    };
}
macro_rules! impl_greatest_cast_inf {
    ($tyi:ty, $tyo:ty) => {
        impl GreatestDifference<$tyi> for $tyo {
            const C: Self = Self::INFINITY;
        }
    };
}

use crate::traits::cartesian;
// the three macros are applied to the upper triangle, diagonal, and lower triangle
// conversion from any int type to any int type not on the diagonal has max distance 1
// conversion from any int type to self 
cartesian! {[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128], impl_greatest_cast_0}
// float to int
cartesian! {
    [f32, f64],
    [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128],
    impl_greatest_cast_1
}
// int to float 
//    essentially unimplemented; the bounds are infinitely loose
//    we can tighten these but we don't currently need them
//    it would be max(ulp(TI::MIN), ulp(TI::MAX))
cartesian! {
    [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128],
    [f32, f64],
    impl_greatest_cast_inf
}
// float to float
impl GreatestDifference<f64> for f32 {
    const C: Self = f32::INFINITY;
}
impl GreatestDifference<f32> for f64 {
    const C: Self = f64::INFINITY;
}
impl GreatestDifference<f32> for f32 {
    const C: Self = 0.;
}
impl GreatestDifference<f64> for f64 {
    const C: Self = 0.;
}



#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_integerize() -> Fallible<()> {
        let integerizer = make_lipschitz_cast::<
            AllDomain<f64>,
            AllDomain<i64>,
            AbsoluteDistance<f64>,
            AbsoluteDistance<i64>,
        >()?;
        assert_eq!(integerizer.invoke(&1.3)?, 1);
        println!("{:?}", integerizer.invoke(&1.3));
        Ok(())
    }
}

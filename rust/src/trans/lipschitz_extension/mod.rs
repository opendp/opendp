use std::ops::Mul;

use crate::{
    core::{Domain, Function, Metric, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, LpDistance},
    dom::{AllDomain, VectorDomain},
    error::{ExplainUnwrap, Fallible},
    traits::{AlertingAbs, CheckNull, DistanceConstant, InfAdd, InfCast, RoundCast},
};

pub trait LipschitzMulDomain: Domain {
    type Atom;
    fn transform(l: Self::Atom, v: &Self::Carrier) -> Self::Carrier;
}

impl<T> LipschitzMulDomain for AllDomain<T>
where
    T: for<'a> Mul<&'a T, Output = T> + CheckNull,
{
    type Atom = T;
    fn transform(l: T, v: &T) -> T {
        l * v
    }
}

impl<T> LipschitzMulDomain for VectorDomain<AllDomain<T>>
where
    T: Clone + for<'a> Mul<&'a T, Output = T> + CheckNull,
{
    type Atom = T;
    fn transform(l: T, v: &Vec<T>) -> Vec<T> {
        v.iter().map(|v_i| l.clone() * v_i).collect()
    }
}

pub trait LipschitzMulMetric: Metric {}

impl<T, const P: usize> LipschitzMulMetric for LpDistance<T, P> {}
impl<T> LipschitzMulMetric for AbsoluteDistance<T> {}

pub fn make_lipschitz_mul<D, M>(l: D::Atom) -> Fallible<Transformation<D, D, M, M>>
where
    D: LipschitzMulDomain + Default,
    D::Atom: AlertingAbs,
    M: LipschitzMulMetric<Distance = D::Atom>,
    M::Distance: DistanceConstant<M::Distance>,
{
    Ok(Transformation::new(
        D::default(),
        D::default(),
        Function::new(enclose!(l, move |arg: &D::Carrier| D::transform(
            l.clone(),
            arg
        ))),
        M::default(),
        M::default(),
        // TODO: check that negative l is permissible
        StabilityRelation::new_from_constant(l.alerting_abs()?),
    ))
}

pub trait LipschitzCastDomain<DI>: Domain
where
    DI: Domain,
{
    fn transform(v: &DI::Carrier) -> Self::Carrier;
}

impl<TI, TO> LipschitzCastDomain<AllDomain<TI>> for AllDomain<TO>
where
    TI: CheckNull + Clone,
    TO: RoundCast<TI> + CheckNull,
{
    fn transform(v: &TI) -> TO {
        TO::round_cast(v.clone())
            .unwrap_assert("casting from floats to ints is infallible in saturation arithmetic")
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
// integers
cartesian! {[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128], impl_greatest_cast_1, impl_greatest_cast_0, impl_greatest_cast_1}
// float to int
cartesian!(
    [f32, f64],
    [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128],
    impl_greatest_cast_1
);
// int to float (essentially unimplemented; the bounds are infinitely loose)
cartesian!(
    [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128],
    [f32, f64],
    impl_greatest_cast_inf
);
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
/// Create a data transformation that rounds a float to the nearest integer on an arbitrary input domain.
///
pub fn make_lipschitz_cast<DI, DO, MI, MO>() -> Fallible<Transformation<DI, DO, MI, MO>>
where
    DI: Domain + Default,
    DO: LipschitzCastDomain<DI> + Default,
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
        StabilityRelation::new_all(
            |d_in: &MI::Distance, d_out: &MO::Distance| {
                Ok(d_out.clone()
                    >= MO::Distance::inf_cast(d_in.clone())?.inf_add(&MO::Distance::C)?)
            },
            Some(|d_in: &MI::Distance| {
                Ok(MO::Distance::inf_cast(d_in.clone())?.inf_add(&MO::Distance::C)?)
            }),
            None::<fn(&_) -> _>,
        ),
    ))
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_lipschitz() -> Fallible<()> {
        let extension = make_lipschitz_mul::<AllDomain<f64>, AbsoluteDistance<f64>>(2.)?;
        assert_eq!(extension.invoke(&1.3)?, 2.6);
        println!("{:?}", extension.invoke(&1.3));
        Ok(())
    }

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

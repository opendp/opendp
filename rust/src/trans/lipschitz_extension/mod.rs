use std::ops::Mul;

use num::{Float, One, NumCast, ToPrimitive, Integer};

use crate::{
    core::{Domain, Function, Metric, StabilityRelation, Transformation},
    dist::{AbsoluteDistance, LpDistance},
    dom::{AllDomain, VectorDomain},
    error::{Fallible, ExplainUnwrap},
    traits::{CheckNull, DistanceConstant, AlertingAbs},
};

pub trait LipschitzDomain: Domain {
    type Atom;
    fn transform(l: Self::Atom, v: &Self::Carrier) -> Self::Carrier;
}

impl<T> LipschitzDomain for AllDomain<T>
where
    T: for<'a> Mul<&'a T, Output = T> + CheckNull,
{
    type Atom = T;
    fn transform(l: T, v: &T) -> T {
        l * v
    }
}

impl<T> LipschitzDomain for VectorDomain<AllDomain<T>>
where
    T: Clone + for<'a> Mul<&'a T, Output = T> + CheckNull,
{
    type Atom = T;
    fn transform(l: T, v: &Vec<T>) -> Vec<T> {
        v.iter().map(|v_i| l.clone() * v_i).collect()
    }
}

pub trait LipschitzMetric: Metric {}

impl<T, const P: usize> LipschitzMetric for LpDistance<T, P> {}
impl<T> LipschitzMetric for AbsoluteDistance<T> {}

pub fn make_lipschitz_extension<D, M>(l: D::Atom) -> Fallible<Transformation<D, D, M, M>>
where
    D: LipschitzDomain + Default,
    D::Atom: AlertingAbs,
    M: LipschitzMetric<Distance = D::Atom>,
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

pub trait IntegerizeDomain<DO>: Domain
where
    DO: Domain,
{
    fn transform(v: &Self::Carrier) -> DO::Carrier;
}

impl<TI, TO> IntegerizeDomain<AllDomain<TO>> for AllDomain<TI>
where
    TI: Float + CheckNull + ToPrimitive,
    TO: NumCast + CheckNull + Integer,
{
    fn transform(v: &TI) -> TO {
        TO::from(v.round()).unwrap_assert("casting from floats to ints is infallible in saturation arithmetic")
    }
}

impl<DI, DO> IntegerizeDomain<VectorDomain<DO>> for VectorDomain<DI>
where
    DI: IntegerizeDomain<DO>,
    DO: Domain,
{
    fn transform(v: &Vec<DI::Carrier>) -> Vec<DO::Carrier> {
        v.iter().map(DI::transform).collect()
    }
}

/// Create a data transformation that rounds a float to the nearest integer on an arbitrary input domain.
///
pub fn make_integerize<DI, DO, M>() -> Fallible<Transformation<DI, DO, M, M>>
where
    DI: Domain + Default,
    DO: Domain + Default,
    M: Metric,
    M::Distance: 'static + Float,
    DI: IntegerizeDomain<DO>,
{
    Ok(Transformation::new(
        DI::default(),
        DO::default(),
        Function::new(|arg: &DI::Carrier| DI::transform(arg)),
        M::default(),
        M::default(),
        StabilityRelation::new_all(
            |d_in: &M::Distance, d_out: &M::Distance| {
                Ok(d_out.clone() >= d_in.clone() + M::Distance::one())
            },
            Some(|d_in: &M::Distance| Ok(d_in.clone() + M::Distance::one())),
            None::<fn(&_) -> _>,
        ),
    ))
}


#[cfg(test)]
pub mod test {
    use super::*;


    #[test]
    fn test_lipschitz() -> Fallible<()> {
        let extension = make_lipschitz_extension::<AllDomain<f64>, AbsoluteDistance<f64>>(2.)?;
        assert_eq!(extension.invoke(&1.3)?, 2.6);
        println!("{:?}", extension.invoke(&1.3));
        Ok(())
    }
    
    #[test]
    fn test_integerize() -> Fallible<()> {
        let integerizer = make_integerize::<AllDomain<f64>, AllDomain<i64>, AbsoluteDistance<f64>>()?;
        assert_eq!(integerizer.invoke(&1.3)?, 1);
        println!("{:?}", integerizer.invoke(&1.3));
        Ok(())
    }
}
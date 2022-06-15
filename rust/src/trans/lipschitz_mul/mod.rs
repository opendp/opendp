use std::ops::Mul;

use crate::{
    core::{Domain, Function, Metric, StabilityMap, Transformation},
    dist::{AbsoluteDistance, LpDistance},
    dom::{AllDomain, VectorDomain},
    error::Fallible,
    traits::{AlertingAbs, CheckNull, DistanceConstant},
};

pub fn make_lipschitz_mul<D, M>(l: D::Atom) -> Fallible<Transformation<D, D, M, M>>
where
    D: LipschitzMulDomain,
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
        StabilityMap::new_from_constant(l.alerting_abs()?),
    ))
}

/// Implemented for any domain that supports multiplication lipschitz extensions
pub trait LipschitzMulDomain: Domain + Default {
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

/// Implemented for any metric that supports multiplication lipschitz extensions
pub trait LipschitzMulMetric: Metric {}

impl<T, const P: usize> LipschitzMulMetric for LpDistance<T, P> {}
impl<T> LipschitzMulMetric for AbsoluteDistance<T> {}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_lipschitz_mul() -> Fallible<()> {
        let extension = make_lipschitz_mul::<AllDomain<f64>, AbsoluteDistance<f64>>(2.)?;
        assert_eq!(extension.invoke(&1.3)?, 2.6);
        println!("{:?}", extension.invoke(&1.3));
        Ok(())
    }
}

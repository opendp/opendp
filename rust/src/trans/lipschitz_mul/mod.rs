use crate::{
    core::{Domain, Function, Metric, StabilityMap, Transformation},
    dist::{AbsoluteDistance, LpDistance},
    dom::{AllDomain, VectorDomain},
    error::Fallible,
    traits::{AlertingAbs, CheckNull, SaturatingMul, InfMul},
};

pub fn make_lipschitz_mul<D, M>(l: D::Atom) -> Fallible<Transformation<D, D, M, M>>
where
    D: LipschitzMulDomain,
    M: LipschitzMulMetric<Distance = D::Atom>,
{
    Ok(Transformation::new(
        D::default(),
        D::default(),
        Function::new(enclose!(l, move |arg: &D::Carrier| {
            D::transform(l.clone(), arg)
        })),
        M::default(),
        M::default(),
        StabilityMap::new_fallible(move |d_in: &M::Distance| d_in.inf_mul(&l.alerting_abs()?)),
    ))
}

/// Implemented for any domain that supports multiplication lipschitz extensions
pub trait LipschitzMulDomain: Domain + Default {
    type Atom: 'static + AlertingAbs + InfMul + Clone + SaturatingMul + CheckNull;
    fn transform(l: Self::Atom, v: &Self::Carrier) -> Self::Carrier;
}

impl<T> LipschitzMulDomain for AllDomain<T>
where
    T: 'static + AlertingAbs + InfMul + Clone + SaturatingMul + CheckNull,
{
    type Atom = T;
    fn transform(l: T, v: &T) -> T {
        l.saturating_mul(v)
    }
}

impl<T> LipschitzMulDomain for VectorDomain<AllDomain<T>>
where
    T: 'static + AlertingAbs + InfMul + Clone + SaturatingMul + CheckNull,
{
    type Atom = T;
    fn transform(l: T, v: &Vec<T>) -> Vec<T> {
        v.iter().map(|v_i| l.saturating_mul(v_i)).collect()
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

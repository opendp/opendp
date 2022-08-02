use num::Float as _;

use crate::{
    core::{Measure, Measurement, PrivacyMap, SensitivityMetric},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{samplers::SampleDiscreteGaussianZ2k, Float, FloatBits, ExactIntCast, CheckNull},
};

use super::{get_discretization_consts, MappableDomain};

#[cfg(feature = "ffi")]
mod ffi;

pub trait GaussianDomain: MappableDomain + Default {
    type InputMetric: SensitivityMetric<Distance = Self::Atom> + Default;
}
impl<T: Clone + CheckNull> GaussianDomain for AllDomain<T> {
    type InputMetric = AbsoluteDistance<T>;
}
impl<T: Clone + CheckNull> GaussianDomain for VectorDomain<AllDomain<T>> {
    type InputMetric = L2Distance<T>;
}

pub trait GaussianMeasure<DI: GaussianDomain>: Measure + Default {
    fn new_forward_map(scale: DI::Atom, relaxation: DI::Atom) -> PrivacyMap<DI::InputMetric, Self>;
}

impl<DI, Q> GaussianMeasure<DI> for ZeroConcentratedDivergence<Q>
where
    DI: GaussianDomain<Atom = Q>,
    Q: Float,
{
    fn new_forward_map(scale: Q, relaxation: Q) -> PrivacyMap<DI::InputMetric, Self> {
        let _2 = Q::one() + Q::one();
        PrivacyMap::new_fallible(move |d_in: &Q| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if scale.is_zero() {
                return Ok(Q::infinity())
            }

            // d_in is loosened by the size of the granularization
            let d_in = d_in.inf_add(&relaxation)?;

            // (d_in / scale)^2 / 2
            (d_in.inf_div(&scale)?).inf_pow(&_2)?.inf_div(&_2)
        })
    }
}

pub fn make_base_gaussian<D, MO>(scale: D::Atom, k: Option<i32>) -> Fallible<Measurement<D, D, D::InputMetric, MO>>
where
    D: GaussianDomain,
    D::Atom: Float + SampleDiscreteGaussianZ2k,
    MO: GaussianMeasure<D>,
    i32: ExactIntCast<<D::Atom as FloatBits>::Bits>
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    
    let (k , relaxation) = get_discretization_consts(k)?;

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(move |arg: &D::Atom| D::Atom::sample_discrete_gaussian_Z2k(*arg, scale, k)),
        D::InputMetric::default(),
        MO::default(),
        MO::new_forward_map(scale, relaxation),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement =
            make_base_gaussian::<VectorDomain<_>, ZeroConcentratedDivergence<_>>(1.0f64, None)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.map(&0.1)? <= 0.0050000001);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_mechanism_zcdp() -> Fallible<()> {
        let measurement =
            make_base_gaussian::<AllDomain<_>, ZeroConcentratedDivergence<_>>(1.0f64, None)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &0.0050000001)?);
        Ok(())
    }
}

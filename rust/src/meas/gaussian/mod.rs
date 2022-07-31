use num::Float as _;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{samplers::SampleDiscreteGaussianZ2k, CheckNull, Float},
};

#[cfg(feature = "ffi")]
mod ffi;

pub trait GaussianDomain: Domain {
    type Metric: GaussianMetric<Distance = Self::Atom> + Default;
    type Atom: Float + SampleDiscreteGaussianZ2k;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom, k: i32) -> Function<Self, Self>;
}

impl<T> GaussianDomain for AllDomain<T>
where
    T: Float + SampleDiscreteGaussianZ2k,
{
    type Metric = AbsoluteDistance<T>;
    type Atom = T;

    fn new() -> Self {
        AllDomain::new()
    }
    fn noise_function(scale: Self::Carrier, k: i32) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            Self::Carrier::sample_discrete_gaussian_Z2k(*arg, scale, k)
        })
    }
}

impl<T> GaussianDomain for VectorDomain<AllDomain<T>>
where
    T: Float + SampleDiscreteGaussianZ2k,
{
    type Metric = L2Distance<T>;
    type Atom = T;

    fn new() -> Self {
        VectorDomain::new_all()
    }
    fn noise_function(scale: T, k: i32) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            arg.iter()
                .map(|v| T::sample_discrete_gaussian_Z2k(*v, scale, k))
                .collect()
        })
    }
}

pub trait GaussianMeasure<MI: GaussianMetric>: Measure + Default {
    type Atom;
    fn new_forward_map(scale: Self::Atom, k: i32) -> PrivacyMap<MI, Self>;
}

pub trait GaussianMetric: Metric {}
impl<Q: CheckNull> GaussianMetric for L2Distance<Q> {}
impl<Q: CheckNull> GaussianMetric for AbsoluteDistance<Q> {}

impl<MI, Q> GaussianMeasure<MI> for ZeroConcentratedDivergence<Q>
where
    MI: GaussianMetric<Distance = Q>,
    Q: Float,
{
    type Atom = Q;

    fn new_forward_map(scale: Q, k: i32) -> PrivacyMap<MI, Self> {
        PrivacyMap::new_fallible(move |d_in: &Q| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative")
            }
            if scale.is_zero() {
                return Ok(Q::infinity())
            }
            
            let _2 = Q::exact_int_cast(2)?;
            let k = Q::exact_int_cast(k)?;

            // d_in is loosened by the size of the granularization
            let d_in = d_in.inf_add(&_2.inf_pow(&k)?)?;

            // (d_in / scale)^2 / 2
            (d_in.inf_div(&scale)?).inf_pow(&_2)?.inf_div(&_2)
        })
    }
}

pub fn make_base_gaussian<D, MO>(scale: D::Atom) -> Fallible<Measurement<D, D, D::Metric, MO>>
where
    D: GaussianDomain,
    MO: GaussianMeasure<D::Metric, Atom = D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let k = -40;
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone(), k),
        D::Metric::default(),
        MO::default(),
        MO::new_forward_map(scale, k),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement =
            make_base_gaussian::<VectorDomain<_>, ZeroConcentratedDivergence<_>>(1.0f64)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.map(&0.1)? <= 0.0050000001);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_mechanism_zcdp() -> Fallible<()> {
        let measurement =
            make_base_gaussian::<AllDomain<_>, ZeroConcentratedDivergence<_>>(1.0f64)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &0.0050000001)?);
        Ok(())
    }
}

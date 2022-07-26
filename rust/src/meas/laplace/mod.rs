#[cfg(feature="ffi")]
mod ffi;

use num::{Zero, Float as _};

use crate::core::{Measurement, Function, PrivacyMap, Domain, SensitivityMetric};
use crate::measures::MaxDivergence;
use crate::metrics::{L1Distance, AbsoluteDistance};
use crate::domains::{AllDomain, VectorDomain};
use crate::error::*;
use crate::traits::samplers::SampleDiscreteLaplace;
use crate::traits::{InfDiv, Float};

pub trait LaplaceDomain: Domain {
    type Metric: SensitivityMetric<Distance=Self::Atom>;
    type Atom: Float + SampleDiscreteLaplace;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom, granularity: usize) -> Function<Self, Self>;
}

impl<T> LaplaceDomain for AllDomain<T>
    where T: Float + SampleDiscreteLaplace {
    type Metric = AbsoluteDistance<T>;
    type Atom = Self::Carrier;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: Self::Carrier, granularity: usize) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| Self::Carrier::sample_discrete_laplace(*arg, scale, granularity))
    }
}

impl<T> LaplaceDomain for VectorDomain<AllDomain<T>>
    where T: Float + SampleDiscreteLaplace {
    type Metric = L1Distance<T>;
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: T, granularity: usize) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_discrete_laplace(*v, scale, granularity))
            .collect())
    }
}

pub fn make_base_laplace<D>(scale: D::Atom, granularity: Option<usize>) -> Fallible<Measurement<D, D, D::Metric, MaxDivergence<D::Atom>>>
    where D: LaplaceDomain {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    let granularity = granularity.unwrap_or(32);

    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone(), granularity),
        D::Metric::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(
            move |d_in: &D::Atom| {
                if d_in.is_sign_negative() {
                    return fallible!(InvalidDistance, "sensitivity must be non-negative")
                }
                if scale.is_zero() {
                    return Ok(D::Atom::infinity())
                }
                // d_in / scale
                d_in.inf_div(&scale)
            })
    ))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{trans::make_sized_bounded_mean, metrics::SymmetricDistance};

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (
            make_sized_bounded_mean::<SymmetricDistance, _>(3, (10.0, 12.0))? >>
            make_base_laplace(1.0, None)?
        )?;
        let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
        Ok(())

    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<AllDomain<_>>(1.0, None)?;
        let _ret = measurement.invoke(&0.0)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<VectorDomain<_>>(1.0, None)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }
}


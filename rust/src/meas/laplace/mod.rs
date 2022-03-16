#[cfg(feature="ffi")]
mod ffi;

use num::{Float};

use crate::core::{Measurement, Function, PrivacyRelation, Domain, SensitivityMetric};
use crate::dist::{L1Distance, MaxDivergence, AbsoluteDistance};
use crate::dom::{AllDomain, VectorDomain};
use crate::samplers::{SampleLaplace};
use crate::error::*;
use crate::traits::{InfCast, CheckNull, TotalOrd, InfMul};

pub trait LaplaceDomain: Domain {
    type Metric: SensitivityMetric<Distance=Self::Atom> + Default;
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom) -> Function<Self, Self>;
}

impl<T> LaplaceDomain for AllDomain<T>
    where T: 'static + SampleLaplace + Float + CheckNull {
    type Metric = AbsoluteDistance<T>;
    type Atom = Self::Carrier;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: Self::Carrier) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| Self::Carrier::sample_laplace(*arg, scale, false))
    }
}

impl<T> LaplaceDomain for VectorDomain<AllDomain<T>>
    where T: 'static + SampleLaplace + Float + CheckNull {
    type Metric = L1Distance<T>;
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: T) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_laplace(*v, scale, false))
            .collect())
    }
}

pub fn make_base_laplace<D>(scale: D::Atom) -> Fallible<Measurement<D, D, D::Metric, MaxDivergence<D::Atom>>>
    where D: LaplaceDomain,
          D::Atom: 'static + Clone + SampleLaplace + Float + InfCast<D::Atom> + CheckNull + TotalOrd + InfMul {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_all(
            move |d_in: &D::Atom, d_out: &D::Atom| {
                if d_in.is_sign_negative() {
                    return fallible!(InvalidDistance, "sensitivity must be non-negative")
                }
                if d_out.is_sign_negative() {
                    return fallible!(InvalidDistance, "epsilon must be non-negative")
                }
                // d_out * scale >= d_in
                Ok(d_out.neg_inf_mul(&scale)? >= d_in.clone())
            },
            Some(move |d_out: &D::Atom| d_out.neg_inf_mul(&scale)))
    ))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::trans::make_sized_bounded_mean;

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (
            make_sized_bounded_mean(3, (10.0, 12.0))? >>
            make_base_laplace(1.0)?
        )?;
        let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
        Ok(())

    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<AllDomain<_>>(1.0)?;
        let _ret = measurement.invoke(&0.0)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<VectorDomain<_>>(1.0)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }
}


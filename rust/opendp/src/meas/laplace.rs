use num::Float;

use crate::core::{Measurement, Function, PrivacyRelation, Domain, SensitivityMetric};
use crate::dist::{L1Distance, MaxDivergence, AbsoluteDistance};
use crate::dom::{AllDomain, VectorDomain};
use crate::samplers::{SampleLaplace};
use crate::error::*;
use crate::traits::DistanceCast;

pub trait LaplaceDomain: Domain {
    type Metric: SensitivityMetric<Distance=Self::Atom> + Default;
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom) -> Function<Self, Self>;
}

impl<T> LaplaceDomain for AllDomain<T>
    where T: 'static + SampleLaplace + Float + DistanceCast {
    type Metric = AbsoluteDistance<T>;
    type Atom = Self::Carrier;

    fn new() -> Self { AllDomain::new() }
    fn noise_function(scale: Self::Carrier) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| Self::Carrier::sample_laplace(*arg, scale, false))
    }
}

impl<T> LaplaceDomain for VectorDomain<AllDomain<T>>
    where T: 'static + SampleLaplace + Float + DistanceCast {
    type Metric = L1Distance<T>;
    type Atom = T;

    fn new() -> Self { VectorDomain::new_all() }
    fn noise_function(scale: T) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| arg.iter()
            .map(|v| T::sample_laplace(*v, scale, false))
            .collect())
    }
}

pub fn make_base_laplace<DI>(scale: DI::Atom) -> Fallible<Measurement<DI, DI, DI::Metric, MaxDivergence<DI::Atom>>>
    where DI: LaplaceDomain,
          DI::Atom: 'static + Clone + SampleLaplace + Float + DistanceCast {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        DI::new(),
        DI::new(),
        DI::noise_function(scale.clone()),
        DI::Metric::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale.recip())
    ))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::trans::make_bounded_mean;
    use crate::dist::HammingDistance;

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (
            make_bounded_mean::<HammingDistance, _>(10.0, 12.0, 3)? >>
            make_base_laplace(1.0)?
        )?;
        let _ret = chain.function.eval(&vec![10.0, 11.0, 12.0])?;
        Ok(())

    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<AllDomain<_>>(1.0)?;
        let _ret = measurement.function.eval(&0.0)?;

        assert!(measurement.privacy_relation.eval(&1., &1.)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<VectorDomain<_>>(1.0)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&1., &1.)?);
        Ok(())
    }
}


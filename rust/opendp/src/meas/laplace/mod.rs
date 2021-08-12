use num::{Float, One};
use rug::float::Round;

use crate::core::{Measurement, Function, PrivacyRelation, Measure, Metric, Domain, SensitivityMetric};
use crate::dist::{L1Distance, MaxDivergence, AbsoluteDistance, FSmoothedMaxDivergence, EpsilonDelta};
use crate::dom::{AllDomain, VectorDomain};
use crate::samplers::{SampleLaplace, CastInternalReal};
use crate::error::*;
use crate::traits::{InfCast, CheckNull, TotalOrd};


pub trait LaplacePrivacyRelation<MI: Metric>: Measure {
    fn privacy_relation(scale: MI::Distance) -> PrivacyRelation<MI, Self>;
}

impl<MI: Metric> LaplacePrivacyRelation<MI> for MaxDivergence<MI::Distance>
    where MI::Distance: 'static + Clone + Float + SampleLaplace + InfCast<<MI as Metric>::Distance> + TotalOrd {
    fn privacy_relation(scale: MI::Distance) -> PrivacyRelation<MI, Self>{
        PrivacyRelation::new_from_constant(scale.recip())
    }
}

impl<MI: Metric> LaplacePrivacyRelation<MI> for FSmoothedMaxDivergence<MI::Distance>
    where MI::Distance: 'static + Clone + Float + One + SampleLaplace + CastInternalReal {

    fn privacy_relation(scale: MI::Distance) -> PrivacyRelation<MI, Self> {
        PrivacyRelation::new_fallible(move |d_in: &MI::Distance, d_out: &Vec<EpsilonDelta<MI::Distance>>| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "laplace mechanism: input sensitivity must be non-negative")
            }

            let mut result = true;
            for EpsilonDelta { epsilon, delta } in d_out {
                if epsilon.is_sign_negative() {
                    return fallible!(InvalidDistance, "laplace mechanism: epsilon must be positive or 0")
                }
                if delta.is_sign_negative() {
                    return fallible!(InvalidDistance, "laplace mechanism: delta must be positive or 0")
                }

                // Compute delta_dual = 1 - exp((epsilon - 1. / scale ) / 2) with rounding up
                //let epsilon_float: rug::Float = epsilon.into_internal();
                let mut delta_dual: rug::Float = scale.into_internal();
                delta_dual.recip_round(Round::Up);
                delta_dual = epsilon.into_internal() - delta_dual;
                delta_dual.div_assign_round(( MI::Distance::one() +  MI::Distance::one()).into_internal(), Round::Down)
                delta_dual.exp_round(Round::Down);
                delta_dual = (MI::Distance::one().into_internal()) - delta_dual;



                //let delta_dual = MI::Distance::one() - ((*epsilon - scale.recip()) / (MI::Distance::one() + MI::Distance::one())).exp();
                result = result & (delta >= &delta_dual);
                if result == false {
                    break;
                }
            }
            Ok(result)
        })
       }
}

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

pub fn make_base_laplace<D, MO>(scale: D::Atom) -> Fallible<Measurement<D, D, D::Metric, MO>>
    where D: LaplaceDomain,
          D::Atom: 'static + Clone + SampleLaplace + Float + InfCast<D::Atom> + CheckNull + TotalOrd,
          MO: Measure + LaplacePrivacyRelation<D::Metric> {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale.clone()),
        D::Metric::default(),
        MO::default(),
        MO::privacy_relation(scale),
    ))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::trans::make_bounded_mean;

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (
            make_bounded_mean(10.0, 12.0, 3)? >>
            make_base_laplace::<AllDomain<_>, MaxDivergence<_>>(1.0)?
        )?;
        let _ret = chain.function.eval(&vec![10.0, 11.0, 12.0])?;
        Ok(())

    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<AllDomain<_>, MaxDivergence<_>>(1.0)?;
        let _ret = measurement.function.eval(&0.0)?;

        assert!(measurement.privacy_relation.eval(&1., &1.)?);

        let measurement = make_base_laplace::<AllDomain<_>, FSmoothedMaxDivergence<_>>(1.0)?;
        let _ret = measurement.function.eval(&0.0)?;

        let d_out = vec!(EpsilonDelta{epsilon: 1., delta: 0.});
        assert!(measurement.privacy_relation.eval(&1., &d_out)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<VectorDomain<_>, MaxDivergence<_>>(1.0)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&1., &1.)?);

        let measurement = make_base_laplace::<VectorDomain<_>, FSmoothedMaxDivergence<_>>(1.0)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg)?;

        let d_out = vec!(EpsilonDelta{epsilon: 1., delta: 0.});
        assert!(measurement.privacy_relation.eval(&1., &d_out)?);
        Ok(())
    }
}


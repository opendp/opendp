#[cfg(feature="ffi")]
mod ffi;

use crate::core::{Domain, Measurement, Metric, PrivacyMap, Measure};
use crate::dist::{MaxDivergence, FixedSmoothedMaxDivergence};
use crate::dom::SizedDomain;
use crate::error::Fallible;
use crate::traits::{ExactIntCast, InfMul, InfExpM1, InfLn1P, InfDiv};

pub trait IsSizedDomain: Domain { fn get_size(&self) -> Fallible<usize>; }
impl<D: Domain> IsSizedDomain for SizedDomain<D> {
    fn get_size(&self) -> Fallible<usize> { Ok(self.size) }
}

pub trait AmplifiableMeasure: Measure {
    fn amplify(&self, budget: &Self::Distance, population_size: usize, sample_size: usize) -> Fallible<Self::Distance>;
}

impl<Q> AmplifiableMeasure for MaxDivergence<Q>
    where Q: ExactIntCast<usize> + InfMul + InfExpM1 + InfLn1P + InfDiv + Clone {
    fn amplify(&self, epsilon: &Q, population_size: usize, sample_size: usize) -> Fallible<Q> {
        let sampling_rate = Q::exact_int_cast(sample_size)?.inf_div(&Q::exact_int_cast(population_size)?)?;
        epsilon.clone().inf_exp_m1()?.inf_mul(&sampling_rate)?.inf_ln_1p()
    }
}
impl<Q> AmplifiableMeasure for FixedSmoothedMaxDivergence<Q>
    where Q: ExactIntCast<usize> + InfMul + InfExpM1 + InfLn1P + InfDiv + Clone {
    fn amplify(&self, (epsilon, delta): &(Q, Q), population_size: usize, sample_size: usize) -> Fallible<(Q, Q)> {
        let sampling_rate = Q::exact_int_cast(sample_size)?.inf_div(&Q::exact_int_cast(population_size)?)?;
        Ok((epsilon.clone().inf_exp_m1()?.inf_mul(&sampling_rate)?.inf_ln_1p()?, delta.inf_mul(&sampling_rate)?))
    }
}

pub fn make_population_amplification<DIA, DO, MI, MO>(
    measurement: &Measurement<DIA, DO, MI, MO>,
    population_size: usize,
) -> Fallible<Measurement<DIA, DO, MI, MO>>
    where DIA: IsSizedDomain,
          DO: Domain,
          MI: 'static + Metric,
          MO: 'static + AmplifiableMeasure {
    let mut measurement = measurement.clone();
    let sample_size = measurement.input_domain.get_size()?;
    if population_size < sample_size { 
        return fallible!(MakeMeasurement, "population size cannot be less than sample size") 
    }

    let privacy_map = measurement.privacy_map;
    let output_measure: MO = measurement.output_measure.clone();

    measurement.privacy_map = PrivacyMap::new_fallible(
        move |d_in| output_measure.amplify(&privacy_map.eval(d_in)?, population_size, sample_size));

    Ok(measurement)
}

#[cfg(test)]
mod test {
    use crate::error::Fallible;
    use crate::trans::make_sized_bounded_mean;
    use crate::meas::make_base_laplace;
    use crate::comb::make_population_amplification;

    #[test]
    fn test_amplifier() -> Fallible<()> {
        let meas = (make_sized_bounded_mean(10, (0., 10.))? >> make_base_laplace(0.5)?)?;
        let amp = make_population_amplification(&meas, 100)?;
        amp.function.eval(&vec![1.; 10])?;
        assert!(meas.check(&2, &2.)?);
        assert!(!meas.check(&2, &1.999)?);
        assert!(amp.check(&2, &0.4941)?);
        assert!(!amp.check(&2, &0.494)?);
        Ok(())
    }
}

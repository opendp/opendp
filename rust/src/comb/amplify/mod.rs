#[cfg(feature="ffi")]
mod ffi;

use crate::core::{Domain, Measurement, Metric, PrivacyRelation, Measure};
use crate::dist::{SmoothedMaxDivergence, MaxDivergence};
use crate::dom::SizedDomain;
use crate::error::Fallible;
use num::Float;
use crate::traits::ExactIntCast;
use std::ops::Div;

pub trait IsSizedDomain: Domain { fn get_size(&self) -> Fallible<usize>; }
impl<D: Domain> IsSizedDomain for SizedDomain<D> {
    fn get_size(&self) -> Fallible<usize> { Ok(self.size) }
}

pub trait AmplifiableMeasure: Measure {
    fn amplify(&self, budget: &Self::Distance, population_size: usize, sample_size: usize) -> Fallible<Self::Distance>;
}

impl<Q> AmplifiableMeasure for MaxDivergence<Q>
    where Q: ExactIntCast<usize> + Div<Output=Q> + Float {
    fn amplify(&self, epsilon: &Q, population_size: usize, sample_size: usize) -> Fallible<Q> {
        let sampling_rate = Q::exact_int_cast(sample_size)? / Q::exact_int_cast(population_size)?;
        Ok((epsilon.exp_m1() / sampling_rate).ln_1p())
    }
}
impl<Q> AmplifiableMeasure for SmoothedMaxDivergence<Q>
    where Q: ExactIntCast<usize> + Div<Output=Q> + Float {
    fn amplify(&self, (epsilon, delta): &(Q, Q), population_size: usize, sample_size: usize) -> Fallible<(Q, Q)> {
        let sampling_rate = Q::exact_int_cast(sample_size)? / Q::exact_int_cast(population_size)?;
        Ok(((epsilon.exp_m1() / sampling_rate).ln_1p(), *delta / sampling_rate))
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

    let privacy_relation = measurement.privacy_relation;
    let output_measure: MO = measurement.output_measure.clone();

    measurement.privacy_relation = PrivacyRelation::new_fallible(
        move |d_in, d_out: &MO::Distance| privacy_relation.eval(
            d_in, &output_measure.amplify(d_out, population_size, sample_size)?));

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
        assert!(meas.privacy_relation.eval(&2, &2.)?);
        assert!(!meas.privacy_relation.eval(&2, &1.999)?);
        assert!(amp.privacy_relation.eval(&2, &0.4941)?);
        assert!(!amp.privacy_relation.eval(&2, &0.494)?);
        Ok(())
    }
}

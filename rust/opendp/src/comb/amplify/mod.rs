use crate::core::{Domain, Measurement, Metric, PrivacyRelation, Measure};
use crate::dist::{SmoothedMaxDivergence, MaxDivergence};
use crate::dom::SizedDomain;
use crate::error::Fallible;
use num::Float;
use crate::traits::ExactIntCast;
use std::ops::Div;
use std::fmt::Debug;

pub trait AmplifiableMeasure: Measure {
    type Atom;
    fn amplify(budget: Self::Distance, sampling_rate: Self::Atom) -> Self::Distance;
}

impl<Q: Float + Debug> AmplifiableMeasure for MaxDivergence<Q> {
    type Atom = Q;
    fn amplify(epsilon: Q, sampling_rate: Q) -> Q {
        (epsilon.exp_m1() / sampling_rate).ln_1p()
    }
}
impl<Q: Float + Debug> AmplifiableMeasure for SmoothedMaxDivergence<Q> {
    type Atom = Q;
    fn amplify((epsilon, delta): (Q, Q), sampling_rate: Q) -> (Q, Q) {
        ((epsilon.exp_m1() / sampling_rate).ln_1p(), delta / sampling_rate)
    }
}

pub fn make_population_amplification<DIA, DO, MI, MO>(
    measurement: &Measurement<SizedDomain<DIA>, DO, MI, MO>,
    n_population: usize,
) -> Fallible<Measurement<SizedDomain<DIA>, DO, MI, MO>>
    where DIA: Domain,
          DO: Domain,
          MI: 'static + Metric,
          MO: 'static + Measure + AmplifiableMeasure,
          MO::Atom: ExactIntCast<usize> + Div<Output=MO::Atom> + Clone,
          MO::Distance: Clone {
    let mut measurement = measurement.clone();
    let n_sample = measurement.input_domain.length;
    if n_population < n_sample { return fallible!(MakeMeasurement, "population size cannot be less than sample size") }

    let privacy_relation = measurement.privacy_relation;
    let sampling_rate = MO::Atom::exact_int_cast(n_sample)? / MO::Atom::exact_int_cast(n_population)?;

    measurement.privacy_relation = PrivacyRelation::new_fallible(
        move |d_in, d_out: &MO::Distance| privacy_relation.eval(
            d_in, &MO::amplify(d_out.clone(), sampling_rate.clone())));

    Ok(measurement)
}

#[cfg(test)]
mod test {
    use crate::error::Fallible;
    use crate::trans::make_bounded_mean;
    use crate::meas::make_base_laplace;
    use crate::comb::make_population_amplification;

    #[test]
    fn test_amplifier() -> Fallible<()> {
        let meas = (make_bounded_mean(0., 10., 10)? >> make_base_laplace(0.5)?)?;
        let amp = make_population_amplification(&meas, 100)?;
        amp.function.eval(&vec![1.; 10])?;
        assert!(meas.privacy_relation.eval(&1, &1.)?);
        assert!(!meas.privacy_relation.eval(&1, &0.999)?);
        assert!(amp.privacy_relation.eval(&1, &0.159)?);
        assert!(!amp.privacy_relation.eval(&1, &0.158)?);
        Ok(())
    }
}

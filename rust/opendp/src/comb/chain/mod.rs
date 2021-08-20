use std::ops::Shr;
use std::fmt::Debug;
use num::{One, Zero, Float};

use crate::core::{Domain, Function, HintMt, HintTt, Measure, Measurement, Metric, PrivacyRelation, StabilityRelation, Transformation};
use crate::dom::{PairDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{Tolerance, Midpoint};
use crate::samplers::{CastRational, CastInternalReal};
use crate::dist::{FSmoothedMaxDivergence, ProbabilitiesRatios, EpsilonDelta};

pub fn make_chain_mt<DI, DX, DO, MI, MX, MO>(
    measurement1: &Measurement<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
    hint: Option<&HintMt<MI, MO, MX>>,
) -> Fallible<Measurement<DI, DO, MI, MO>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    if transformation0.output_domain != measurement1.input_domain {
        return fallible!(DomainMismatch, "Intermediate domain mismatch");
    } else if transformation0.output_metric != measurement1.input_metric {
        return fallible!(MetricMismatch, "Intermediate metric mismatch");
    }

    Ok(Measurement::new(
        transformation0.input_domain.clone(),
        measurement1.output_domain.clone(),
        Function::make_chain(&measurement1.function, &transformation0.function),
        transformation0.input_metric.clone(),
        measurement1.output_measure.clone(),
        PrivacyRelation::make_chain(&measurement1.privacy_relation,&transformation0.stability_relation, hint)
    ))
}

pub fn make_chain_tt<DI, DX, DO, MI, MX, MO>(
    transformation1: &Transformation<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
    hint: Option<&HintTt<MI, MO, MX>>,
) -> Fallible<Transformation<DI, DO, MI, MO>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    if transformation0.output_domain != transformation1.input_domain {
        return fallible!(DomainMismatch, "Intermediate domain mismatch");
    } else if transformation0.output_metric != transformation1.input_metric {
        return fallible!(MetricMismatch, "Intermediate metric mismatch");
    }

    Ok(Transformation::new(
        transformation0.input_domain.clone(),
        transformation1.output_domain.clone(),
        Function::make_chain(&transformation1.function, &transformation0.function),
        transformation0.input_metric.clone(),
        transformation1.output_metric.clone(),
        StabilityRelation::make_chain(&transformation1.stability_relation,&transformation0.stability_relation, hint)
    ))
}

pub fn make_basic_composition<DI, DO0, DO1, MI, MO>(measurement0: &Measurement<DI, DO0, MI, MO>, measurement1: &Measurement<DI, DO1, MI, MO>) -> Fallible<Measurement<DI, PairDomain<DO0, DO1>, MI, MO>>
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MO: 'static + Measure {
    if measurement0.input_domain != measurement1.input_domain {
        return fallible!(DomainMismatch, "Input domain mismatch");
    } else if measurement0.input_metric != measurement1.input_metric {
        return fallible!(MetricMismatch, "Input metric mismatch");
    } else if measurement0.output_measure != measurement1.output_measure {
        return fallible!(MeasureMismatch, "Output measure mismatch");
    }

    Ok(Measurement::new(
        measurement0.input_domain.clone(),
        PairDomain::new(measurement0.output_domain.clone(), measurement1.output_domain.clone()),
        Function::make_basic_composition(&measurement0.function, &measurement1.function),
        measurement0.input_metric.clone(),
        measurement0.output_measure.clone(),
        // TODO: PrivacyRelation for make_composition
        PrivacyRelation::new(|_i, _o| false),
    ))
}


pub fn bounded_complexity_composition_privacy_relation <MI, Q>(
    relation1: &PrivacyRelation<MI, FSmoothedMaxDivergence<Q>>,
    relation2: &PrivacyRelation<MI, FSmoothedMaxDivergence<Q>>,
    npoints: u8,
    delta_min: Q,
) -> PrivacyRelation<MI, FSmoothedMaxDivergence<Q>>
    where MI: Metric,
          Q: 'static + One + Zero + PartialOrd + CastRational + CastInternalReal + Clone + Debug + Float + Midpoint + Tolerance,
          MI::Distance: Clone + One + Zero + PartialOrd {

    let probas_ratios1 = ProbabilitiesRatios::from_privacy_relation(relation1, npoints.clone(), delta_min.clone());
    if probas_ratios1.len() == 0 {
        return relation2.clone()
    }
    let probas_ratios2 = ProbabilitiesRatios::from_privacy_relation(relation2, npoints.clone(), delta_min.clone());
    if probas_ratios2.len() == 0 {
        return relation1.clone()
    }

    let compo_probas_ratios = probas_ratios1.compose(&probas_ratios2);
    let compo_alphas_betas = compo_probas_ratios.to_alphas_betas();

    PrivacyRelation::new_fallible(move |d_in: &MI::Distance, d_out: &Vec<EpsilonDelta<Q>>| {
        if d_in <= &MI::Distance::zero() {
            return fallible!(InvalidDistance, "input sensitivity must be non-negative")
        }

        let mut result = true;
        for EpsilonDelta { epsilon, delta } in d_out {
            if epsilon <= &Q::zero() {
                return fallible!(InvalidDistance, "epsilon must be positive or 0")
            }
            if delta <= &Q::zero() {
                return fallible!(InvalidDistance, "delta must be positive or 0")
            }

            let delta_dual = compo_alphas_betas.to_delta(epsilon.clone().exp().into_rational());
            result = result & (delta >= &Q::from_rational(delta_dual));
            if result == false {
                break;
            }
        }
        Ok(result)
    })
}

pub fn make_bounded_complexity_composition<DI, DO0, DO1, MI, Q>(
    measurement0: &Measurement<DI, DO0, MI, FSmoothedMaxDivergence<Q>>,
    measurement1: &Measurement<DI, DO1, MI, FSmoothedMaxDivergence<Q>>,
    npoints: u8,
    delta_min: Q,
) -> Fallible<Measurement<DI, PairDomain<DO0, DO1>, MI, FSmoothedMaxDivergence<Q>>>
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MI::Distance: Clone + One + Zero + PartialOrd,
          Q:'static + One + Zero + PartialOrd + CastRational + CastInternalReal + Clone + Debug + Float + Midpoint + Tolerance {

    if measurement0.input_domain != measurement1.input_domain {
        return fallible!(DomainMismatch, "Input domain mismatch");
    } else if measurement0.input_metric != measurement1.input_metric {
        return fallible!(MetricMismatch, "Input metric mismatch");
    } else if measurement0.output_measure != measurement1.output_measure {
        return fallible!(MeasureMismatch, "Output measure mismatch");
    }


    Ok(Measurement::new(
        measurement0.input_domain.clone(),
        PairDomain::new(measurement0.output_domain.clone(), measurement1.output_domain.clone()),
        Function::make_basic_composition(&measurement0.function, &measurement1.function), // TODO: check that
        measurement0.input_metric.clone(),
        measurement0.output_measure.clone(),
        bounded_complexity_composition_privacy_relation(
            &measurement0.privacy_relation,
            &measurement1.privacy_relation,
            npoints,
            delta_min,
        )
    ))
}

pub fn make_bounded_complexity_composition_multi<DI, DO, MI, Q>(
    measurements: &Vec<&'static Measurement<DI, DO, MI, FSmoothedMaxDivergence<Q>>>,
    npoints: u8,
    delta_min: Q,
) -> Fallible<Measurement<DI, VectorDomain<DO>, MI, FSmoothedMaxDivergence<Q>>>
    where DI: 'static + Domain + Clone,
          DO: 'static + Domain + Clone,
          MI: 'static + Metric,
          MI::Distance: Clone + One + Zero + PartialOrd,
          Q:'static + One + Zero + PartialOrd + CastRational + CastInternalReal + Clone + Debug + Float + Midpoint + Tolerance {

    if measurements.is_empty() {
        return fallible!(MakeMeasurement, "Must have at least one measurement")
    }

    if !measurements.iter().all(|v| measurements[0].input_domain.clone() == v.input_domain) {
        return fallible!(DomainMismatch, "Input domain mismatch");
    } else if !measurements.iter().all(|v| measurements[0].input_metric.clone() == v.input_metric) {
        return fallible!(MetricMismatch, "Input metric mismatch");
    } else if !measurements.iter().all(|v| measurements[0].output_domain.clone() == v.output_domain) {
        return fallible!(DomainMismatch, "Output domain mismatch");
    } else if !measurements.iter().all(|v| measurements[0].output_measure.clone() == v.output_measure) {
        return fallible!(MeasureMismatch, "Output measure mismatch");
    }

    let mut functions = Vec::new();
    let mut composed_privacy_relation = measurements[0].privacy_relation.clone();

    for measurement in measurements {
        functions.push(&measurement.function);
        composed_privacy_relation = bounded_complexity_composition_privacy_relation(
            &composed_privacy_relation,
            &measurement.privacy_relation,
            npoints,
            delta_min,
        );
    }

    Ok(Measurement::new(
        measurements[0].input_domain.clone(),
        VectorDomain::new(measurements[0].output_domain.clone()),
        Function::new_fallible(move |arg| functions.iter().map(|f| f.eval(arg)).collect()),
        measurements[0].input_metric.clone(),
        measurements[0].output_measure.clone(),
        composed_privacy_relation,
    ))
}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::dist::{L1Distance, MaxDivergence};
    use crate::dom::AllDomain;
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_make_chain_mt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = L1Distance::<i32>::default();
        let output_metric0 = L1Distance::<i32>::default();
        let stability_relation0 = StabilityRelation::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_relation1 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let chain = make_chain_mt(&measurement1, &transformation0, None).unwrap_test();
        let arg = 99_u8;
        let ret = chain.function.eval(&arg).unwrap_test();
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_tt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = L1Distance::<i32>::default();
        let output_metric0 = L1Distance::<i32>::default();
        let stability_relation0 = StabilityRelation::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_metric1 = L1Distance::<i32>::default();
        let stability_relation1 = StabilityRelation::new_from_constant(1);
        let transformation1 = Transformation::new(input_domain1, output_domain1, function1, input_metric1, output_metric1, stability_relation1);
        let chain = make_chain_tt(&transformation1, &transformation0, None).unwrap_test();
        let arg = 99_u8;
        let ret = chain.function.eval(&arg).unwrap_test();
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_basic_composition() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f32>::new();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f32);
        let input_metric0 = L1Distance::<i32>::default();
        let output_measure0 = MaxDivergence::default();
        let privacy_relation0 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement0 = Measurement::new(input_domain0, output_domain0, function0, input_metric0, output_measure0, privacy_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_relation1 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let composition = make_basic_composition(&measurement0, &measurement1).unwrap_test();
        let arg = 99;
        let ret = composition.function.eval(&arg).unwrap_test();
        assert_eq!(ret, (100_f32, 98_f64));
    }

    #[test]
    fn test_make_bounded_complexity_composition() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f32>::new();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f32);
        let input_metric0 = L1Distance::<i32>::default();
        let output_measure0 = FSmoothedMaxDivergence::default();
        let privacy_relation0 = PrivacyRelation::new(|_d_in: &i32, _d_out: &Vec<EpsilonDelta<f64>>| true);
        let measurement0 = Measurement::new(input_domain0, output_domain0, function0, input_metric0, output_measure0, privacy_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_measure1 = FSmoothedMaxDivergence::default();
        let privacy_relation1 = PrivacyRelation::new(|_d_in: &i32, _d_out: &Vec<EpsilonDelta<f64>>| true);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let npoints: u8 = 3;
        let delta_min: f64 = 1e-5;
        let composition = make_bounded_complexity_composition(&measurement0, &measurement1, npoints, delta_min).unwrap_test();
        let arg = 99;
        let ret = composition.function.eval(&arg).unwrap_test();
        assert_eq!(ret, (100_f32, 98_f64));
    }
}


impl<DI, DX, DO, MI, MX, MO> Shr<Measurement<DX, DO, MX, MO>> for Transformation<DI, DX, MI, MX>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    type Output = Fallible<Measurement<DI, DO, MI, MO>>;

    fn shr(self, rhs: Measurement<DX, DO, MX, MO>) -> Self::Output {
        make_chain_mt(&rhs, &self, None)
    }
}

impl<DI, DX, DO, MI, MX, MO> Shr<Measurement<DX, DO, MX, MO>> for Fallible<Transformation<DI, DX, MI, MX>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    type Output = Fallible<Measurement<DI, DO, MI, MO>>;

    fn shr(self, rhs: Measurement<DX, DO, MX, MO>) -> Self::Output {
        make_chain_mt(&rhs, &self?, None)
    }
}

impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, DO, MX, MO>> for Transformation<DI, DX, MI, MX>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MX, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self, None)
    }
}

impl<DI, DX, DO, MI, MX, MO> Shr<Transformation<DX, DO, MX, MO>> for Fallible<Transformation<DI, DX, MI, MX>>
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    type Output = Fallible<Transformation<DI, DO, MI, MO>>;

    fn shr(self, rhs: Transformation<DX, DO, MX, MO>) -> Self::Output {
        make_chain_tt(&rhs, &self?, None)
    }
}


#[cfg(test)]
mod tests_shr {
    use crate::meas::geometric::make_base_geometric;
    use crate::trans::{make_bounded_sum, make_cast_default, make_clamp, make_split_lines};

    use super::*;

    #[test]
    fn test_shr() -> Fallible<()> {
        (
            make_split_lines()? >>
            make_cast_default()? >>
            make_clamp(0, 1)? >>
            make_bounded_sum(0, 1)? >>
            make_base_geometric(1., Some((0, 10)))?
        ).map(|_| ())
    }
}

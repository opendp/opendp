#[cfg(feature="ffi")]
pub mod ffi;

use std::ops::Shr;

use num::Zero;

use crate::core::{Domain, Function, HintMt, HintTt, Measure, Measurement, Metric, PrivacyRelation, StabilityRelation, Transformation};
use crate::dist::{MaxDivergence, SmoothedMaxDivergence};
use crate::dom::{PairDomain, VectorDomain};
use crate::error::Fallible;
use std::fmt::Debug;
use crate::traits::InfAdd;

const ERROR_URL: &str = "https://github.com/opendp/opendp/discussions/297";

fn mismatch_message<T1: Debug, T2: Debug>(mode: &str, struct1: &T1, struct2: &T2) -> String {
    let str1 = format!("{:?}", struct1);
    let str2 = format!("{:?}", struct2);
    let explanation = if str1 == str2 {
        format!("\n    The structure of the intermediate {mode}s are the same, but the types or parameters differ.\n    shared_{mode}: {str1}\n", mode=mode, str1=str1)
    } else {
        format!("\n    output_{mode}: {struct1}\n    input_{mode}:  {struct2}\n", mode=mode, struct1=str1, struct2=str2)
    };
    return format!("Intermediate {}s don't match. See {}{}", mode, ERROR_URL, explanation)
}

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
        return fallible!(DomainMismatch, mismatch_message("domain", &transformation0.output_domain, &measurement1.input_domain))
    }
    if transformation0.output_metric != measurement1.input_metric {
        return fallible!(MetricMismatch, mismatch_message("metric", &transformation0.output_metric, &measurement1.input_metric))
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
        return fallible!(DomainMismatch, mismatch_message("domain", &transformation0.output_domain, &transformation1.input_domain))
    }
    if transformation0.output_metric != transformation1.input_metric {
        return fallible!(MetricMismatch, mismatch_message("metric", &transformation0.output_metric, &transformation1.input_metric))
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

pub trait ComposableMeasure: Measure {
    fn compose(&self, d_i: &Vec<Self::Distance>)
        -> Fallible<Self::Distance>;
}

impl<Q: InfAdd + Zero + Clone> ComposableMeasure for MaxDivergence<Q> {
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(
            Q::zero(),
            |sum, d_i| sum.inf_add(d_i))
    }
}

impl<Q: InfAdd + Zero + Clone> ComposableMeasure for SmoothedMaxDivergence<Q> {
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(
            (Q::zero(), Q::zero()),
            |(e1, d1), (e2, d2)| Ok((e1.inf_add(e2)?, d1.inf_add(d2)?)))
    }
}

pub fn make_sequential_composition_static_distances<DI, DO, MI, MO>(
    d_in: MI::Distance,
    measurement_pairs: Vec<(&Measurement<DI, DO, MI, MO>, MO::Distance)>
) -> Fallible<Measurement<DI, VectorDomain<DO>, MI, MO>>
    where DI: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MI::Distance: 'static + PartialOrd,
          MO: 'static + ComposableMeasure,
          MO::Distance: 'static + PartialOrd {

    if measurement_pairs.is_empty() {
        return fallible!(MakeMeasurement, "Must have at least one measurement")
    }

    for (measurement, d_mid) in &measurement_pairs {
        if !measurement.privacy_relation.eval(&d_in, d_mid)? {
            return fallible!(MakeMeasurement, "one of the relations does not pass with its respective d_mid");
        }
    }

    let (measurements, d_mids): (Vec<_>, Vec<_>) =
        measurement_pairs.into_iter().unzip();

    let input_domain = measurements[0].input_domain.clone();
    let output_domain = measurements[0].output_domain.clone();
    let input_metric = measurements[0].input_metric.clone();
    let output_measure = measurements[0].output_measure.clone();

    if !measurements.iter().all(|v| input_domain == v.input_domain) {
        return fallible!(DomainMismatch, "All input domains must be the same");
    }
    if !measurements.iter().all(|v| output_domain == v.output_domain) {
        return fallible!(DomainMismatch, "All output domains must be the same");
    }
    if !measurements.iter().all(|v| input_metric == v.input_metric) {
        return fallible!(MetricMismatch, "All input metrics must be the same");
    }
    if !measurements.iter().all(|v| output_measure == v.output_measure) {
        return fallible!(MetricMismatch, "All output measures must be the same");
    }

    let functions = measurements.iter()
        .map(|m| m.function.clone()).collect::<Vec<_>>();
    let d_out = output_measure.compose(&d_mids)?;

    Ok(Measurement::new(
        input_domain,
        VectorDomain::new(output_domain),
        Function::new_fallible(move |arg: &DI::Carrier|
            functions.iter().map(|f| f.eval(arg)).collect()),
        input_metric,
        output_measure.clone(),
        PrivacyRelation::new(move |d_in_prime: &MI::Distance, d_out_prime: &MO::Distance|
            d_in_prime <= &d_in && &d_out <= d_out_prime)
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


// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::dist::{L1Distance, MaxDivergence};
    use crate::dom::AllDomain;
    use crate::error::ExplainUnwrap;
    use crate::meas::make_base_laplace;

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
        let ret = chain.invoke(&arg).unwrap_test();
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
        let ret = chain.invoke(&arg).unwrap_test();
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
        let ret = composition.invoke(&arg).unwrap_test();
        assert_eq!(ret, (100_f32, 98_f64));
    }

    #[test]
    fn test_make_sequential_composition_static_distances() -> Fallible<()> {
        let laplace = make_base_laplace::<AllDomain<_>>(1.)?;
        let measurements = vec![
            (&laplace, 1.),
            (&laplace, 1.),
        ];
        let composition = make_sequential_composition_static_distances(1., &measurements)?;
        let arg = 99.;
        let ret = composition.function.eval(&arg)?;

        assert_eq!(ret.len(), 2);

        assert!(composition.privacy_relation.eval(&1., &2.)?);
        assert!(composition.privacy_relation.eval(&1., &2.0001)?);
        assert!(!composition.privacy_relation.eval(&1., &1.999)?);
        Ok(())
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
            make_clamp((0, 1))? >>
            make_bounded_sum((0, 1))? >>
            make_base_geometric(1., Some((0, 10)))?
        ).map(|_| ())
    }
}

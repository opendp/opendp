#[cfg(feature="ffi")]
pub mod ffi;

use std::ops::Shr;

use num::Zero;

use crate::core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap, StabilityMap, Transformation, MaxDivergence, VectorDomain, FixedSmoothedMaxDivergence, ZeroConcentratedDivergence};
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
    transformation0: &Transformation<DI, DX, MI, MX>
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
        PrivacyMap::make_chain(&measurement1.privacy_map,&transformation0.stability_map)
    ))
}

pub fn make_chain_tt<DI, DX, DO, MI, MX, MO>(
    transformation1: &Transformation<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
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
        StabilityMap::make_chain(&transformation1.stability_map, &transformation0.stability_map)
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

impl<Q: InfAdd + Zero + Clone> ComposableMeasure for FixedSmoothedMaxDivergence<Q> {
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(
            (Q::zero(), Q::zero()),
            |(e1, d1), (e2, d2)| Ok((e1.inf_add(e2)?, d1.inf_add(d2)?)))
    }
}

impl<Q: InfAdd + Zero + Clone> ComposableMeasure for ZeroConcentratedDivergence<Q> {
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(
            Q::zero(),
            |sum, d_i| sum.inf_add(d_i))
    }
}

pub fn make_sequential_composition_static_distances<DI, DO, MI, MO>(
    measurements: Vec<&Measurement<DI, DO, MI, MO>>
) -> Fallible<Measurement<DI, VectorDomain<DO>, MI, MO>>
    where DI: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MO: 'static + ComposableMeasure {

    if measurements.is_empty() {
        return fallible!(MakeMeasurement, "Must have at least one measurement")
    }
    let input_domain = measurements[0].input_domain.clone();
    let output_domain = measurements[0].output_domain.clone();
    let input_metric = measurements[0].input_metric.clone();
    let output_measure = measurements[0].output_measure.clone();

    println!("checking all measurements match");
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
    println!("collecting functions and relations");

    let functions = measurements.iter()
        .map(|m| m.function.clone()).collect::<Vec<_>>();

    let maps = measurements.iter()
        .map(|m| m.privacy_map.clone()).collect::<Vec<_>>();

    println!("making measurement");
    Ok(Measurement::new(
        input_domain,
        VectorDomain::new(output_domain),
        Function::new_fallible(move |arg: &DI::Carrier|
            functions.iter().map(|f| f.eval(arg)).collect()),
        input_metric,
        output_measure.clone(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance|
            output_measure.compose(&maps.iter()
                .map(|map| map.eval(d_in))
                .collect::<Fallible<_>>()?))
    ))
}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::core::{L1Distance, MaxDivergence};
    use crate::core::AllDomain;
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
        let stability_relation0 = StabilityMap::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_relation1 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let chain = make_chain_mt(&measurement1, &transformation0).unwrap_test();
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
        let stability_relation0 = StabilityMap::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_metric1 = L1Distance::<i32>::default();
        let stability_relation1 = StabilityMap::new_from_constant(1);
        let transformation1 = Transformation::new(input_domain1, output_domain1, function1, input_metric1, output_metric1, stability_relation1);
        let chain = make_chain_tt(&transformation1, &transformation0).unwrap_test();
        let arg = 99_u8;
        let ret = chain.invoke(&arg).unwrap_test();
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_sequential_composition_static_distances() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f64>::new();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f64);
        let input_metric0 = L1Distance::<i32>::default();
        let output_measure0 = MaxDivergence::default();
        let privacy_relation0 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement0 = Measurement::new(input_domain0, output_domain0, function0, input_metric0, output_measure0, privacy_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_relation1 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let composition = make_sequential_composition_static_distances(vec![&measurement0, &measurement1]).unwrap_test();
        let arg = 99;
        let ret = composition.invoke(&arg).unwrap_test();
        assert_eq!(ret, vec![100_f64, 98_f64]);
    }

    #[test]
    fn test_make_sequential_composition_static_distances_2() -> Fallible<()> {
        let laplace = make_base_laplace::<AllDomain<_>>(1.0f64)?;
        let measurements = vec![&laplace; 2];
        let composition = make_sequential_composition_static_distances(measurements)?;
        let arg = 99.;
        let ret = composition.function.eval(&arg)?;

        assert_eq!(ret.len(), 2);

        assert!(composition.check(&1., &2.)?);
        assert!(composition.check(&1., &2.0001)?);
        assert!(!composition.check(&1., &1.999)?);
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
        make_chain_mt(&rhs, &self)
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
        make_chain_mt(&rhs, &self?)
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
        make_chain_tt(&rhs, &self)
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
        make_chain_tt(&rhs, &self?)
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

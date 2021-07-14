use std::ops::{Shr, Sub, Div};

use crate::core::{Domain, Function, HintMt, HintTt, Measure, Measurement, Metric, PrivacyRelation, StabilityRelation, Transformation};
use crate::dom::{PairDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{MetricDistance, MeasureDistance, Midpoint, FallibleSub, Tolerance};
use num::{Zero, One};
use crate::dist::{MaxDivergence, SmoothedMaxDivergence, EpsilonDelta};

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

pub trait BasicComposition<MI: Metric>: Measure {
    fn basic_composition(
        &self,
        relations: &Vec<PrivacyRelation<MI, Self>>,
        d_in: &MI::Distance, d_out: &Self::Distance
    ) -> Fallible<bool>;
}

pub trait BasicCompositionDistance: MeasureDistance + Clone + Midpoint + Zero + Tolerance {
    type Atom: Sub<Output=Self::Atom> + One + Div<Output=Self::Atom>;
}

impl BasicCompositionDistance for f64 {type Atom = f64;}
impl BasicCompositionDistance for f32 {type Atom = f64;}
impl<T> BasicCompositionDistance for EpsilonDelta<T>
    where T: for<'a> Sub<&'a T, Output=T> + Sub<Output=T> + One + Div<Output=T> + PartialOrd + Tolerance + Zero + Clone {
    type Atom = T;
}

// impl<Q: MeasureDistance + Clone + FallibleSub<Output=Q> + Midpoint + Zero + Tolerance + PartialOrd> BasicCompositionDistance for Q {}

impl<MI: Metric, Q: Clone> BasicComposition<MI> for MaxDivergence<Q>
    where Q: BasicCompositionDistance<Atom=Q>,
          MI::Distance: Clone {
    fn basic_composition(
        &self, relations: &Vec<PrivacyRelation<MI, Self>>,
        d_in: &MI::Distance, d_out: &Self::Distance
    ) -> Fallible<bool> {
        basic_composition(relations, d_in, d_out)
    }
}
impl<MI: Metric, Q: Clone> BasicComposition<MI> for SmoothedMaxDivergence<Q>
    where EpsilonDelta<Q>: BasicCompositionDistance<Atom=Q>,
          MI::Distance: Clone {
    fn basic_composition(
        &self, relations: &Vec<PrivacyRelation<MI, Self>>,
        d_in: &MI::Distance, d_out: &Self::Distance
    ) -> Fallible<bool> {
        basic_composition(relations, d_in, d_out)
    }
}

fn basic_composition<MI: Metric, MO: Measure>(
    relations: &Vec<PrivacyRelation<MI, MO>>,
    d_in: &MI::Distance,
    d_out: &MO::Distance
) -> Fallible<bool>
    where MO::Distance: BasicCompositionDistance,
          MI::Distance: Clone {
    let mut d_out = d_out.clone();

    for relation in relations {
        if let Some(usage) = basic_composition_binary_search(
            d_in.clone(), d_out.clone(), relation)? {

            d_out = d_out.sub(&usage)?;
        } else {
            return Ok(false)
        }
    }
    Ok(true)
}
pub fn make_basic_composition_multi<DI, DO, MI, MO>(
    measurements: &Vec<&Measurement<DI, DO, MI, MO>>
) -> Fallible<Measurement<DI, VectorDomain<DO>, MI, MO>>
    where DI: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MI::Distance: 'static + MetricDistance + Clone,
          MO: 'static + Measure + BasicComposition<MI>,
          MO::Distance: 'static + MeasureDistance + Clone {

    if measurements.is_empty() {
        return fallible!(MakeMeasurement, "Must have at least one measurement")
    }

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

    let mut functions = Vec::new();
    let mut relations = Vec::new();
    for measurement in measurements {
        functions.push(measurement.function.clone());
        relations.push(measurement.privacy_relation.clone());
    }

    Ok(Measurement::new(
        input_domain,
        VectorDomain::new(output_domain),
        Function::new_fallible(move |arg: &DI::Carrier|
            functions.iter().map(|f| f.eval(arg)).collect()),
        input_metric,
        output_measure.clone(),
        PrivacyRelation::new_fallible(move |d_in: &MI::Distance, d_out: &MO::Distance| {
            output_measure.basic_composition(&relations, d_in, d_out)
        })
    ))
}

const MAX_ITERATIONS: usize = 100;

fn basic_composition_binary_search<MI, MO>(
    d_in: MI::Distance, mut d_out: MO::Distance,
    predicate: &PrivacyRelation<MI, MO>
) -> Fallible<Option<MO::Distance>>
    where MI: Metric,
          MO: Measure,
          MO::Distance: Midpoint + Zero + Clone + Tolerance + PartialOrd {

    // d_out is d_max, we use binary search to reduce d_out
    // to the smallest value that still passes the relation
    if !predicate.eval(&d_in, &d_out)? {
        return Ok(None)
    }

    let mut d_min = MO::Distance::zero();
    for _ in 0..MAX_ITERATIONS {
        let d_mid = d_min.clone().midpoint(d_out.clone());

        if predicate.eval(&d_in, &d_mid)? {
            d_out = d_mid;
        } else {
            d_min = d_mid;
        }
        if d_out <= MO::Distance::TOLERANCE + d_min.clone() { return Ok(Some(d_out)) }
    }
    Ok(Some(d_out))
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

    use super::*;
    use crate::meas::make_base_laplace;

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
    fn test_make_basic_composition_multi() -> Fallible<()> {
        let measurements = vec![
            make_base_laplace::<AllDomain<_>>(0.)?,
            make_base_laplace(0.)?
        ];
        let composition = make_basic_composition_multi(&measurements.iter().collect())?;
        let arg = 99.;
        let ret = composition.function.eval(&arg)?;

        assert_eq!(ret.len(), 2);
        assert_eq!(ret, vec![99., 99.]);

        let measurements = vec![
            make_base_laplace::<AllDomain<_>>(1.)?,
            make_base_laplace(1.)?
        ];
        let composition = make_basic_composition_multi(&measurements.iter().collect())?;
        // runs once because it sits on a power of 2
        assert!(composition.privacy_relation.eval(&1., &2.)?);
        // runs a few steps- it will tighten to within TOLERANCE of 1 on the first measurement
        assert!(composition.privacy_relation.eval(&1., &2.0001)?);
        // should fail
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
            make_clamp(0, 1)? >>
            make_bounded_sum(0, 1)? >>
            make_base_geometric(1., Some((0, 10)))?
        ).map(|_| ())
    }
}

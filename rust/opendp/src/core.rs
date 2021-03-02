//! Core concepts of OpenDP.
//!
//! This module provides the central building blocks used throughout OpenDP:
//! * Measurement
//! * Transformation
//! * Domain
//! * Metric/Measure
//! * Function
//! * PrivacyRelation/StabilityRelation

// Generic legend
// M: Metric and Measure
// Q: Metric and Measure Carrier/Distance
// D: Domain
// T: Domain Carrier

// *I: Input
// *O: Output

use std::rc::Rc;

use crate::dom::{BoxDomain, PairDomain};
use crate::meas::{MakeMeasurement2, MakeMeasurement3};
use crate::trans::MakeTransformation2;

/// A set which constrains the input or output of a [`Function`].
///
/// Domains capture the notion of what values are allowed to be the input or output of a `Function`.
pub trait Domain: Clone + PartialEq {
    /// The underlying type that the Domain specializes.
    type Carrier;
    /// Predicate to test an element for membership in the domain.
    fn member(&self, val: &Self::Carrier) -> bool;
}

/// A mathematical function which maps values from an input [`Domain`] to an output [`Domain`].
#[derive(Clone)]
pub struct Function<DI: Domain, DO: Domain> {
    pub function: Rc<dyn Fn(&DI::Carrier) -> Box<DO::Carrier>>
}

impl<DI: Domain, DO: Domain> Function<DI, DO> {
    pub fn new(function: impl Fn(&DI::Carrier) -> DO::Carrier + 'static) -> Self {
        let function = move |arg: &DI::Carrier| {
            let res = function(arg);
            Box::new(res)
        };
        let function = Rc::new(function);
        Function { function }
    }

    pub fn eval(&self, arg: &DI::Carrier) -> DO::Carrier {
        *(self.function)(arg)
    }

    pub fn eval_ffi(&self, arg: &DI::Carrier) -> Box<DO::Carrier> {
        (self.function)(arg)
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain> Function<DI, DO> {
    pub fn make_chain<XD: 'static + Domain>(function1: &Function<XD, DO>, function0: &Function<DI, XD>) -> Function<DI, DO> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: &DI::Carrier| {
            let res0 = function0(arg);
            function1(&res0)
        };
        let function = Rc::new(function);
        Function { function }
    }
}

impl<DI: 'static + Domain, DO1: 'static + Domain, DO2: 'static + Domain> Function<DI, PairDomain<BoxDomain<DO1>, BoxDomain<DO2>>> {
    pub fn make_composition(function0: &Function<DI, DO1>, function1: &Function<DI, DO2>) -> Self {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: & DI::Carrier| {
            let res0 = function0(arg);
            let res1 = function1(arg);
            Box::new((res0, res1))
        };
        let function = Rc::new(function);
        Function { function }
    }
}

/// A representation of the distance between two elements in a set.
pub trait Metric: Clone {
    type Distance;
}

/// A representation of the distance between two distributions.
pub trait Measure: Clone {
    type Distance;
}

/// A indicator trait that is only implemented for dataset distance
pub trait DatasetMetric: Metric { fn new() -> Self; }
pub trait SensitivityMetric: Metric { fn new() -> Self; }

/// A boolean relation evaluating the privacy of a [`Measurement`].
///
/// A `PrivacyRelation` is implemented as a function that takes an input [`Metric::Distance`] and output [`Measure::Distance`],
/// and returns a boolean indicating if the relation holds.
#[derive(Clone)]
pub struct PrivacyRelation<MI: Metric, MO: Measure> {
    pub relation: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> bool>
}
impl<MI: Metric, MO: Measure> PrivacyRelation<MI, MO> {
    pub fn new(relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static) -> Self {
        let relation = Rc::new(relation);
        PrivacyRelation { relation }
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> bool {
        (self.relation)(input_distance, output_distance)
    }
}

/// A boolean relation evaluating the stability of a [`Transformation`].
///
/// A `StabilityRelation` is implemented as a function that takes an input and output [`Metric::Distance`],
/// and returns a boolean indicating if the relation holds.
#[derive(Clone)]
pub struct StabilityRelation<MI: Metric, MO: Metric> {
    pub relation: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> bool>
}
impl<MI: Metric, MO: Metric> StabilityRelation<MI, MO> {
    pub fn new(relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static) -> Self {
        let relation = Rc::new(relation);
        StabilityRelation { relation }
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> bool {
        (self.relation)(input_distance, output_distance)
    }
}


/// A randomized mechanism with certain privacy characteristics.
pub struct Measurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    pub input_domain: Box<DI>,
    pub output_domain: Box<DO>,
    pub function: Function<DI, DO>,
    pub input_metric: Box<MI>,
    pub output_measure: Box<MO>,
    pub privacy_relation: PrivacyRelation<MI, MO>,
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> Measurement<DI, DO, MI, MO> {
    pub fn new(
        input_domain: DI,
        output_domain: DO,
        function: impl Fn(&DI::Carrier) -> DO::Carrier + 'static,
        input_metric: MI,
        output_measure: MO,
        privacy_relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static,
    ) -> Self {
        let input_domain = Box::new(input_domain);
        let output_domain = Box::new(output_domain);
        let function = Function::new(function);
        let input_metric = Box::new(input_metric);
        let output_measure = Box::new(output_measure);
        let privacy_relation = PrivacyRelation::new(privacy_relation);
        Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
    }
}

/// A data transformation with certain stability characteristics.
pub struct Transformation<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
    pub input_domain: Box<DI>,
    pub output_domain: Box<DO>,
    pub function: Function<DI, DO>,
    pub input_metric: Box<MI>,
    pub output_metric: Box<MO>,
    pub stability_relation: StabilityRelation<MI, MO>,
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> Transformation<DI, DO, MI, MO> {
    pub fn new(
        input_domain: DI,
        output_domain: DO,
        function: impl Fn(&DI::Carrier) -> DO::Carrier + 'static,
        input_metric: MI,
        output_metric: MO,
        stability_relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static,
    ) -> Self {
        Transformation {
            input_domain: Box::new(input_domain),
            output_domain: Box::new(output_domain),
            function: Function::new(function),
            input_metric: Box::new(input_metric),
            output_metric: Box::new(output_metric),
            stability_relation: StabilityRelation::new(stability_relation)
        }
    }
}


// GLUE FOR FFI USE OF COMBINATORS
fn new_clone<T: Clone>() -> Rc<dyn Fn(&Box<T>) -> Box<T>> {
    let clone = |t: &Box<T>| t.clone();
    Rc::new(clone)
}

fn new_domain_glue<D: Domain>() -> (Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>, Rc<dyn Fn(&Box<D>) -> Box<D>>) {
    let eq = |d0: &Box<D>, d1: &Box<D>| d0 == d1;
    let eq = Rc::new(eq);
    let clone = new_clone();
    (eq, clone)
}

/// Public only for access from FFI.
#[derive(Clone)]
pub struct MeasureGlue<D: Domain, M: Measure> {
    pub domain_eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>,
    pub domain_clone: Rc<dyn Fn(&Box<D>) -> Box<D>>,
    pub measure_clone: Rc<dyn Fn(&Box<M>) -> Box<M>>,
}
impl<D: 'static + Domain, M: 'static + Measure> MeasureGlue<D, M> {
    pub fn new() -> Self {
        let (domain_eq, domain_clone) = new_domain_glue();
        let measure_clone = new_clone();
        MeasureGlue { domain_eq, domain_clone, measure_clone }
    }
}

/// Public only for access from FFI.
#[derive(Clone)]
pub struct MetricGlue<D: Domain, M: Metric> {
    pub domain_eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>,
    pub domain_clone: Rc<dyn Fn(&Box<D>) -> Box<D>>,
    pub metric_clone: Rc<dyn Fn(&Box<M>) -> Box<M>>,
}
impl<D: 'static + Domain, M: 'static + Metric> MetricGlue<D, M> {
    pub fn new() -> Self {
        let (domain_eq, domain_clone) = new_domain_glue();
        let metric_clone = new_clone();
        MetricGlue { domain_eq, domain_clone, metric_clone }
    }
}


// CHAINING & COMPOSITION
pub struct ChainMT;

impl<DI, DX, DO, MI, MX, MO> MakeMeasurement2<DI, DO, MI, MO, &Measurement<DX, DO, MX, MO>, &Transformation<DI, DX, MI, MX>> for ChainMT
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    fn make2(measurement1: &Measurement<DX, DO, MX, MO>, transformation0: &Transformation<DI, DX, MI, MX>) -> Measurement<DI, DO, MI, MO> {
        let input_glue = MetricGlue::<DI, MI>::new();
        let x_glue = MetricGlue::<DX, MX>::new();
        let output_glue = MeasureGlue::<DO, MO>::new();
        make_chain_mt_glue(measurement1, transformation0, &input_glue, &x_glue, &output_glue)
    }
}


pub fn make_chain_mt_glue<DI, DX, DO, MI, MX, MO>(
    measurement1: & Measurement<DX, DO, MX, MO>,
    transformation0: & Transformation<DI, DX, MI, MX>,
    input_glue: &MetricGlue<DI, MI>,
    x_glue: &MetricGlue<DX, MX>,
    output_glue: &MeasureGlue<DO, MO>
) -> Measurement<DI, DO, MI, MO> where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure {
    assert!((x_glue.domain_eq)(&transformation0.output_domain, &measurement1.input_domain));
    let input_domain = (input_glue.domain_clone)(&transformation0.input_domain);
    let output_domain = (output_glue.domain_clone)(&measurement1.output_domain);
    let function = Function::make_chain(&measurement1.function, &transformation0.function);
    let input_metric = (input_glue.metric_clone)(&transformation0.input_metric);
    let output_measure = (output_glue.measure_clone)(&measurement1.output_measure);
    // TODO: PrivacyRelation for make_chain_mt
    let privacy_relation = PrivacyRelation::new(|_d_in: &MI::Distance, _d_out: &MO::Distance| false);
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}

impl<DI, DX, DO, MI, MX, MO, F> MakeMeasurement3<DI, DO, MI, MO, & Measurement<DX, DO, MX, MO>, & Transformation<DI, DX, MI, MX>, F> for ChainMT
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure,
        MI::Distance: Clone,
    MO::Distance: Clone,
    MX::Distance: Clone,
          F: 'static + Fn(&MI::Distance, &MO::Distance) -> MX::Distance {
    fn make3(measurement1: &Measurement<DX, DO, MX, MO>, transformation0: &Transformation<DI, DX, MI, MX>, hint: F) -> Measurement<DI, DO, MI, MO> {
        let input_glue = MetricGlue::<DI, MI>::new();
        let x_glue = MetricGlue::<DX, MX>::new();
        let output_glue = MeasureGlue::<DO, MO>::new();
        make_chain_mt_glue_hint(measurement1, transformation0, &input_glue, &x_glue, &output_glue, hint)
    }
}

pub fn make_chain_mt_glue_hint<DI, DX, DO, MI, MX, MO, F>(
    measurement1: &Measurement<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
    input_glue: &MetricGlue<DI, MI>,
    x_glue: &MetricGlue<DX, MX>,
    output_glue: &MeasureGlue<DO, MO>,
    hint: F
) -> Measurement<DI, DO, MI, MO> where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure,
    MI::Distance: Clone,
    MO::Distance: Clone,
    MX::Distance: Clone,
    F: 'static + Fn(&MI::Distance, &MO::Distance) -> MX::Distance {
    assert!((x_glue.domain_eq)(&transformation0.output_domain, &measurement1.input_domain));
    let input_domain = (input_glue.domain_clone)(&transformation0.input_domain);
    let output_domain = (output_glue.domain_clone)(&measurement1.output_domain);
    let function = Function::make_chain(&measurement1.function, &transformation0.function);
    let input_metric = (input_glue.metric_clone)(&transformation0.input_metric);
    let output_measure = (output_glue.measure_clone)(&measurement1.output_measure);

    let stability_relation = transformation0.stability_relation.clone();
    let privacy_relation = measurement1.privacy_relation.clone();
    let privacy_relation = PrivacyRelation::new(move |d_in: &MI::Distance, d_out: &MO::Distance| {
        let d_mid = hint(d_in, d_out);
        return stability_relation.eval(d_in, &d_mid)
            && privacy_relation.eval(&d_mid, d_out)
    });
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}


pub struct ChainTT;

impl<DI, DX, DO, MI, MX, MO> MakeTransformation2<DI, DO, MI, MO, &Transformation<DX, DO, MX, MO>, &Transformation<DI, DX, MI, MX>> for ChainTT
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    fn make2(transformation1: &Transformation<DX, DO, MX, MO>, transformation0: &Transformation<DI, DX, MI, MX>) -> Transformation<DI, DO, MI, MO> {
        let input_glue = MetricGlue::<DI, MI>::new();
        let x_glue = MetricGlue::<DX, MX>::new();
        let output_glue = MetricGlue::<DO, MO>::new();
        make_chain_tt_glue(transformation1, transformation0, &input_glue, &x_glue, &output_glue)
    }
}

pub fn make_chain_tt_glue<DI, DX, DO, MI, MX, MO>(transformation1: &Transformation<DX, DO, MX, MO>, transformation0: &Transformation<DI, DX, MI, MX>, input_glue: &MetricGlue<DI, MI>, x_glue: &MetricGlue<DX, MX>, output_glue: &MetricGlue<DO, MO>) -> Transformation<DI, DO, MI, MO> where
    DI: 'static + Domain, DX: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MX: 'static + Metric, MO: 'static + Metric {
    assert!((x_glue.domain_eq)(&transformation0.output_domain, &transformation1.input_domain));
    let input_domain = (input_glue.domain_clone)(&transformation0.input_domain);
    let output_domain = (output_glue.domain_clone)(&transformation1.output_domain);
    let function = Function::make_chain(&transformation1.function, &transformation0.function);
    let input_metric = (input_glue.metric_clone)(&transformation0.input_metric);
    let output_metric = (output_glue.metric_clone)(&transformation1.output_metric);
    // TODO: StabilityRelation for make_chain_tt
    let stability_relation = StabilityRelation::new(|_i, _o| false);

    Transformation { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
}

pub struct Composition;

impl<DI, DO0, DO1, MI, MO> MakeMeasurement2<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO, &Measurement<DI, DO0, MI, MO>, &Measurement<DI, DO1, MI, MO>> for Composition
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MO: 'static + Measure {
    fn make2(measurement0: &Measurement<DI, DO0, MI, MO>, measurement1: &Measurement<DI, DO1, MI, MO>) -> Measurement<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO> {
        let input_glue = MetricGlue::<DI, MI>::new();
        let output_glue0 = MeasureGlue::<DO0, MO>::new();
        let output_glue1 = MeasureGlue::<DO1, MO>::new();
        make_composition_glue(measurement0, measurement1, &input_glue, &output_glue0, &output_glue1)
    }
}

pub fn make_composition_glue<DI, DO0, DO1, MI, MO>(measurement0: &Measurement<DI, DO0, MI, MO>, measurement1: &Measurement<DI, DO1, MI, MO>, input_glue: &MetricGlue<DI, MI>, output_glue0: &MeasureGlue<DO0, MO>, output_glue1: &MeasureGlue<DO1, MO>) -> Measurement<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO> where
    DI: 'static + Domain, DO0: 'static + Domain, DO1: 'static + Domain, MI: 'static + Metric, MO: 'static + Measure {
    assert!((input_glue.domain_eq)(&measurement0.input_domain, &measurement1.input_domain));
    let input_domain = (input_glue.domain_clone)(&measurement0.input_domain);
    let output_domain0 = (output_glue0.domain_clone)(&measurement0.output_domain);
    let output_domain0 = BoxDomain::new(output_domain0);
    let output_domain1 = (output_glue1.domain_clone)(&measurement1.output_domain);
    let output_domain1 = BoxDomain::new(output_domain1);
    let output_domain = PairDomain::new(output_domain0, output_domain1);
    let output_domain = Box::new(output_domain);
    let function = Function::make_composition(&measurement0.function, &measurement1.function);
    // TODO: Figure out input_metric for composition.
    let input_metric = (input_glue.metric_clone)(&measurement0.input_metric);
    // TODO: Figure out output_measure for composition.
    let output_measure = (output_glue0.measure_clone)(&measurement0.output_measure);
    // TODO: PrivacyRelation for make_composition
    let privacy_relation = PrivacyRelation::new(|_i, _o| false);
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::dist::{L1Sensitivity, MaxDivergence};
    use crate::dom::AllDomain;

    use super::*;

    #[test]
    fn test_identity() {
        let input_domain = AllDomain::<i32>::new();
        let output_domain = AllDomain::<i32>::new();
        let function = |arg: &i32| arg.clone();
        let input_metric = L1Sensitivity::<i32>::new();
        let output_metric = L1Sensitivity::<i32>::new();
        let stability_relation = |_d_in: &i32, _d_out: &i32| true;
        let identity = Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_relation);
        let arg = 99;
        let ret = identity.function.eval(&arg);
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_chain_mt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = |a: &u8| (a + 1) as i32;
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_metric0 = L1Sensitivity::<i32>::new();
        let stability_relation0 = |_d_in: &i32, _d_out: &i32| true;
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = |a: &i32| (a + 1) as f64;
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_measure1 = MaxDivergence::new();
        let privacy_relation1 = |_d_in: &i32, _d_out: &f64| true;
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let chain = ChainMT::make(&measurement1, &transformation0);
        let arg = 99_u8;
        let ret = chain.function.eval(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_tt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = |a: &u8| (a + 1) as i32;
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_metric0 = L1Sensitivity::<i32>::new();
        let stability_relation0 = |_d_in: &i32, _d_out: &i32| true;
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = |a: &i32| (a + 1) as f64;
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_metric1 = L1Sensitivity::<i32>::new();
        let stability_relation1 = |_d_in: &i32, _d_out: &i32| true;
        let transformation1 = Transformation::new(input_domain1, output_domain1, function1, input_metric1, output_metric1, stability_relation1);
        let chain = ChainTT::make(&transformation1, &transformation0);
        let arg = 99_u8;
        let ret = chain.function.eval(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_composition() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f32>::new();
        let function0 = |arg: &i32| (arg + 1) as f32;
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_measure0 = MaxDivergence::new();
        let privacy_relation0 = |_d_in: &i32, _d_out: &f64| true;
        let measurement0 = Measurement::new(input_domain0, output_domain0, function0, input_metric0, output_measure0, privacy_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = |arg: &i32| (arg - 1) as f64;
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_measure1 = MaxDivergence::new();
        let privacy_relation1 = |_d_in: &i32, _d_out: &f64| true;
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let composition = Composition::make(&measurement0, &measurement1);
        let arg = 99;
        let ret = composition.function.eval(&arg);
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }

}

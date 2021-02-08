//! Core concepts of OpenDP.
//!
//! This module provides the central building blocks used throughout OpenDP:
//! * Measurement
//! * Transformation
//! * Domain
//! * Metric/Measure
//! * Function
//! * PrivacyRelation/StabilityRelation

use std::rc::Rc;

use crate::dom::{BoxDomain, PairDomain};

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
pub struct Function<ID: Domain, OD: Domain> {
    pub function: Rc<dyn Fn(&ID::Carrier) -> Box<OD::Carrier>>
}

impl<ID: Domain, OD: Domain> Function<ID, OD> {
    pub fn new(function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static) -> Self {
        let function = move |arg: &ID::Carrier| {
            let res = function(arg);
            Box::new(res)
        };
        let function = Rc::new(function);
        Function { function }
    }

    pub fn eval(&self, arg: &ID::Carrier) -> OD::Carrier {
        *(self.function)(arg)
    }

    pub fn eval_ffi(&self, arg: &ID::Carrier) -> Box<OD::Carrier> {
        (self.function)(arg)
    }
}

impl<ID: 'static + Domain, OD: 'static + Domain> Function<ID, OD> {
    pub fn make_chain<XD: 'static + Domain>(function1: &Function<XD, OD>, function0: &Function<ID, XD>) -> Function<ID, OD> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: &ID::Carrier| {
            let res0 = function0(arg);
            function1(&res0)
        };
        let function = Rc::new(function);
        Function { function }
    }
}

impl<ID: 'static + Domain, ODA: 'static + Domain, ODB: 'static + Domain> Function<ID, PairDomain<BoxDomain<ODA>, BoxDomain<ODB>>> {
    pub fn make_composition(function0: &Function<ID, ODA>, function1: &Function<ID, ODB>) -> Self {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: & ID::Carrier| {
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

/// A boolean relation evaluating the privacy of a [`Measurement`].
///
/// A `PrivacyRelation` is implemented as a function that takes an input [`Metric::Distance`] and output [`Measure::Distance`],
/// and returns a boolean indicating if the relation holds.
#[derive(Clone)]
pub struct PrivacyRelation<IM: Metric, OM: Measure> {
    pub relation: Rc<dyn Fn(&IM::Distance, &OM::Distance) -> bool>
}
impl<IM: Metric, OM: Measure> PrivacyRelation<IM, OM> {
    pub fn new(relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static) -> Self {
        let relation = Rc::new(relation);
        PrivacyRelation { relation }
    }
    pub fn eval(&self, input_distance: &IM::Distance, output_distance: &OM::Distance) -> bool {
        (self.relation)(input_distance, output_distance)
    }
}

/// A boolean relation evaluating the stability of a [`Transformation`].
///
/// A `StabilityRelation` is implemented as a function that takes an input and output [`Metric::Distance`],
/// and returns a boolean indicating if the relation holds.
#[derive(Clone)]
pub struct StabilityRelation<IM: Metric, OM: Metric> {
    pub relation: Rc<dyn Fn(&IM::Distance, &OM::Distance) -> bool>
}
impl<IM: Metric, OM: Metric> StabilityRelation<IM, OM> {
    pub fn new(relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static) -> Self {
        let relation = Rc::new(relation);
        StabilityRelation { relation }
    }
    pub fn eval(&self, input_distance: &IM::Distance, output_distance: &OM::Distance) -> bool {
        (self.relation)(input_distance, output_distance)
    }
}


/// A randomized mechanism with certain privacy characteristics.
pub struct Measurement<ID: Domain, OD: Domain, IM: Metric, OM: Measure> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID, OD>,
    pub input_metric: Box<IM>,
    pub output_measure: Box<OM>,
    pub privacy_relation: PrivacyRelation<IM, OM>,
}

impl<ID: Domain, OD: Domain, IM: Metric, OM: Measure> Measurement<ID, OD, IM, OM> {
    pub fn new(
        input_domain: ID,
        output_domain: OD,
        function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static,
        input_metric: IM,
        output_measure: OM,
        privacy_relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static,
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
pub struct Transformation<ID: Domain, OD: Domain, IM: Metric, OM: Metric> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID, OD>,
    pub input_metric: Box<IM>,
    pub output_metric: Box<OM>,
    pub stability_relation: StabilityRelation<IM, OM>,
}

impl<ID: Domain, OD: Domain, IM: Metric, OM: Metric> Transformation<ID, OD, IM, OM> {
    pub fn new(
        input_domain: ID,
        output_domain: OD,
        function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static,
        input_metric: IM,
        output_metric: OM,
        stability_relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static,
    ) -> Self {
        let input_domain = Box::new(input_domain);
        let output_domain = Box::new(output_domain);
        let function = Function::new(function);
        let input_metric = Box::new(input_metric);
        let output_metric = Box::new(output_metric);
        let stability_relation = StabilityRelation::new(stability_relation);
        Transformation { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
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
pub fn make_chain_mt<ID, XD, OD, IM, XM, OM>(measurement1: &Measurement<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>) -> Measurement<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Measure {
    let input_glue = MetricGlue::<ID, IM>::new();
    let x_glue = MetricGlue::<XD, XM>::new();
    let output_glue = MeasureGlue::<OD, OM>::new();
    make_chain_mt_glue(measurement1, transformation0, &input_glue, &x_glue, &output_glue)
}

pub fn make_chain_mt_glue<ID, XD, OD, IM, XM, OM>(measurement1: &Measurement<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>, input_glue: &MetricGlue<ID, IM>, x_glue: &MetricGlue<XD, XM>, output_glue: &MeasureGlue<OD, OM>) -> Measurement<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Measure {
    assert!((x_glue.domain_eq)(&transformation0.output_domain, &measurement1.input_domain));
    let input_domain = (input_glue.domain_clone)(&transformation0.input_domain);
    let output_domain = (output_glue.domain_clone)(&measurement1.output_domain);
    let function = Function::make_chain(&measurement1.function, &transformation0.function);
    let input_metric = (input_glue.metric_clone)(&transformation0.input_metric);
    let output_measure = (output_glue.measure_clone)(&measurement1.output_measure);
    // TODO: PrivacyRelation for make_chain_mt
    let privacy_relation = PrivacyRelation::new(|_i, _o| false);
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}

pub fn make_chain_tt<ID, XD, OD, IM, XM, OM>(transformation1: &Transformation<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>) -> Transformation<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Metric {
    let input_glue = MetricGlue::<ID, IM>::new();
    let x_glue = MetricGlue::<XD, XM>::new();
    let output_glue = MetricGlue::<OD, OM>::new();
    make_chain_tt_glue(transformation1, transformation0, &input_glue, &x_glue, &output_glue)
}

pub fn make_chain_tt_glue<ID, XD, OD, IM, XM, OM>(transformation1: &Transformation<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>, input_glue: &MetricGlue<ID, IM>, x_glue: &MetricGlue<XD, XM>, output_glue: &MetricGlue<OD, OM>) -> Transformation<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Metric {
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

pub fn make_composition<ID, OD0, OD1, IM, OM>(measurement0: &Measurement<ID, OD0, IM, OM>, measurement1: &Measurement<ID, OD1, IM, OM>) -> Measurement<ID, PairDomain<BoxDomain<OD0>, BoxDomain<OD1>>, IM, OM> where
    ID: 'static + Domain, OD0: 'static + Domain, OD1: 'static + Domain, IM: 'static + Metric, OM: 'static + Measure {
    let input_glue = MetricGlue::<ID, IM>::new();
    let output_glue0 = MeasureGlue::<OD0, OM>::new();
    let output_glue1 = MeasureGlue::<OD1, OM>::new();
    make_composition_glue(measurement0, measurement1, &input_glue, &output_glue0, &output_glue1)
}

pub fn make_composition_glue<ID, OD0, OD1, IM, OM>(measurement0: &Measurement<ID, OD0, IM, OM>, measurement1: &Measurement<ID, OD1, IM, OM>, input_glue: &MetricGlue<ID, IM>, output_glue0: &MeasureGlue<OD0, OM>, output_glue1: &MeasureGlue<OD1, OM>) -> Measurement<ID, PairDomain<BoxDomain<OD0>, BoxDomain<OD1>>, IM, OM> where
    ID: 'static + Domain, OD0: 'static + Domain, OD1: 'static + Domain, IM: 'static + Metric, OM: 'static + Measure {
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
        let chain = make_chain_mt(&measurement1, &transformation0);
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
        let chain = make_chain_tt(&transformation1, &transformation0);
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
        let composition = make_composition(&measurement0, &measurement1);
        let arg = 99;
        let ret = composition.function.eval(&arg);
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }

}

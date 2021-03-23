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

use std::ops::{Div, Mul};
use std::rc::Rc;

use crate::error::Fallible;
use crate::dom::{BoxDomain, PairDomain};
use crate::meas::MakeMeasurement2;
use crate::traits::DistanceCast;
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
    pub function: Rc<dyn Fn(&DI::Carrier) -> Fallible<Box<DO::Carrier>>>
}

impl<DI: Domain, DO: Domain> Function<DI, DO> {
    pub fn new(function: impl Fn(&DI::Carrier) -> DO::Carrier + 'static) -> Self {
        Function { function: Rc::new(move |arg: &DI::Carrier| Ok(Box::new(function(arg)))) }
    }
    pub fn new_fallible(function: impl Fn(&DI::Carrier) -> Fallible<DO::Carrier> + 'static) -> Self {
        Function { function: Rc::new(move |arg: &DI::Carrier| function(arg).map(Box::new)) }
    }

    pub fn eval(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        (self.function)(arg).map(|v| *v)
    }

    pub fn eval_ffi(&self, arg: &DI::Carrier) -> Fallible<Box<DO::Carrier>> {
        (self.function)(arg)
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain> Function<DI, DO> {
    pub fn make_chain<XD: 'static + Domain>(function1: &Function<XD, DO>, function0: &Function<DI, XD>) -> Function<DI, DO> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: &DI::Carrier| -> Fallible<Box<DO::Carrier>> {
            function1(&*function0(arg)?)
        };
        let function = Rc::new(function);
        Function { function }
    }
}

impl<DI: 'static + Domain, DO1: 'static + Domain, DO2: 'static + Domain> Function<DI, PairDomain<BoxDomain<DO1>, BoxDomain<DO2>>> {
    pub fn make_composition(function0: &Function<DI, DO1>, function1: &Function<DI, DO2>) -> Self {
        let f0 = &function0.function;
        let f1 = &function1.function;
        Function {
            function: Rc::new(enclose!((f0, f1), move |arg: & DI::Carrier| {
                Ok(Box::new((f0(arg)?, f1(arg)?)))
            }))
        }
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


// HINTS
#[derive(Clone)]
pub struct HintMt<MI: Metric, MO: Measure, MX: Metric> {
    pub hint: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<Box<MX::Distance>>>
}
impl<MI: Metric, MO: Measure, MX: Metric> HintMt<MI, MO, MX> {
    pub fn new(hint: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<Box<MX::Distance>> + 'static) -> Self {
        HintMt { hint: Rc::new(hint) }
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<MX::Distance> {
        (self.hint)(input_distance, output_distance).map(|v| *v)
    }
}

#[derive(Clone)]
pub struct HintTt<MI: Metric, MO: Metric, MX: Metric> {
    pub hint: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<Box<MX::Distance>>>
}
impl<MI: Metric, MO: Metric, MX: Metric> HintTt<MI, MO, MX> {
    pub fn new_fallible(hint: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<Box<MX::Distance>> + 'static) -> Self {
        HintTt { hint: Rc::new(hint) }
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<MX::Distance> {
        (self.hint)(input_distance, output_distance).map(|v| *v)
    }
}


/// A boolean relation evaluating the privacy of a [`Measurement`].
///
/// A `PrivacyRelation` is implemented as a function that takes an input [`Metric::Distance`] and output [`Measure::Distance`],
/// and returns a boolean indicating if the relation holds.
#[derive(Clone)]
pub struct PrivacyRelation<MI: Metric, MO: Measure> {
    pub relation: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<bool>>,
    pub backward_map: Option<Rc<dyn Fn(&MO::Distance) -> Fallible<Box<MI::Distance>>>>,
}

impl<MI: Metric, MO: Measure> PrivacyRelation<MI, MO> {
    pub fn new(relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static) -> Self {
        PrivacyRelation {
            relation: Rc::new(move |d_in: &MI::Distance, d_out: &MO::Distance| Ok(relation(d_in, d_out))),
            backward_map: None
        }
    }
    pub fn new_fallible(relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static) -> Self {
        PrivacyRelation {
            relation: Rc::new(relation),
            backward_map: None
        }
    }
    fn new_all(
        relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static,
        backward_map: Option<impl Fn(&MO::Distance) -> Fallible<Box<MI::Distance>> + 'static>
    ) -> Self {
        PrivacyRelation {
            relation: Rc::new(relation),
            backward_map: backward_map.map(|h| Rc::new(h) as Rc<_>),
        }
    }
    pub fn new_from_constant(c: MO::Distance) -> Self where
        MI::Distance: Clone + DistanceCast,
        MO::Distance: Clone + DistanceCast + Mul<Output=MO::Distance> + Div<Output=MO::Distance> + PartialOrd + 'static {

        PrivacyRelation::new_all(
            enclose!(c, move |d_in: &MI::Distance, d_out: &MO::Distance|
                Ok(d_out.clone() >= MO::Distance::cast(d_in.clone())? * c.clone())),
            Some(enclose!(c, move |d_out: &MO::Distance|
                Ok(Box::new(MI::Distance::cast(d_out.clone() / c.clone())?)))))
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<bool> {
        (self.relation)(input_distance, output_distance)
    }
}

impl<MI: 'static + Metric, MO: 'static + Measure> PrivacyRelation<MI, MO> {
    pub fn make_chain<MX: 'static + Metric>(
        relation1: &PrivacyRelation<MX, MO>,
        relation0: &StabilityRelation<MI, MX>,
        hint: Option<&HintMt<MI, MO, MX>>
    ) -> Self {
        if let Some(hint) = hint {
            Self::make_chain_hint(relation1, relation0, hint)
        } else {
            Self::make_chain_no_hint(relation1, relation0)
        }
    }

    fn make_chain_no_hint<MX: 'static + Metric>(
        relation1: &PrivacyRelation<MX, MO>,
        relation0: &StabilityRelation<MI, MX>
    ) -> Self {
        let hint = if let Some(forward_map) = &relation0.forward_map {
            Some(HintMt::new(enclose!(forward_map, move |d_in, _d_out| forward_map(d_in))))
        } else if let Some(backward_map) = &relation1.backward_map {
            Some(HintMt::new(enclose!(backward_map, move |_d_in, d_out| backward_map(d_out))))
        } else {
            None
        };
        if let Some(hint) = hint {
            Self::make_chain_hint(relation1, relation0, &hint)
        } else {
            // TODO: Implement binary search for hints.
            panic!("Binary search for hints not implemented, must have maps or supply explicit hint.")
        }
    }

    fn make_chain_hint<MX: 'static + Metric>(relation1: &PrivacyRelation<MX, MO>, relation0: &StabilityRelation<MI, MX>, hint: &HintMt<MI, MO, MX>) -> Self {
        fn chain_option_maps<QI, QX, QO>(
            map1: &Option<Rc<dyn Fn(&QX) -> Fallible<Box<QO>>>>,
            map0: &Option<Rc<dyn Fn(&QI) -> Fallible<Box<QX>>>>
        ) -> Option<impl Fn(&QI) -> Fallible<Box<QO>>> {
            if let (Some(map0), Some(map1)) = (map0, map1) {
                Some(enclose!((map1, map0), move |d_in: &QI| map1(&*map0(d_in)?)))
            } else {
                None
            }
        }
        let PrivacyRelation {
            relation: relation1,
            backward_map: backward_map1
        } = relation1;

        let StabilityRelation {
            relation: relation0,
            forward_map: _,
            backward_map: backward_map0,
        } = relation0;

        let h = hint.hint.clone();

        PrivacyRelation::new_all(
            enclose!((relation1, relation0), move |d_in: &MI::Distance, d_out: &MO::Distance| {
                let d_mid = h(d_in, d_out)?;
                Ok(relation0(d_in, &d_mid)? && relation1(&d_mid, d_out)?)
            }),
            chain_option_maps(backward_map0, backward_map1))
    }
}

/// A boolean relation evaluating the stability of a [`Transformation`].
///
/// A `StabilityRelation` is implemented as a function that takes an input and output [`Metric::Distance`],
/// and returns a boolean indicating if the relation holds.
#[derive(Clone)]
pub struct StabilityRelation<MI: Metric, MO: Metric> {
    pub relation: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<bool>>,
    pub forward_map: Option<Rc<dyn Fn(&MI::Distance) -> Fallible<Box<MO::Distance>>>>,
    pub backward_map: Option<Rc<dyn Fn(&MO::Distance) -> Fallible<Box<MI::Distance>>>>,
}
impl<MI: Metric, MO: Metric> StabilityRelation<MI, MO> {
    pub fn new(relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static) -> Self {
        StabilityRelation {
            relation: Rc::new(move |d_in: &MI::Distance, d_out: &MO::Distance| Ok(relation(d_in, d_out))),
            forward_map: None,
            backward_map: None
        }
    }
    pub fn new_fallible(relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static) -> Self {
        StabilityRelation { relation: Rc::new(relation), forward_map: None, backward_map: None }
    }
    fn new_all(
        relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static,
        forward_map: Option<impl Fn(&MI::Distance) -> Fallible<Box<MO::Distance>> + 'static>,
        backward_map: Option<impl Fn(&MO::Distance) -> Fallible<Box<MI::Distance>> + 'static>
    ) -> Self {
        StabilityRelation {
            relation: Rc::new(relation),
            forward_map: forward_map.map(|h| Rc::new(h) as Rc<_>),
            backward_map: backward_map.map(|h| Rc::new(h) as Rc<_>),
        }
    }
    pub fn new_from_constant(c: MO::Distance) -> Self where
        MI::Distance: Clone + DistanceCast,
        MO::Distance: Clone + DistanceCast + Mul<Output=MO::Distance> + Div<Output=MO::Distance> + PartialOrd + 'static {

        StabilityRelation::new_all(
            // relation
            enclose!(c, move |d_in: &MI::Distance, d_out: &MO::Distance|
                Ok(d_out.clone() >= MO::Distance::cast(d_in.clone())? * c.clone())),
            // forward map
            Some(enclose!(c, move |d_in: &MI::Distance|
                Ok(Box::new(MO::Distance::cast(d_in.clone())? * c.clone())))),
            // backward map
            Some(enclose!(c, move |d_out: &MO::Distance|
                Ok(Box::new(MI::Distance::cast(d_out.clone() / c.clone())?)))))
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<bool> {
        (self.relation)(input_distance, output_distance)
    }
}

impl<MI: 'static + Metric, MO: 'static + Metric> StabilityRelation<MI, MO> {
    pub fn make_chain<MX: 'static + Metric>(relation1: &StabilityRelation<MX, MO>, relation0: &StabilityRelation<MI, MX>, hint: Option<&HintTt<MI, MO, MX>>) -> Self {
        if let Some(hint) = hint {
            Self::make_chain_hint(relation1, relation0, hint)
        } else {
            Self::make_chain_no_hint(relation1, relation0)
        }
    }

    fn make_chain_no_hint<MX: 'static + Metric>(relation1: &StabilityRelation<MX, MO>, relation0: &StabilityRelation<MI, MX>) -> Self {
        let hint = if let Some(forward_map) = &relation0.forward_map {
            let forward_map = forward_map.clone();
            Some(HintTt::new_fallible(move |d_in, _d_out| forward_map(d_in)))
        } else if let Some(backward_map) = &relation1.backward_map {
            let backward_map = backward_map.clone();
            Some(HintTt::new_fallible(move |_d_in, d_out| backward_map(d_out)))
        } else {
            None
        };
        if let Some(hint) = hint {
            Self::make_chain_hint(relation1, relation0, &hint)
        } else {
            // TODO: Implement binary search for hints.
            panic!("Binary search for hints not implemented, must have maps or supply explicit hint.")
        }
    }

    fn make_chain_hint<MX: 'static + Metric>(relation1: &StabilityRelation<MX, MO>, relation0: &StabilityRelation<MI, MX>, hint: &HintTt<MI, MO, MX>) -> Self {
        fn chain_option_maps<QI, QX, QO>(
            map1: &Option<Rc<dyn Fn(&QX) -> Fallible<Box<QO>>>>,
            map0: &Option<Rc<dyn Fn(&QI) -> Fallible<Box<QX>>>>
        ) -> Option<impl Fn(&QI) -> Fallible<Box<QO>>> {
            if let (Some(map0), Some(map1)) = (map0, map1) {
                Some(enclose!((map0, map1), move |d_in: &QI| map1(&*map0(d_in)?)))
            } else {
                None
            }
        }

        let StabilityRelation {
            relation: relation0,
            forward_map: forward_map0,
            backward_map: backward_map0
        } = relation0;

        let StabilityRelation {
            relation: relation1,
            forward_map: forward_map1,
            backward_map: backward_map1
        } = relation1;

        let h = hint.hint.clone();
        StabilityRelation::new_all(
            enclose!((relation1, relation0), move |d_in: &MI::Distance, d_out: &MO::Distance| {
                let d_mid = h(d_in, d_out)?;
                Ok(relation0(d_in, &d_mid)? && relation1(&d_mid, d_out)?)
            }),
            chain_option_maps(forward_map1, forward_map0),
            chain_option_maps(backward_map0, backward_map1))
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
        function: Function<DI, DO>,
        input_metric: MI,
        output_measure: MO,
        privacy_relation: PrivacyRelation<MI, MO>,
    ) -> Self {
        Measurement {
            input_domain: Box::new(input_domain),
            output_domain: Box::new(output_domain),
            function,
            input_metric: Box::new(input_metric),
            output_measure: Box::new(output_measure),
            privacy_relation,
        }
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
        function: Function<DI, DO>,
        input_metric: MI,
        output_metric: MO,
        stability_relation: StabilityRelation<MI, MO>,
    ) -> Self {
        Transformation {
            input_domain: Box::new(input_domain),
            output_domain: Box::new(output_domain),
            function,
            input_metric: Box::new(input_metric),
            output_metric: Box::new(output_metric),
            stability_relation
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

impl ChainMT {
    pub fn make_chain_mt_glue<DI, DX, DO, MI, MX, MO>(
        measurement1: &Measurement<DX, DO, MX, MO>,
        transformation0: &Transformation<DI, DX, MI, MX>,
        hint: Option<&HintMt<MI, MO, MX>>,
        input_glue: &MetricGlue<DI, MI>,
        x_glue: &MetricGlue<DX, MX>,
        output_glue: &MeasureGlue<DO, MO>
    ) -> Fallible<Measurement<DI, DO, MI, MO>>
        where DI: 'static + Domain,
              DX: 'static + Domain,
              DO: 'static + Domain,
              MI: 'static + Metric,
              MX: 'static + Metric,
              MO: 'static + Measure {
        if (x_glue.domain_eq)(&transformation0.output_domain, &measurement1.input_domain) {
            Ok(Measurement {
                input_domain: (input_glue.domain_clone)(&transformation0.input_domain),
                output_domain: (output_glue.domain_clone)(&measurement1.output_domain),
                function: Function::make_chain(&measurement1.function, &transformation0.function),
                input_metric: (input_glue.metric_clone)(&transformation0.input_metric),
                output_measure: (output_glue.measure_clone)(&measurement1.output_measure),
                privacy_relation: PrivacyRelation::make_chain(
                    &measurement1.privacy_relation,
                    &transformation0.stability_relation, hint)
            })
        } else {
            fallible!(DomainMismatch)
        }
    }
}

impl<DI, DX, DO, MI, MX, MO> MakeMeasurement2<DI, DO, MI, MO, &Measurement<DX, DO, MX, MO>, &Transformation<DI, DX, MI, MX>> for ChainMT
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Measure {
    fn make2(measurement1: &Measurement<DX, DO, MX, MO>, transformation0: &Transformation<DI, DX, MI, MX>) -> Fallible<Measurement<DI, DO, MI, MO>> {
        Self::make_chain_mt_glue(
            measurement1, transformation0, None,
            &MetricGlue::<DI, MI>::new(),
            &MetricGlue::<DX, MX>::new(),
            &MeasureGlue::<DO, MO>::new())
    }
}

pub struct ChainTT;

impl ChainTT {
    pub fn make_chain_tt_glue<DI, DX, DO, MI, MX, MO>(
        transformation1: &Transformation<DX, DO, MX, MO>,
        transformation0: &Transformation<DI, DX, MI, MX>,
        hint: Option<&HintTt<MI, MO, MX>>,
        input_glue: &MetricGlue<DI, MI>,
        x_glue: &MetricGlue<DX, MX>,
        output_glue: &MetricGlue<DO, MO>
    ) -> Fallible<Transformation<DI, DO, MI, MO>>
        where DI: 'static + Domain,
              DX: 'static + Domain,
              DO: 'static + Domain,
              MI: 'static + Metric,
              MX: 'static + Metric,
              MO: 'static + Metric {
        if (x_glue.domain_eq)(&transformation0.output_domain, &transformation1.input_domain) {
            Ok(Transformation {
                input_domain: (input_glue.domain_clone)(&transformation0.input_domain),
                output_domain: (output_glue.domain_clone)(&transformation1.output_domain),
                function: Function::make_chain(&transformation1.function, &transformation0.function),
                input_metric: (input_glue.metric_clone)(&transformation0.input_metric),
                output_metric: (output_glue.metric_clone)(&transformation1.output_metric),
                stability_relation: StabilityRelation::make_chain(
                    &transformation1.stability_relation,
                    &transformation0.stability_relation, hint)
            })
        } else {
            fallible!(DomainMismatch)
        }
    }
}

impl<DI, DX, DO, MI, MX, MO> MakeTransformation2<DI, DO, MI, MO, &Transformation<DX, DO, MX, MO>, &Transformation<DI, DX, MI, MX>> for ChainTT
    where DI: 'static + Domain,
          DX: 'static + Domain,
          DO: 'static + Domain,
          MI: 'static + Metric,
          MX: 'static + Metric,
          MO: 'static + Metric {
    fn make2(transformation1: &Transformation<DX, DO, MX, MO>, transformation0: &Transformation<DI, DX, MI, MX>) -> Fallible<Transformation<DI, DO, MI, MO>> {
        Self::make_chain_tt_glue(
            transformation1, transformation0, None,
            &MetricGlue::<DI, MI>::new(),
            &MetricGlue::<DX, MX>::new(),
            &MetricGlue::<DO, MO>::new())
    }
}

pub struct Composition;

impl<DI, DO0, DO1, MI, MO> MakeMeasurement2<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO, &Measurement<DI, DO0, MI, MO>, &Measurement<DI, DO1, MI, MO>> for Composition
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MO: 'static + Measure {
    fn make2(measurement0: &Measurement<DI, DO0, MI, MO>, measurement1: &Measurement<DI, DO1, MI, MO>) -> Fallible<Measurement<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO>> {
        make_composition_glue(
            measurement0, measurement1,
            &MetricGlue::<DI, MI>::new(),
            &MeasureGlue::<DO0, MO>::new(),
            &MeasureGlue::<DO1, MO>::new())
    }
}

pub fn make_composition_glue<DI, DO0, DO1, MI, MO>(
    measurement0: &Measurement<DI, DO0, MI, MO>,
    measurement1: &Measurement<DI, DO1, MI, MO>,
    input_glue: &MetricGlue<DI, MI>,
    output_glue0: &MeasureGlue<DO0, MO>,
    output_glue1: &MeasureGlue<DO1, MO>
) -> Fallible<Measurement<DI, PairDomain<BoxDomain<DO0>, BoxDomain<DO1>>, MI, MO>>
    where DI: 'static + Domain,
          DO0: 'static + Domain,
          DO1: 'static + Domain,
          MI: 'static + Metric,
          MO: 'static + Measure {
    if (input_glue.domain_eq)(&measurement0.input_domain, &measurement1.input_domain) {
        Ok(Measurement {
            input_domain: (input_glue.domain_clone)(&measurement0.input_domain),
            output_domain: Box::new(PairDomain::new(
                BoxDomain::new((output_glue0.domain_clone)(&measurement0.output_domain)),
                BoxDomain::new((output_glue1.domain_clone)(&measurement1.output_domain)))),
            function: Function::make_composition(&measurement0.function, &measurement1.function),
            // TODO: Figure out input_metric for composition.
            input_metric: (input_glue.metric_clone)(&measurement0.input_metric),
            // TODO: Figure out output_measure for composition.
            output_measure: (output_glue0.measure_clone)(&measurement0.output_measure),
            // TODO: PrivacyRelation for make_composition
            privacy_relation: PrivacyRelation::new(|_i, _o| false)
        })
    } else {
        fallible!(DomainMismatch)
    }
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
        let function = Function::new(|arg: &i32| arg.clone());
        let input_metric = L1Sensitivity::<i32>::new();
        let output_metric = L1Sensitivity::<i32>::new();
        let stability_relation = StabilityRelation::new_from_constant(1);
        let identity = Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_relation);
        let arg = 99;
        let ret = identity.function.eval(&arg).unwrap();
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_chain_mt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_metric0 = L1Sensitivity::<i32>::new();
        let stability_relation0 = StabilityRelation::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_measure1 = MaxDivergence::new();
        let privacy_relation1 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let chain = ChainMT::make(&measurement1, &transformation0).unwrap();
        let arg = 99_u8;
        let ret = chain.function.eval(&arg).unwrap();
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_tt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_metric0 = L1Sensitivity::<i32>::new();
        let stability_relation0 = StabilityRelation::new_from_constant(1);
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_metric1 = L1Sensitivity::<i32>::new();
        let stability_relation1 = StabilityRelation::new_from_constant(1);
        let transformation1 = Transformation::new(input_domain1, output_domain1, function1, input_metric1, output_metric1, stability_relation1);
        let chain = ChainTT::make(&transformation1, &transformation0).unwrap();
        let arg = 99_u8;
        let ret = chain.function.eval(&arg).unwrap();
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_composition() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f32>::new();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f32);
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_measure0 = MaxDivergence::new();
        let privacy_relation0 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement0 = Measurement::new(input_domain0, output_domain0, function0, input_metric0, output_measure0, privacy_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_measure1 = MaxDivergence::new();
        let privacy_relation1 = PrivacyRelation::new(|_d_in: &i32, _d_out: &f64| true);
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let composition = Composition::make(&measurement0, &measurement1).unwrap();
        let arg = 99;
        let ret = composition.function.eval(&arg).unwrap();
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }
}

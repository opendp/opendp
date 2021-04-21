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

// Ordering of generic arguments
// DI, DO, MI, MO, TI, TO, QI, QO

use std::ops::{Div, Mul};
use std::rc::Rc;

use crate::dom::{BoxDomain, PairDomain};
use crate::error::*;
use crate::traits::{Distance, DistanceCast};

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
pub trait Metric: Default + Clone {
    type Distance;
}

/// A representation of the distance between two distributions.
pub trait Measure: Default + Clone {
    type Distance;
}

/// A indicator trait that is only implemented for dataset distance
pub trait DatasetMetric: Metric {}
pub trait SensitivityMetric: Metric {}


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
        MO::Distance: 'static + Distance {

        PrivacyRelation::new_all(
            enclose!(c, move |d_in: &MI::Distance, d_out: &MO::Distance|
                Ok(d_out.clone() >= MO::Distance::distance_cast(d_in.clone())? * c.clone())),
            Some(enclose!(c, move |d_out: &MO::Distance|
                Ok(Box::new(MI::Distance::distance_cast(d_out.clone() / c.clone())?)))))
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<bool> {
        (self.relation)(input_distance, output_distance)
    }
}

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
            unimplemented!("Binary search for hints not implemented, must have maps or supply explicit hint.")
        }
    }

    fn make_chain_hint<MX: 'static + Metric>(relation1: &PrivacyRelation<MX, MO>, relation0: &StabilityRelation<MI, MX>, hint: &HintMt<MI, MO, MX>) -> Self {
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
                Ok(d_out.clone() >= MO::Distance::distance_cast(d_in.clone())? * c.clone())),
            // forward map
            Some(enclose!(c, move |d_in: &MI::Distance|
                Ok(Box::new(MO::Distance::distance_cast(d_in.clone())? * c.clone())))),
            // backward map
            Some(enclose!(c, move |d_out: &MO::Distance|
                Ok(Box::new(MI::Distance::distance_cast(d_out.clone() / c.clone())?)))))
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
            unimplemented!("Binary search for hints not implemented, must have maps or supply explicit hint.")
        }
    }

    fn make_chain_hint<MX: 'static + Metric>(relation1: &StabilityRelation<MX, MO>, relation0: &StabilityRelation<MI, MX>, hint: &HintTt<MI, MO, MX>) -> Self {

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

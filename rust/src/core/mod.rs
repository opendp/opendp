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

#[cfg(feature="ffi")]
mod ffi;
#[cfg(feature="ffi")]
pub use ffi::*;

use std::rc::Rc;

use crate::dom::PairDomain;
use crate::error::*;
use crate::traits::{DistanceConstant, InfCast, InfMul, InfDiv};
use crate::dist::IntDistance;
use std::fmt::Debug;

/// A set which constrains the input or output of a [`Function`].
///
/// Domains capture the notion of what values are allowed to be the input or output of a `Function`.
pub trait Domain: Clone + PartialEq + Debug {
    /// The underlying type that the Domain specializes.
    type Carrier;
    /// Predicate to test an element for membership in the domain.
    fn member(&self, val: &Self::Carrier) -> Fallible<bool>;
}

/// A mathematical function which maps values from an input [`Domain`] to an output [`Domain`].
pub struct Function<DI: Domain, DO: Domain> {
    pub function: Rc<dyn Fn(&DI::Carrier) -> Fallible<DO::Carrier>>,
}
impl<DI: Domain, DO: Domain> Clone for Function<DI, DO> {
    fn clone(&self) -> Self {
        Function {function: self.function.clone()}
    }
}

impl<DI: Domain, DO: Domain> Function<DI, DO> {
    pub fn new(function: impl Fn(&DI::Carrier) -> DO::Carrier + 'static) -> Self {
        Self::new_fallible(move |arg| Ok(function(arg)))
    }

    pub fn new_fallible(function: impl Fn(&DI::Carrier) -> Fallible<DO::Carrier> + 'static) -> Self {
        Self { function: Rc::new(function) }
    }

    pub fn eval(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        (self.function)(arg)
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain> Function<DI, DO> {
    pub fn make_chain<XD: 'static + Domain>(function1: &Function<XD, DO>, function0: &Function<DI, XD>) -> Function<DI, DO> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        Self::new_fallible(move |arg| function1(&function0(arg)?))
    }
}

impl<DI: 'static + Domain, DO0: 'static + Domain, DO1: 'static + Domain> Function<DI, PairDomain<DO0, DO1>> {
    pub fn make_basic_composition(function0: &Function<DI, DO0>, function1: &Function<DI, DO1>) -> Self {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        Self::new_fallible(move |arg| Ok((function0(arg)?, function1(arg)?)))
    }
}

/// A representation of the distance between two elements in a set.
pub trait Metric: Default + Clone + PartialEq + Debug {
    type Distance;
}

/// A representation of the distance between two distributions.
pub trait Measure: Default + Clone + PartialEq + Debug {
    type Distance;
}

/// An indicator trait that is only implemented for dataset distances.
pub trait DatasetMetric: Metric<Distance=IntDistance> {}

/// An indicator trait that is only implemented for statistic distances.
pub trait SensitivityMetric: Metric {}


// HINTS
#[derive(Clone)]
pub struct HintMt<MI: Metric, MO: Measure, MX: Metric> {
    pub hint: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<MX::Distance>>,
}

impl<MI: Metric, MO: Measure, MX: Metric> HintMt<MI, MO, MX> {
    pub fn new(hint: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<MX::Distance> + 'static) -> Self {
        HintMt { hint: Rc::new(hint) }
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<MX::Distance> {
        (self.hint)(input_distance, output_distance)
    }
}

#[derive(Clone)]
pub struct HintTt<MI: Metric, MO: Metric, MX: Metric> {
    pub hint: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<MX::Distance>>,
}

impl<MI: Metric, MO: Metric, MX: Metric> HintTt<MI, MO, MX> {
    pub fn new_fallible(hint: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<MX::Distance> + 'static) -> Self {
        HintTt { hint: Rc::new(hint) }
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<MX::Distance> {
        (self.hint)(input_distance, output_distance)
    }
}


/// A boolean relation evaluating the privacy of a [`Measurement`].
///
/// A `PrivacyRelation` is implemented as a function that takes an input [`Metric::Distance`] and output [`Measure::Distance`],
/// and returns a boolean indicating if the relation holds.
pub struct PrivacyRelation<MI: Metric, MO: Measure> {
    pub relation: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<bool>>,
    pub backward_map: Option<Rc<dyn Fn(&MO::Distance) -> Fallible<MI::Distance>>>,
}

impl<MI: Metric, MO: Measure> Clone for PrivacyRelation<MI, MO> {
    fn clone(&self) -> Self {
        PrivacyRelation {
            relation: self.relation.clone(),
            backward_map: self.backward_map.clone()
        }
    }
}

impl<MI: Metric, MO: Measure> PrivacyRelation<MI, MO> {
    pub fn new(relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static) -> Self {
        PrivacyRelation {
            relation: Rc::new(move |d_in: &MI::Distance, d_out: &MO::Distance| Ok(relation(d_in, d_out))),
            backward_map: None,
        }
    }
    pub fn new_fallible(relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static) -> Self {
        PrivacyRelation {
            relation: Rc::new(relation),
            backward_map: None,
        }
    }
    pub fn new_all(
        relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static,
        backward_map: Option<impl Fn(&MO::Distance) -> Fallible<MI::Distance> + 'static>,
    ) -> Self {
        PrivacyRelation {
            relation: Rc::new(relation),
            backward_map: backward_map.map(|h| Rc::new(h) as Rc<_>),
        }
    }
    pub fn new_from_constant(c: MO::Distance) -> Self where
        MI::Distance: InfCast<MO::Distance> + Clone,
        MO::Distance: DistanceConstant<MI::Distance> {
        PrivacyRelation::new_all(
            enclose!(c, move |d_in: &MI::Distance, d_out: &MO::Distance|
                Ok(d_out.clone() >= MO::Distance::inf_cast(d_in.clone())?.inf_mul(&c)?)),
            Some(enclose!(c, move |d_out: &MO::Distance|
                Ok(MI::Distance::inf_cast(d_out.inf_div(&c)?)?))))
    }
    pub fn eval(&self, input_distance: &MI::Distance, output_distance: &MO::Distance) -> Fallible<bool> {
        (self.relation)(input_distance, output_distance)
    }
}

fn chain_option_maps<QI, QX, QO>(
    map1: &Option<Rc<dyn Fn(&QX) -> Fallible<QO>>>,
    map0: &Option<Rc<dyn Fn(&QI) -> Fallible<QX>>>,
) -> Option<impl Fn(&QI) -> Fallible<QO>> {
    if let (Some(map0), Some(map1)) = (map0, map1) {
        Some(enclose!((map0, map1), move |d_in: &QI| map1(&map0(d_in)?)))
    } else {
        None
    }
}

impl<MI: 'static + Metric, MO: 'static + Measure> PrivacyRelation<MI, MO> {
    pub fn make_chain<MX: 'static + Metric>(
        relation1: &PrivacyRelation<MX, MO>,
        relation0: &StabilityRelation<MI, MX>,
        hint: Option<&HintMt<MI, MO, MX>>,
    ) -> Self {
        if let Some(hint) = hint {
            Self::make_chain_hint(relation1, relation0, hint)
        } else {
            Self::make_chain_no_hint(relation1, relation0)
        }
    }

    fn make_chain_no_hint<MX: 'static + Metric>(
        relation1: &PrivacyRelation<MX, MO>,
        relation0: &StabilityRelation<MI, MX>,
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
pub struct StabilityRelation<MI: Metric, MO: Metric> {
    pub relation: Rc<dyn Fn(&MI::Distance, &MO::Distance) -> Fallible<bool>>,
    pub forward_map: Option<Rc<dyn Fn(&MI::Distance) -> Fallible<MO::Distance>>>,
    pub backward_map: Option<Rc<dyn Fn(&MO::Distance) -> Fallible<MI::Distance>>>,
}

impl<MI: Metric, MO: Metric> Clone for StabilityRelation<MI, MO> {
    fn clone(&self) -> Self {
        StabilityRelation {
            relation: self.relation.clone(),
            forward_map: self.forward_map.clone(),
            backward_map: self.backward_map.clone()
        }
    }
}

impl<MI: Metric, MO: Metric> StabilityRelation<MI, MO> {
    pub fn new(relation: impl Fn(&MI::Distance, &MO::Distance) -> bool + 'static) -> Self {
        StabilityRelation {
            relation: Rc::new(move |d_in: &MI::Distance, d_out: &MO::Distance| Ok(relation(d_in, d_out))),
            forward_map: None,
            backward_map: None,
        }
    }
    pub fn new_fallible(relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static) -> Self {
        StabilityRelation { relation: Rc::new(relation), forward_map: None, backward_map: None }
    }
    pub fn new_all(
        relation: impl Fn(&MI::Distance, &MO::Distance) -> Fallible<bool> + 'static,
        forward_map: Option<impl Fn(&MI::Distance) -> Fallible<MO::Distance> + 'static>,
        backward_map: Option<impl Fn(&MO::Distance) -> Fallible<MI::Distance> + 'static>,
    ) -> Self {
        StabilityRelation {
            relation: Rc::new(relation),
            forward_map: forward_map.map(|h| Rc::new(h) as Rc<_>),
            backward_map: backward_map.map(|h| Rc::new(h) as Rc<_>),
        }
    }
    pub fn new_from_forward(
        forward_map: impl Fn(&MI::Distance) -> Fallible<MO::Distance> + Clone + 'static
    ) -> Self
        where MI::Distance: 'static, MO::Distance: 'static + PartialOrd + Clone {
        StabilityRelation::new_all(
            enclose!(forward_map, move |d_in: &MI::Distance, d_out: &MO::Distance|
                Ok(d_out.clone() >= forward_map(d_in)?)),
            Some(forward_map),
            None::<fn(&_)->_>
        )
    }
    pub fn new_from_constant(c: MO::Distance) -> Self where
        MI::Distance: InfCast<MO::Distance> + Clone,
        MO::Distance: DistanceConstant<MI::Distance> {
        StabilityRelation::new_all(
            // relation
            enclose!(c, move |d_in: &MI::Distance, d_out: &MO::Distance|
                Ok(d_out.clone() >= MO::Distance::inf_cast(d_in.clone())?.inf_mul(&c)?)),
            // forward map
            Some(enclose!(c, move |d_in: &MI::Distance|
                Ok(MO::Distance::inf_cast(d_in.clone())?.inf_mul(&c)?))),
            // backward map
            Some(enclose!(c, move |d_out: &MO::Distance|
                Ok(MI::Distance::inf_cast(d_out.inf_div(&c)?)?))))
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
#[derive(Clone)]
pub struct Measurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    pub input_domain: DI,
    pub output_domain: DO,
    pub function: Function<DI, DO>,
    pub input_metric: MI,
    pub output_measure: MO,
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
        Self {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure,
            privacy_relation,
        }
    }

    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        self.function.eval(arg)
    }

    pub fn check(&self, d_in: &MI::Distance, d_out: &MO::Distance) -> Fallible<bool> {
        self.privacy_relation.eval(d_in, d_out)
    }
}

/// A data transformation with certain stability characteristics.
#[derive(Clone)]
pub struct Transformation<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
    pub input_domain: DI,
    pub output_domain: DO,
    pub function: Function<DI, DO>,
    pub input_metric: MI,
    pub output_metric: MO,
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
        Self {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_metric,
            stability_relation,
        }
    }

    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        self.function.eval(arg)
    }

    pub fn check(&self, d_in: &MI::Distance, d_out: &MO::Distance) -> Fallible<bool> {
        self.stability_relation.eval(d_in, d_out)
    }
}


#[cfg(test)]
mod tests {
    use crate::dist::L1Distance;
    use crate::dom::AllDomain;
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_identity() {
        let input_domain = AllDomain::<i32>::new();
        let output_domain = AllDomain::<i32>::new();
        let function = Function::new(|arg: &i32| arg.clone());
        let input_metric = L1Distance::<i32>::default();
        let output_metric = L1Distance::<i32>::default();
        let stability_relation = StabilityRelation::new_from_constant(1);
        let identity = Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_relation);
        let arg = 99;
        let ret = identity.invoke(&arg).unwrap_test();
        assert_eq!(ret, 99);
    }
}

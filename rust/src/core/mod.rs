//! Core concepts of OpenDP.
//!
//! This module provides the central building blocks used throughout OpenDP:
//! * Measurement
//! * Transformation
//! * Domain
//! * Metric/Measure
//! * Function
//! * StabilityMap/PrivacyMap

// Generic legend
// M: Metric and Measure
// Q: Metric and Measure Carrier/Distance
// D: Domain
// T: Domain Carrier

// *I: Input
// *O: Output

// Ordering of generic arguments
// DI, DO, MI, MO, TI, TO, QI, QO

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(feature = "ffi")]
pub use ffi::*;

use std::rc::Rc;

use crate::domains::QueryableDomain;
use crate::error::*;
use crate::interactive::{Queryable, Context};
use crate::traits::{DistanceConstant, InfCast, InfMul, TotalOrd};
use std::fmt::Debug;

/// A set which constrains the input or output of a [`Function`].
///
/// Domains capture the notion of what values are allowed to be the input or output of a `Function`.
///
/// # Proof Definition
/// A type `Self` implements `Domain` iff it can represent a set of values that make up a domain.
pub trait Domain: Clone + PartialEq + Debug {
    /// The underlying type that the Domain specializes.
    /// This is the type of a member of a domain, where a domain is any data type that implements this trait.
    ///
    /// On any type `D` for which the `Domain` trait is implemented,
    /// the syntax `D::Carrier` refers to this associated type.
    /// For example, consider `D` to be `AllDomain<T>`, the domain of all non-null values of type `T`.
    /// The implementation of this trait for `AllDomain<T>` designates that `type Carrier = T`.
    /// Thus `AllDomain<T>::Carrier` is `T`.
    ///
    /// # Proof Definition
    /// `Self::Carrier` can represent all values in the set described by `Self`.
    type Carrier;

    /// Predicate to test an element for membership in the domain.
    /// Not all possible values of `::Carrier` are a member of the domain.
    ///
    /// # Proof Definition
    /// For all settings of the input parameters,
    /// returns `Err(e)` if the member check failed,
    /// or `Ok(out)`, where `out` is true if `val` is a member of `self`, otherwise false.
    ///
    /// # Notes
    /// It generally suffices to treat `Err(e)` as if `val` is not a member of the domain.
    /// It can be useful, however, to see richer debug information via `e` in the event of a failure.
    fn member(&self, val: &Self::Carrier) -> Fallible<bool>;

    /// A helper to communicate with a member of this domain.
    /// The members of most domains are not interactive, so the default implementation returns itself without changes
    fn inject_context<QD: 'static + Clone>(_member: &mut Self::Carrier, _context: Context, _max_usage: Option<QD>) {
    }
}

/// A mathematical function which maps values from an input [`Domain`] to an output [`Domain`].
pub struct Function<DI: Domain, DO: Domain> {
    pub function: Rc<dyn Fn(&DI::Carrier) -> Fallible<DO::Carrier>>,
}
impl<DI: Domain, DO: Domain> Clone for Function<DI, DO> {
    fn clone(&self) -> Self {
        Function {
            function: self.function.clone(),
        }
    }
}

impl<DI: Domain, DO: Domain> Function<DI, DO> {
    pub fn new(function: impl Fn(&DI::Carrier) -> DO::Carrier + 'static) -> Self {
        Self::new_fallible(move |arg| Ok(function(arg)))
    }

    pub fn new_fallible(
        function: impl Fn(&DI::Carrier) -> Fallible<DO::Carrier> + 'static,
    ) -> Self {
        Self {
            function: Rc::new(function),
        }
    }

    pub fn eval(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        (self.function)(arg)
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain> Function<DI, DO> {
    pub fn make_chain<XD: 'static + Domain>(
        function1: &Function<XD, DO>,
        function0: &Function<DI, XD>,
    ) -> Function<DI, DO> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        Self::new_fallible(move |arg| function1(&function0(arg)?))
    }
}

/// A representation of the distance between two elements in a set.
///
/// # Proof Definition
/// A type `Self` has an implementation for `Metric` iff it can represent a metric for quantifying distances between values in a set.
pub trait Metric: Default + Clone + PartialEq + Debug {
    /// # Proof Definition
    /// `Self::Distance` is a type that represents distances in terms of a metric `Self`.
    type Distance;
}

/// A representation of the distance between two distributions.
///
/// # Proof Definition
/// A type `Self` has an implementation for `Measure` iff it can represent a measure for quantifying distances between distributions.

pub trait Measure: Default + Clone + PartialEq + Debug {
    /// # Proof Definition
    /// `Self::Distance` is a type that represents distances in terms of a measure `Self`.
    type Distance;
}

/// A map evaluating the privacy of a [`Measurement`].
///
/// A `PrivacyMap` is implemented as a function that takes an input [`Metric::Distance`]
/// and returns the smallest upper bound on distances between output distributions on neighboring input datasets.
pub struct PrivacyMap<MI: Metric, MO: Measure>(
    pub Rc<dyn Fn(&MI::Distance) -> Fallible<MO::Distance>>,
);

impl<MI: Metric, MO: Measure> Clone for PrivacyMap<MI, MO> {
    fn clone(&self) -> Self {
        PrivacyMap(self.0.clone())
    }
}

impl<MI: Metric, MO: Measure> PrivacyMap<MI, MO> {
    pub fn new(map: impl Fn(&MI::Distance) -> MO::Distance + 'static) -> Self {
        PrivacyMap(Rc::new(move |d_in: &MI::Distance| Ok(map(d_in))))
    }
    pub fn new_fallible(map: impl Fn(&MI::Distance) -> Fallible<MO::Distance> + 'static) -> Self {
        PrivacyMap(Rc::new(map))
    }
    pub fn new_from_constant(c: MO::Distance) -> Self
    where
        MI::Distance: Clone,
        MO::Distance: DistanceConstant<MI::Distance>,
    {
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            MO::Distance::inf_cast(d_in.clone())?.inf_mul(&c)
        })
    }
    pub fn eval(&self, input_distance: &MI::Distance) -> Fallible<MO::Distance> {
        (self.0)(input_distance)
    }
}

impl<MI: 'static + Metric, MO: 'static + Measure> PrivacyMap<MI, MO> {
    pub fn make_chain<MX: 'static + Metric>(
        map1: &PrivacyMap<MX, MO>,
        map0: &StabilityMap<MI, MX>,
    ) -> Self {
        let map1 = map1.0.clone();
        let map0 = map0.0.clone();
        PrivacyMap(Rc::new(move |d_in: &MI::Distance| map1(&map0(d_in)?)))
    }
}

/// A map evaluating the stability of a [`Transformation`].
///
/// A `StabilityMap` is implemented as a function that takes an input [`Metric::Distance`],
/// and returns the smallest upper bound on distances between output datasets on neighboring input datasets.
pub struct StabilityMap<MI: Metric, MO: Metric>(
    pub Rc<dyn Fn(&MI::Distance) -> Fallible<MO::Distance>>,
);

impl<MI: Metric, MO: Metric> Clone for StabilityMap<MI, MO> {
    fn clone(&self) -> Self {
        StabilityMap(self.0.clone())
    }
}

impl<MI: Metric, MO: Metric> StabilityMap<MI, MO> {
    pub fn new(map: impl Fn(&MI::Distance) -> MO::Distance + 'static) -> Self {
        StabilityMap(Rc::new(move |d_in: &MI::Distance| Ok(map(d_in))))
    }
    pub fn new_fallible(map: impl Fn(&MI::Distance) -> Fallible<MO::Distance> + 'static) -> Self {
        StabilityMap(Rc::new(map))
    }
    pub fn new_from_constant(c: MO::Distance) -> Self
    where
        MI::Distance: Clone,
        MO::Distance: DistanceConstant<MI::Distance>,
    {
        StabilityMap::new_fallible(move |d_in: &MI::Distance| {
            MO::Distance::inf_cast(d_in.clone())?.inf_mul(&c)
        })
    }
    pub fn eval(&self, input_distance: &MI::Distance) -> Fallible<MO::Distance> {
        (self.0)(input_distance)
    }
}

impl<MI: 'static + Metric, MO: 'static + Metric> StabilityMap<MI, MO> {
    pub fn make_chain<MX: 'static + Metric>(
        map1: &StabilityMap<MX, MO>,
        map0: &StabilityMap<MI, MX>,
    ) -> Self {
        let map1 = map1.0.clone();
        let map0 = map0.0.clone();
        StabilityMap(Rc::new(move |d_in: &MI::Distance| map1(&map0(d_in)?)))
    }
}

/// A randomized mechanism with certain privacy characteristics.
///
/// The trait bounds provided by the Rust type system guarantee that:
/// * `input_domain` and `output_domain` are valid domains
/// * `input_metric` is a valid metric
/// * `output_measure` is a valid measure
///
/// It is, however, left to constructor functions to prove that:
/// * `input_metric` is compatible with `input_domain`
/// * `privacy_map` is a mapping from the input metric to the output measure
#[derive(Clone)]
pub struct Measurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    pub input_domain: DI,
    pub output_domain: DO,
    pub function: Function<DI, DO>,
    pub input_metric: MI,
    pub output_measure: MO,
    pub privacy_map: PrivacyMap<MI, MO>,
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> Measurement<DI, DO, MI, MO> {
    pub fn new(
        input_domain: DI,
        output_domain: DO,
        function: Function<DI, DO>,
        input_metric: MI,
        output_measure: MO,
        privacy_map: PrivacyMap<MI, MO>,
    ) -> Self {
        Self {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure,
            privacy_map,
        }
    }

    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        self.function.eval(arg)
    }

    pub fn map(&self, d_in: &MI::Distance) -> Fallible<MO::Distance> {
        self.privacy_map.eval(d_in)
    }

    pub fn check(&self, d_in: &MI::Distance, d_out: &MO::Distance) -> Fallible<bool>
    where
        MO::Distance: TotalOrd,
    {
        d_out.total_ge(&self.map(d_in)?)
    }
}

/// A data transformation with certain stability characteristics.
///
/// The trait bounds provided by the Rust type system guarantee that:
/// * `input_domain` and `output_domain` are valid domains
/// * `input_metric` and `output_metric` are valid metrics
///
/// It is, however, left to constructor functions to prove that:
/// * metrics are compatible with domains
/// * `function` is a mapping from the input domain to the output domain
/// * `stability_map` is a mapping from the input metric to the output metric
#[derive(Clone)]
pub struct Transformation<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
    pub input_domain: DI,
    pub output_domain: DO,
    pub function: Function<DI, DO>,
    pub input_metric: MI,
    pub output_metric: MO,
    pub stability_map: StabilityMap<MI, MO>,
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> Transformation<DI, DO, MI, MO> {
    pub fn new(
        input_domain: DI,
        output_domain: DO,
        function: Function<DI, DO>,
        input_metric: MI,
        output_metric: MO,
        stability_map: StabilityMap<MI, MO>,
    ) -> Self {
        Self {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_metric,
            stability_map,
        }
    }

    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        self.function.eval(arg)
    }

    pub fn map(&self, d_in: &MI::Distance) -> Fallible<MO::Distance> {
        self.stability_map.eval(d_in)
    }

    pub fn check(&self, d_in: &MI::Distance, d_out: &MO::Distance) -> Fallible<bool>
    where
        MO::Distance: TotalOrd,
    {
        d_out.total_ge(&self.map(d_in)?)
    }
}

#[derive(Clone)]
pub struct Odometer<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    pub input_domain: DI,
    pub output_domain: DO,
    pub function: Function<DI, DO>,
    pub input_metric: MI,
    pub output_measure: MO,
    pub d_in: MI::Distance,
}

impl<DI: Domain, Q: 'static + Clone, DA: Domain + 'static, MI: Metric, MO: Measure>
    Odometer<DI, QueryableDomain<Q, DA>, MI, MO>
    where DA::Carrier: 'static
{
    pub fn new(
        input_domain: DI,
        output_domain: QueryableDomain<Q, DA>,
        function: Function<DI, QueryableDomain<Q, DA>>,
        input_metric: MI,
        output_measure: MO,
        d_in: MI::Distance
    ) -> Self {
        Self {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure,
            d_in
        }
    }

    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<Queryable<Q, DA>> {
        self.function.eval(arg)
    }
}

#[cfg(test)]
mod tests {
    use crate::domains::AllDomain;
    use crate::error::ExplainUnwrap;
    use crate::metrics::L1Distance;

    use super::*;

    #[test]
    fn test_identity() {
        let input_domain = AllDomain::<i32>::new();
        let output_domain = AllDomain::<i32>::new();
        let function = Function::new(|arg: &i32| arg.clone());
        let input_metric = L1Distance::<i32>::default();
        let output_metric = L1Distance::<i32>::default();
        let stability_map = StabilityMap::new_from_constant(1);
        let identity = Transformation::new(
            input_domain,
            output_domain,
            function,
            input_metric,
            output_metric,
            stability_map,
        );
        let arg = 99;
        let ret = identity.invoke(&arg).unwrap_test();
        assert_eq!(ret, 99);
    }
}

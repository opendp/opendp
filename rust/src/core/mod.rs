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

use std::sync::Arc;

use crate::error::*;
use crate::traits::{DistanceConstant, InfCast, InfMul, ProductOrd};
use num::Zero;
use std::fmt::{Debug, Display};

/// A set which constrains the input or output of a [`Function`].
///
/// Domains capture the notion of what values are allowed to be the input or output of a `Function`.
///
/// # Proof Definition
/// A type `Self` implements `Domain` iff it can represent a set of values that make up a domain.
pub trait Domain: Clone + PartialEq + Debug + Send + Sync {
    /// The underlying type that the Domain specializes.
    /// This is the type of a member of a domain, where a domain is any data type that implements this trait.
    ///
    /// On any type `D` for which the `Domain` trait is implemented,
    /// the syntax `D::Carrier` refers to this associated type.
    ///
    /// For example, consider `D` to be `AtomDomain<T>`, the domain of all non-null values of type `T`.
    /// The implementation of this trait for `AtomDomain<T>` designates that `type Carrier = T`.
    /// Thus `AtomDomain<T>::Carrier` is `T`.
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
}

/// A mathematical function.
pub struct Function<TI, TO> {
    pub function: Arc<dyn Fn(&TI) -> Fallible<TO> + Send + Sync>,
}
impl<TI, TO> Clone for Function<TI, TO> {
    fn clone(&self) -> Self {
        Function {
            function: self.function.clone(),
        }
    }
}

impl<TI, TO> Function<TI, TO> {
    pub fn new(function: impl Fn(&TI) -> TO + 'static + Send + Sync) -> Self {
        Self::new_fallible(move |arg| Ok(function(arg)))
    }

    pub fn new_fallible(function: impl Fn(&TI) -> Fallible<TO> + 'static + Send + Sync) -> Self {
        Self {
            function: Arc::new(function),
        }
    }

    pub fn eval(&self, arg: &TI) -> Fallible<TO> {
        (self.function)(arg)
    }
}

impl<TI: 'static, TO: 'static> Function<TI, TO> {
    pub fn make_chain<TX: 'static>(
        function1: &Function<TX, TO>,
        function0: &Function<TI, TX>,
    ) -> Function<TI, TO> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        Self::new_fallible(move |arg| function1(&function0(arg)?))
    }
}

/// A representation of the distance between two elements in a set.
///
/// # Proof Definition
/// A type `Self` has an implementation for `Metric` iff it can represent a metric for quantifying distances between values in a set.
pub trait Metric: Default + Clone + PartialEq + Debug + Send + Sync {
    /// # Proof Definition
    /// `Self::Distance` is a type that represents distances in terms of a metric `Self`.
    type Distance;
}

/// A representation of the distance between two distributions.
///
/// # Proof Definition
/// A type `Self` has an implementation for `Measure` iff it can represent a measure for quantifying distances between distributions.

pub trait Measure: Default + Clone + PartialEq + Debug + Send + Sync {
    /// # Proof Definition
    /// `Self::Distance` is a type that represents distances in terms of a measure `Self`.
    type Distance;
}

/// A map evaluating the privacy of a [`Measurement`].
///
/// A `PrivacyMap` is implemented as a function that takes an input [`Metric::Distance`]
/// and returns the smallest upper bound on distances between output distributions on neighboring input datasets.
pub struct PrivacyMap<MI: Metric, MO: Measure>(
    pub Arc<dyn Fn(&MI::Distance) -> Fallible<MO::Distance> + Send + Sync>,
);

impl<MI: Metric, MO: Measure> Clone for PrivacyMap<MI, MO> {
    fn clone(&self) -> Self {
        PrivacyMap(self.0.clone())
    }
}

impl<MI: Metric, MO: Measure> PrivacyMap<MI, MO> {
    pub fn new(map: impl Fn(&MI::Distance) -> MO::Distance + 'static + Send + Sync) -> Self {
        PrivacyMap(Arc::new(move |d_in: &MI::Distance| Ok(map(d_in))))
    }
    pub fn new_fallible(
        map: impl Fn(&MI::Distance) -> Fallible<MO::Distance> + 'static + Send + Sync,
    ) -> Self {
        PrivacyMap(Arc::new(map))
    }
    pub fn new_from_constant(c: MO::Distance) -> Self
    where
        MI::Distance: Clone,
        MO::Distance: DistanceConstant<MI::Distance> + Display,
    {
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            if c < MO::Distance::zero() {
                return fallible!(FailedMap, "constant ({}) must be non-negative", c);
            }
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
        PrivacyMap(Arc::new(move |d_in: &MI::Distance| map1(&map0(d_in)?)))
    }
}

/// A map evaluating the stability of a [`Transformation`].
///
/// A `StabilityMap` is implemented as a function that takes an input [`Metric::Distance`],
/// and returns the smallest upper bound on distances between output datasets on neighboring input datasets.
pub struct StabilityMap<MI: Metric, MO: Metric>(
    pub Arc<dyn Fn(&MI::Distance) -> Fallible<MO::Distance> + Send + Sync>,
);

impl<MI: Metric, MO: Metric> Clone for StabilityMap<MI, MO> {
    fn clone(&self) -> Self {
        StabilityMap(self.0.clone())
    }
}

impl<MI: Metric, MO: Metric> StabilityMap<MI, MO> {
    pub fn new(map: impl Fn(&MI::Distance) -> MO::Distance + 'static + Send + Sync) -> Self {
        StabilityMap(Arc::new(move |d_in: &MI::Distance| Ok(map(d_in))))
    }
    pub fn new_fallible(
        map: impl Fn(&MI::Distance) -> Fallible<MO::Distance> + 'static + Send + Sync,
    ) -> Self {
        StabilityMap(Arc::new(map))
    }
    pub fn new_from_constant(c: MO::Distance) -> Self
    where
        MI::Distance: Clone,
        MO::Distance: DistanceConstant<MI::Distance> + Display,
    {
        StabilityMap::new_fallible(move |d_in: &MI::Distance| {
            if c < MO::Distance::zero() {
                return fallible!(FailedMap, "constant ({}) must be non-negative", c);
            }
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
        StabilityMap(Arc::new(move |d_in: &MI::Distance| map1(&map0(d_in)?)))
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
#[readonly::make]
pub struct Measurement<DI: Domain, TO, MI: Metric, MO: Measure> {
    pub input_domain: DI,
    pub function: Function<DI::Carrier, TO>,
    pub input_metric: MI,
    pub output_measure: MO,
    pub privacy_map: PrivacyMap<MI, MO>,
}

impl<DI: Domain, TO, MI: Metric, MO: Measure> Clone for Measurement<DI, TO, MI, MO> {
    fn clone(&self) -> Self {
        Self {
            input_domain: self.input_domain.clone(),
            function: self.function.clone(),
            input_metric: self.input_metric.clone(),
            output_measure: self.output_measure.clone(),
            privacy_map: self.privacy_map.clone(),
        }
    }
}

impl<DI: Domain, TO, MI: Metric, MO: Measure> Measurement<DI, TO, MI, MO>
where
    (DI, MI): MetricSpace,
{
    pub fn new(
        input_domain: DI,
        function: Function<DI::Carrier, TO>,
        input_metric: MI,
        output_measure: MO,
        privacy_map: PrivacyMap<MI, MO>,
    ) -> Fallible<Self> {
        (input_domain.clone(), input_metric.clone()).check_space()?;
        Ok(Self {
            input_domain,
            function,
            input_metric,
            output_measure,
            privacy_map,
        })
    }

    pub(crate) fn with_map<MI2: Metric, MO2: Measure>(
        &self,
        input_metric: MI2,
        output_metric: MO2,
        privacy_map: PrivacyMap<MI2, MO2>,
    ) -> Fallible<Measurement<DI, TO, MI2, MO2>>
    where
        (DI, MI2): MetricSpace,
    {
        Measurement::new(
            self.input_domain.clone(),
            self.function.clone(),
            input_metric,
            output_metric,
            privacy_map,
        )
    }

    pub fn input_space(&self) -> (DI, MI) {
        (self.input_domain.clone(), self.input_metric.clone())
    }
}

impl<DI: Domain, TO, MI: Metric, MO: Measure> Measurement<DI, TO, MI, MO> {
    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<TO> {
        self.function.eval(arg)
    }

    pub fn map(&self, d_in: &MI::Distance) -> Fallible<MO::Distance> {
        self.privacy_map.eval(d_in)
    }

    pub fn check(&self, d_in: &MI::Distance, d_out: &MO::Distance) -> Fallible<bool>
    where
        MO::Distance: ProductOrd,
    {
        d_out.total_ge(&self.map(d_in)?)
    }
}

impl<DI: Domain, TO, MI: Metric, MO: Measure> Debug for Measurement<DI, TO, MI, MO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Measurement")
            .field("input_domain", &self.input_domain)
            .field("input_metric", &self.input_metric)
            .field("output_measure", &self.output_measure)
            .finish()
    }
}

pub trait MetricSpace {
    fn check_space(&self) -> Fallible<()>;
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
#[readonly::make]
pub struct Transformation<DI: Domain, DO: Domain, MI: Metric, MO: Metric> {
    pub input_domain: DI,
    pub output_domain: DO,
    pub function: Function<DI::Carrier, DO::Carrier>,
    pub input_metric: MI,
    pub output_metric: MO,
    pub stability_map: StabilityMap<MI, MO>,
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> Transformation<DI, DO, MI, MO>
where
    (DI, MI): MetricSpace,
    (DO, MO): MetricSpace,
{
    pub fn new(
        input_domain: DI,
        output_domain: DO,
        function: Function<DI::Carrier, DO::Carrier>,
        input_metric: MI,
        output_metric: MO,
        stability_map: StabilityMap<MI, MO>,
    ) -> Fallible<Self> {
        (input_domain.clone(), input_metric.clone()).check_space()?;
        (output_domain.clone(), output_metric.clone()).check_space()?;
        Ok(Self {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_metric,
            stability_map,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn with_map<MI2: Metric, MO2: Metric>(
        &self,
        input_metric: MI2,
        output_metric: MO2,
        privacy_map: StabilityMap<MI2, MO2>,
    ) -> Fallible<Transformation<DI, DO, MI2, MO2>>
    where
        (DI, MI2): MetricSpace,
        (DO, MO2): MetricSpace,
    {
        Transformation::new(
            self.input_domain.clone(),
            self.output_domain.clone(),
            self.function.clone(),
            input_metric,
            output_metric,
            privacy_map,
        )
    }

    pub fn input_space(&self) -> (DI, MI) {
        (self.input_domain.clone(), self.input_metric.clone())
    }

    pub fn output_space(&self) -> (DO, MO) {
        (self.output_domain.clone(), self.output_metric.clone())
    }
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> Transformation<DI, DO, MI, MO> {
    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        self.function.eval(arg)
    }

    pub fn map(&self, d_in: &MI::Distance) -> Fallible<MO::Distance> {
        self.stability_map.eval(d_in)
    }

    pub fn check(&self, d_in: &MI::Distance, d_out: &MO::Distance) -> Fallible<bool>
    where
        MO::Distance: ProductOrd,
    {
        d_out.total_ge(&self.map(d_in)?)
    }
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> Debug for Transformation<DI, DO, MI, MO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transformation")
            .field("input_domain", &self.input_domain)
            .field("input_metric", &self.input_metric)
            .field("output_domain", &self.output_domain)
            .field("output_metric", &self.output_metric)
            .finish()
    }
}

#[cfg(test)]
mod test;

#[cfg(feature = "partials")]
mod partials {
    pub use super::*;

    pub struct PartialTransformation<DI: Domain, DO: Domain, MI: Metric, MO: Metric>(
        Box<dyn FnOnce(DI, MI) -> Fallible<Transformation<DI, DO, MI, MO>>>,
    );

    impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> PartialTransformation<DI, DO, MI, MO>
    where
        (DI, MI): MetricSpace,
        (DO, MO): MetricSpace,
    {
        pub fn new(
            partial: impl FnOnce(DI, MI) -> Fallible<Transformation<DI, DO, MI, MO>> + 'static,
        ) -> Self {
            Self(Box::new(partial))
        }
        pub fn fix(
            self,
            input_domain: DI,
            input_metric: MI,
        ) -> Fallible<Transformation<DI, DO, MI, MO>> {
            (self.0)(input_domain, input_metric)
        }
    }

    pub struct PartialMeasurement<DI: Domain, TO, MI: Metric, MO: Measure>(
        Box<dyn FnOnce(DI, MI) -> Fallible<Measurement<DI, TO, MI, MO>>>,
    );

    impl<DI: Domain, TO, MI: Metric, MO: Measure> PartialMeasurement<DI, TO, MI, MO>
    where
        (DI, MI): MetricSpace,
    {
        pub fn new(
            partial: impl FnOnce(DI, MI) -> Fallible<Measurement<DI, TO, MI, MO>> + 'static,
        ) -> Self {
            Self(Box::new(partial))
        }
        pub fn fix(
            self,
            input_domain: DI,
            input_metric: MI,
        ) -> Fallible<Measurement<DI, TO, MI, MO>> {
            (self.0)(input_domain, input_metric)
        }
    }
}

#[cfg(feature = "partials")]
pub use partials::*;

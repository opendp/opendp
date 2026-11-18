//! Actual OpenDP domain/metric/measure traits and callback containers.
//!
//! Kept in one source module so a verifier can import the production
//! definitions rather than introduce parallel callback representations.
#[cfg(not(verus_only))]
use super::StabilityMap;
#[cfg(not(verus_only))]
use crate::error::*;
#[cfg(verus_only)]
type Fallible<T> = Result<T, ()>;
#[cfg(not(verus_only))]
use crate::traits::{DistanceConstant, InfCast, InfMul};
#[cfg(not(verus_only))]
use num::Zero;
use std::fmt::{Debug, Display};
use std::sync::Arc;
#[cfg(verus_only)]
use vstd::prelude::*;

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
    pub function: Arc<dyn Fn(&TI) -> Fallible<TO>>,
}
impl<TI, TO> Clone for Function<TI, TO> {
    fn clone(&self) -> Self {
        Function {
            function: self.function.clone(),
        }
    }
}

impl<TI, TO> Function<TI, TO> {
    pub fn new(function: impl Fn(&TI) -> TO + 'static) -> Self {
        Self::new_fallible(move |arg| Ok(function(arg)))
    }

    #[cfg_attr(verus_only, verus_verify(external_body))]
    pub fn new_fallible(function: impl Fn(&TI) -> Fallible<TO> + 'static) -> Self {
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
pub trait Metric: Clone + PartialEq + Debug {
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
    pub Arc<dyn Fn(&MI::Distance) -> Fallible<MO::Distance>>,
);

impl<MI: Metric, MO: Measure> Clone for PrivacyMap<MI, MO> {
    fn clone(&self) -> Self {
        PrivacyMap(self.0.clone())
    }
}

impl<MI: Metric, MO: Measure> PrivacyMap<MI, MO> {
    #[cfg_attr(verus_only, verus_verify(external_body))]
    pub fn new(map: impl Fn(&MI::Distance) -> MO::Distance + 'static) -> Self {
        PrivacyMap(Arc::new(move |d_in: &MI::Distance| Ok(map(d_in))))
    }
    pub fn new_fallible(map: impl Fn(&MI::Distance) -> Fallible<MO::Distance> + 'static) -> Self {
        PrivacyMap(Arc::new(map))
    }

    /// # Proof Definition
    /// Creates a privacy map from `d_in` to `d_out`
    /// where `d_out = MO.inf_cast(d_in).inf_mul(c) >= d_in * c`,
    /// or returns an error.
    #[cfg(not(verus_only))]
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

#[cfg(not(verus_only))]
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


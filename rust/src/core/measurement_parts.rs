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
#[cfg_attr(verus_only, verus_verify)]
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
#[cfg_attr(verus_only, verifier::reject_recursive_types(TI))]
#[cfg_attr(verus_only, verifier::reject_recursive_types(TO))]
#[cfg_attr(verus_only, verus_verify(external_body))]
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

// Verified code can reason about the exact input/output relation of the
// closure stored inside Function, without reasoning about Arc allocation.
// This is only a *pointwise* contract, not a probability law.
#[cfg(verus_only)]
verus! {

/// The actual distributional expectation of the erased executable Function.
/// This interpretation is opaque until a concrete sampling/erasure
/// constructor establishes its law from the trusted entropy primitive.
pub uninterp spec fn function_expectation<TI, TO>(
    function: &Function<TI, TO>,
    input: &TI,
    payoff: spec_fn(Result<TO, ()>) -> real,
) -> real;
pub uninterp spec fn function_call_requires<TI, TO>(
    function: &Function<TI, TO>,
    input: &TI,
) -> bool;

pub uninterp spec fn function_call_ensures<TI, TO>(
    function: &Function<TI, TO>,
    input: &TI,
    value: Result<TO, ()>,
) -> bool;
}

#[cfg_attr(verus_only, verus_verify)]
impl<TI, TO> Function<TI, TO> {
    #[cfg_attr(verus_only, verus_verify(external))]
    pub fn new(function: impl Fn(&TI) -> TO + 'static) -> Self {
        Self::new_fallible(move |arg| Ok(function(arg)))
    }

    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures
            forall|input: &TI|
                #[trigger] function_call_requires(&result, input)
                    == call_requires(function, (input,)),
            forall|input: &TI, value: Fallible<TO>|
                #[trigger] function_call_ensures(&result, input, value)
                    == call_ensures(function, (input,), value),
    ))]
    pub fn new_fallible(function: impl Fn(&TI) -> Fallible<TO> + 'static) -> Self {
        #[cfg(not(verus_only))]
        {
            Self {
                function: Arc::new(function),
            }
        }
        #[cfg(verus_only)]
        {
            unimplemented!()
        }
    }

    /// Erase a named method of a concrete operator. This is a narrow
    /// trusted Arc-dispatch correspondence, not a privacy assumption.
    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures
            forall|arg: &TI|
                #[trigger] function_call_requires(&result, arg)
                    == call_requires(invoke, (&operator, arg)),
            forall|arg: &TI, out: Fallible<TO>|
                #[trigger] function_call_ensures(&result, arg, out)
                    == call_ensures(invoke, (&operator, arg), out),
    ))]
    pub(crate) fn new_from_operator<Op, F>(
        operator: Op,
        invoke: F,
    ) -> Self
    where
        Op: Copy + 'static,
        F: Fn(&Op, &TI) -> Fallible<TO> + Copy + 'static,
    {
        #[cfg(not(verus_only))]
        { Self::new_fallible(move |arg| invoke(&operator, arg)) }
        #[cfg(verus_only)]
        { unimplemented!() }
    }

    /// Erase an exact one-word sampler and deterministic postprocessor.
    /// Trusted: Arc dispatch preserves the expectation of the executable
    /// sample_word_then wrapper. The caller must prove its postprocessor's
    /// actual outputs match the supplied ghost result model.
    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        requires forall|arg: &TI, word: Result<u64, ()>| {
            &&& call_requires(postprocess, (&operator, arg, word))
            &&& forall|output: Fallible<TO>|
                call_ensures(postprocess, (&operator, arg, word), output)
                    ==> output == model@(*arg, word)
        },
        ensures
            forall|arg: &TI| #[trigger] function_call_requires(&result, arg),
            forall|arg: &TI, output: Fallible<TO>|
                #[trigger] function_call_ensures(&result, arg, output)
                    ==> exists|word: Result<u64, ()>|
                        call_ensures(postprocess, (&operator, arg, word), output),
            forall|arg: &TI, payoff: spec_fn(Result<TO, ()>) -> real|
                #[trigger] function_expectation(&result, arg, payoff)
                    == crate::uniform_word::one_word_expectation(*arg, model@, payoff),
    ))]
    pub(crate) fn new_from_one_word_operator<Op, F>(
        operator: Op,
        postprocess: F,
        #[cfg(verus_only)]
        model: Ghost<crate::uniform_word::OneWordModel<TI, TO>>,
    ) -> Self
    where
        Op: Copy + 'static,
        TI: Copy,
        F: Fn(&Op, &TI, Fallible<u64>) -> Fallible<TO> + Copy + 'static,
    {
        #[cfg(not(verus_only))]
        {
            Self::new_fallible(move |arg| {
                crate::traits::samplers::sample_word_then(&operator, arg, postprocess)
            })
        }
        #[cfg(verus_only)]
        {
            unimplemented!()
        }
    }

    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        requires function_call_requires(self, arg),
        ensures function_call_ensures(self, arg, result),
    ))]
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
#[cfg_attr(verus_only, verus_verify)]
pub trait Metric: Clone + PartialEq + Debug {
    /// # Proof Definition
    /// `Self::Distance` is a type that represents distances in terms of a metric `Self`.
    type Distance;
}

/// A representation of the distance between two distributions.
///
/// # Proof Definition
/// A type `Self` has an implementation for `Measure` iff it can represent a measure for quantifying distances between distributions.

#[cfg_attr(verus_only, verus_verify)]
pub trait Measure: Default + Clone + PartialEq + Debug {
    /// # Proof Definition
    /// `Self::Distance` is a type that represents distances in terms of a measure `Self`.
    type Distance;
}

/// A map evaluating the privacy of a [`Measurement`].
///
/// A `PrivacyMap` is implemented as a function that takes an input [`Metric::Distance`]
/// and returns the smallest upper bound on distances between output distributions on neighboring input datasets.
#[cfg_attr(verus_only, verifier::reject_recursive_types(MI))]
#[cfg_attr(verus_only, verifier::reject_recursive_types(MO))]
#[cfg_attr(verus_only, verus_verify(external_body))]
pub struct PrivacyMap<MI: Metric, MO: Measure>(
    pub Arc<dyn Fn(&MI::Distance) -> Fallible<MO::Distance>>,
);

impl<MI: Metric, MO: Measure> Clone for PrivacyMap<MI, MO> {
    fn clone(&self) -> Self {
        PrivacyMap(self.0.clone())
    }
}

// A narrow model of the erased callback's *possible return values*.
// This never asserts distributional privacy, nor does it assume that a
// callback is deterministic. Its only trusted correspondence is forwarding.
#[cfg(verus_only)]
verus! {
pub uninterp spec fn privacy_map_call_requires<MI: Metric, MO: Measure>(
    map: &PrivacyMap<MI, MO>,
    d: &MI::Distance,
) -> bool;

pub uninterp spec fn privacy_map_call_ensures<MI: Metric, MO: Measure>(
    map: &PrivacyMap<MI, MO>,
    d: &MI::Distance,
    value: Result<MO::Distance, ()>,
) -> bool;
}

#[cfg_attr(verus_only, verus_verify)]
impl<MI: Metric, MO: Measure> PrivacyMap<MI, MO> {
    #[cfg_attr(verus_only, verus_verify(external_body))]
    pub fn new(map: impl Fn(&MI::Distance) -> MO::Distance + 'static) -> Self {
        #[cfg(not(verus_only))]
        {
            PrivacyMap(Arc::new(move |d_in: &MI::Distance| Ok(map(d_in))))
        }
        #[cfg(verus_only)]
        { unimplemented!() }
    }
    /// Erasure of a fallible privacy-map callback. Verus treats only
    /// allocation and dynamic dispatch as external; no privacy claim is
    /// attached to this function.
    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures
            forall|d: &MI::Distance|
                #[trigger] privacy_map_call_requires(&result, d)
                    == call_requires(map, (d,)),
            forall|d: &MI::Distance, value: Fallible<MO::Distance>|
                #[trigger] privacy_map_call_ensures(&result, d, value)
                    == call_ensures(map, (d,), value),
    ))]
    pub fn new_fallible(map: impl Fn(&MI::Distance) -> Fallible<MO::Distance> + 'static) -> Self {
        #[cfg(not(verus_only))]
        {
            PrivacyMap(Arc::new(map))
        }
        #[cfg(verus_only)]
        { unimplemented!() }
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
    /// Erase a named operator privacy-map method. The only trusted
    /// obligation is exact forwarding of possible outcomes, including errors.
    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures
            forall|d: &MI::Distance|
                #[trigger] privacy_map_call_requires(&result, d)
                    == call_requires(map, (&operator, d)),
            forall|d: &MI::Distance, out: Fallible<MO::Distance>|
                #[trigger] privacy_map_call_ensures(&result, d, out)
                    == call_ensures(map, (&operator, d), out),
    ))]
    pub(crate) fn new_from_operator<Op, P>(
        operator: Op,
        map: P,
    ) -> Self
    where
        Op: Copy + 'static,
        P: Fn(&Op, &MI::Distance) -> Fallible<MO::Distance> + Copy + 'static,
    {
        #[cfg(not(verus_only))]
        { Self::new_fallible(move |d| map(&operator, d)) }
        #[cfg(verus_only)]
        { unimplemented!() }
    }

    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        requires privacy_map_call_requires(self, input_distance),
        ensures privacy_map_call_ensures(self, input_distance, result),
    ))]
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


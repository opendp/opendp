//! The production OpenDP Measurement type and constructor implementations.
//! Extracted unchanged from `core/mod.rs` so the Verus harness can import
//! the same source rather than a duplicate model.
use super::{Domain, Function, Measure, Metric, PrivacyMap};
#[cfg(not(verus_only))]
use crate::error::*;
#[cfg(verus_only)]
type Fallible<T> = Result<T, ()>;
#[cfg(not(verus_only))]
use crate::traits::ProductOrd;
use std::fmt::Debug;
#[cfg(verus_only)]
use vstd::prelude::*;

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
#[cfg_attr(not(verus_only), readonly::make)]
#[cfg_attr(verus_only, verifier::reject_recursive_types(DI))]
#[cfg_attr(verus_only, verifier::reject_recursive_types(MI))]
#[cfg_attr(verus_only, verifier::reject_recursive_types(MO))]
#[cfg_attr(verus_only, verifier::reject_recursive_types(TO))]
#[cfg_attr(verus_only, verus_verify)]
pub struct Measurement<DI: Domain, MI: Metric, MO: Measure, TO> {
    pub input_domain: DI,
    pub input_metric: MI,
    pub output_measure: MO,
    pub function: Function<DI::Carrier, TO>,
    pub privacy_map: PrivacyMap<MI, MO>,
}

impl<DI: Domain, MI: Metric, MO: Measure, TO> Clone for Measurement<DI, MI, MO, TO> {
    fn clone(&self) -> Self {
        Self {
            input_domain: self.input_domain.clone(),
            input_metric: self.input_metric.clone(),
            output_measure: self.output_measure.clone(),
            function: self.function.clone(),
            privacy_map: self.privacy_map.clone(),
        }
    }
}

#[cfg_attr(verus_only, verus_verify)]
impl<DI: Domain, MI: Metric, MO: Measure, TO> Measurement<DI, MI, MO, TO>
where
    (DI, MI): MetricSpace,
{
    /// Common executable assembly for the legacy and proof-gated paths.
    /// The body is verified; it checks compatibility and returns the exact
    /// supplied descriptors, function, and privacy map. No privacy is assumed.
    #[cfg_attr(verus_only, verus_verify)]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures match result {
            Ok(meas) => {
                &&& meas.input_domain == input_domain
                &&& meas.input_metric == input_metric
                &&& meas.output_measure == output_measure
                &&& meas.function == function
                &&& meas.privacy_map == privacy_map
            },
            Err(_) => true,
        },
    ))]
    fn construct(
        input_domain: DI,
        input_metric: MI,
        output_measure: MO,
        function: Function<DI::Carrier, TO>,
        privacy_map: PrivacyMap<MI, MO>,
    ) -> Fallible<Self> {
        (input_domain.clone(), input_metric.clone()).check_space()?;
        Ok(Self {
            input_domain,
            input_metric,
            output_measure,
            function,
            privacy_map,
        })
    }

    /// Legacy constructor: callers remain responsible for validity.
    #[cfg_attr(verus_only, verus_verify(external))]
    pub fn new(
        input_domain: DI,
        input_metric: MI,
        output_measure: MO,
        function: Function<DI::Carrier, TO>,
        privacy_map: PrivacyMap<MI, MO>,
    ) -> Fallible<Self> {
        Self::construct(
            input_domain, input_metric, output_measure, function, privacy_map,
        )
    }

    /// Construct a measurement whose sampling function consists of one
    /// uniform-word draw followed by the caller's verified deterministic
    /// postprocessor. The one-word distributional correspondence is captured
    /// *before* Arc erasure, rather than inferred from an arbitrary Fn.
    ///
    /// TODO: the complete descriptor/metric/privacy validity predicate must
    /// become a constructor precondition; this method currently establishes
    /// the exact distributional callback correspondence, not DP by itself.
    #[cfg_attr(verus_only, verus_verify)]
    #[cfg_attr(verus_only, verus_spec(result =>
        with Ghost(model): Ghost<crate::uniform_word::OneWordModel<DI::Carrier, TO>>
        requires super::measurement_parts::exact_word_postprocessing(
            operator, postprocess, model),
        ensures match result {
            Ok(meas) => {
                &&& meas.input_domain == input_domain
                &&& meas.input_metric == input_metric
                &&& meas.output_measure == output_measure
                &&& forall|arg: &DI::Carrier|
                    #[trigger] super::measurement_parts::function_call_requires(
                        &meas.function, arg)
                &&& forall|arg: &DI::Carrier, out: Fallible<TO>|
                    #[trigger] super::measurement_parts::function_call_ensures(
                        &meas.function, arg, out)
                        ==> exists|word: Result<u64, ()>|
                            call_ensures(postprocess, (&operator, arg, word), out)
                &&& forall|arg: &DI::Carrier,
                              payoff: spec_fn(Result<TO, ()>) -> real|
                    #[trigger] super::measurement_parts::function_expectation(
                        &meas.function, arg, payoff)
                        == crate::uniform_word::one_word_expectation(
                            *arg, model, payoff)
                &&& forall|d: &MI::Distance|
                    #[trigger] super::measurement_parts::privacy_map_call_requires(
                        &meas.privacy_map, d)
                        == call_requires(privacy_map, (&operator, d))
                &&& forall|d: &MI::Distance, out: Fallible<MO::Distance>|
                    #[trigger] super::measurement_parts::privacy_map_call_ensures(
                        &meas.privacy_map, d, out)
                        == call_ensures(privacy_map, (&operator, d), out)
            },
            Err(_) => true,
        },
    ))]
    pub(crate) fn new_certified<Op, F, P>(
        input_domain: DI,
        input_metric: MI,
        output_measure: MO,
        operator: Op,
        postprocess: F,
        privacy_map: P,
    ) -> Fallible<Self>
    where
        Op: Copy + 'static,
        DI::Carrier: Copy,
        F: Fn(&Op, &DI::Carrier, Fallible<u64>) -> Fallible<TO> + Copy + 'static,
        P: Fn(&Op, &MI::Distance) -> Fallible<MO::Distance> + Copy + 'static,
    {
        #[cfg(verus_only)]
        let function = {
            proof_with! { Ghost(model) }
            Function::new_from_one_word_operator(operator, postprocess)
        };
        #[cfg(not(verus_only))]
        let function = Function::new_from_one_word_operator(operator, postprocess);
        let privacy_map = PrivacyMap::new_from_operator(operator, privacy_map);
        Self::construct(
            input_domain, input_metric, output_measure, function, privacy_map,
        )
    }

    #[allow(dead_code)]
    #[cfg(not(verus_only))]
    pub(crate) fn with_map<MI2: Metric, MO2: Measure>(
        &self,
        input_metric: MI2,
        output_metric: MO2,
        privacy_map: PrivacyMap<MI2, MO2>,
    ) -> Fallible<Measurement<DI, MI2, MO2, TO>>
    where
        (DI, MI2): MetricSpace,
    {
        Measurement::new(
            self.input_domain.clone(),
            input_metric,
            output_metric,
            self.function.clone(),
            privacy_map,
        )
    }

    #[cfg_attr(verus_only, verus_verify(external))]
    pub fn input_space(&self) -> (DI, MI) {
        (self.input_domain.clone(), self.input_metric.clone())
    }
}

impl<DI: Domain, MI: Metric, MO: Measure, TO> Measurement<DI, MI, MO, TO> {
    pub fn invoke(&self, arg: &DI::Carrier) -> Fallible<TO> {
        self.function.eval(arg)
    }

    pub fn map(&self, d_in: &MI::Distance) -> Fallible<MO::Distance> {
        self.privacy_map.eval(d_in)
    }

    #[cfg(not(verus_only))]
    pub fn check(&self, d_in: &MI::Distance, d_out: &MO::Distance) -> Fallible<bool>
    where
        MO::Distance: ProductOrd,
    {
        d_out.total_ge(&self.map(d_in)?)
    }
}

impl<DI: Domain, MI: Metric, MO: Measure, TO> Debug for Measurement<DI, MI, MO, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Measurement")
            .field("input_domain", &self.input_domain)
            .field("input_metric", &self.input_metric)
            .field("output_measure", &self.output_measure)
            .finish()
    }
}

#[cfg_attr(verus_only, verus_verify)]
pub trait MetricSpace {
    /// # Proof Definition
    /// Returns Ok(()) if self forms a valid metric space.
    fn check_space(&self) -> Fallible<()>;
}


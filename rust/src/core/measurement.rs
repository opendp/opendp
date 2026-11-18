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
#[cfg(not(verus_only))]
use crate::traits::samplers::OneWordProgram;
#[cfg(verus_only)]
use crate::uniform_word::OneWordProgram;
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::{CertifiedDomain, CertifiedMetric, CertifiedMeasure};
#[cfg(not(verus_only))]
use super::{CertifiedDomain, CertifiedMetric, CertifiedMeasure};

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

/// Proof boundary for a concrete one-word sampler. The caller must first
/// prove *all* measurement invariants of the real program and map. This
/// constructor never inspects their implementations or re-proves privacy.
///
/// The only unchecked operation is ordinary storage of the two callbacks.
/// The runtime Measurement and legacy Measurement::new remain unchanged.
#[cfg_attr(verus_only, verus_verify)]
impl<DI, MI, MO, TO> Measurement<DI, MI, MO, TO>
where
    DI: CertifiedDomain,
    DI::Carrier: Copy,
    MI: CertifiedMetric<DI::Carrier>,
    MO: CertifiedMeasure<Result<TO, ()>>,
    (DI, MI): MetricSpace,
{
    #[cfg_attr(verus_only, verus_verify)]
    #[cfg_attr(verus_only, verus_spec(result =>
        with Ghost(model): Ghost<
            crate::uniform_word::OneWordModel<DI::Carrier, TO>>
        requires crate::measurement_proof::valid_one_word_components(
            &input_domain, &input_metric, &output_measure, &program, map, model),
        ensures match result {
            Ok(meas) => {
                &&& meas.input_domain == input_domain
                &&& meas.input_metric == input_metric
                &&& meas.output_measure == output_measure
                &&& crate::measurement_proof::valid_certified_measurement(&meas)
            },
            Err(_) => true,
        },
    ))]
    pub(crate) fn new_certified<Op, F, P>(
        input_domain: DI,
        input_metric: MI,
        output_measure: MO,
        program: OneWordProgram<Op, F>,
        map: P,
    ) -> Fallible<Self>
    where
        Op: Copy + 'static,
        F: Fn(&Op, &DI::Carrier, Fallible<u64>) -> Fallible<TO> + Copy + 'static,
        P: Fn(&Op, &MI::Distance) -> Fallible<MO::Distance> + Copy + 'static,
    {
        #[cfg(verus_only)]
        proof_with! {Ghost(model)}
        Self::assemble_unchecked(input_domain, input_metric, output_measure, program, map)
    }

    /// Audited Rust plumbing only. All mathematical proof obligations are
    /// preconditions. The trusted fact is that storing the supplied concrete
    /// callbacks and invoking them through Arc preserves their behavior.
    #[cfg_attr(verus_only, verus_verify(external_body))]
    #[cfg_attr(verus_only, verus_spec(result =>
        with Ghost(model): Ghost<
            crate::uniform_word::OneWordModel<DI::Carrier, TO>>
        requires crate::measurement_proof::valid_one_word_components(
            &input_domain, &input_metric, &output_measure, &program, map, model),
        ensures match result {
            Ok(meas) => {
                &&& meas.input_domain == input_domain
                &&& meas.input_metric == input_metric
                &&& meas.output_measure == output_measure
                &&& crate::measurement_proof::valid_certified_measurement(&meas)
            },
            Err(_) => true,
        },
    ))]
    fn assemble_unchecked<Op, F, P>(
        input_domain: DI,
        input_metric: MI,
        output_measure: MO,
        program: OneWordProgram<Op, F>,
        map: P,
    ) -> Fallible<Self>
    where
        Op: Copy + 'static,
        F: Fn(&Op, &DI::Carrier, Fallible<u64>) -> Fallible<TO> + Copy + 'static,
        P: Fn(&Op, &MI::Distance) -> Fallible<MO::Distance> + Copy + 'static,
    {
        #[cfg(not(verus_only))]
        {
            let operator = program.operator;
            Self::new(
                input_domain,
                input_metric,
                output_measure,
                Function::new_fallible(move |input| program.run(input)),
                PrivacyMap::new_fallible(move |d| map(&operator, d)),
            )
        }
        #[cfg(verus_only)]
        { unimplemented!() }
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


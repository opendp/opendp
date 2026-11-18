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
#[cfg(verus_only)]
use crate::{discrete_distance::DiscreteDistance, max_divergence::MaxDivergence};
#[cfg(not(verus_only))]
use crate::{metrics::DiscreteDistance, measures::MaxDivergence};

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

/// Proof-gated Boolean/DiscreteDistance/MaxDivergence constructor.
/// The same function/privacy-map objects passed here are installed in the
/// returned Measurement. No proof token and no hidden operator-specific
/// callback adapter can stand in for the required validity of these parts.
///
/// The descriptor semantics remain to be tied to concrete production types;
/// only Boolean carrier/u32 distance/f64 measure are admitted here.
#[cfg_attr(verus_only, verus_verify)]
impl<DI> Measurement<DI, DiscreteDistance, MaxDivergence, bool>
where
    DI: Domain<Carrier = bool>,
    (DI, DiscreteDistance): MetricSpace,
{
    #[cfg_attr(verus_only, verus_verify)]
    #[cfg_attr(verus_only, verus_spec(result =>
        requires crate::measurement_proof::valid_bool_rr_parts(
            &function, &privacy_map),
        ensures match result {
            Ok(meas) => {
                &&& meas.input_domain == input_domain
                &&& meas.input_metric == input_metric
                &&& meas.output_measure == output_measure
                &&& meas.function == function
                &&& meas.privacy_map == privacy_map
                &&& crate::measurement_proof::valid_bool_rr_measurement(&meas)
            },
            Err(_) => true,
        },
    ))]
    pub(crate) fn new_certified(
        input_domain: DI,
        input_metric: DiscreteDistance,
        output_measure: MaxDivergence,
        function: Function<bool, bool>,
        privacy_map: PrivacyMap<DiscreteDistance, MaxDivergence>,
    ) -> Fallible<Self> {
        Self::construct(
            input_domain, input_metric, output_measure, function, privacy_map,
        )
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


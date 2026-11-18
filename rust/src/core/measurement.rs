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
    /// Legacy constructor: callers remain responsible for establishing
    /// the privacy guarantee of arbitrary supplied callbacks.
    #[cfg_attr(verus_only, verus_verify(external))]
    pub fn new(
        input_domain: DI,
        input_metric: MI,
        output_measure: MO,
        function: Function<DI::Carrier, TO>,
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

    /// The certification boundary is deliberately *before* callback erasure.
    /// Verus must require the full measurement-validity contract of the two
    /// concrete closures at every call site. This method only packages those
    /// callbacks into the ordinary OpenDP Measurement.
    ///
    /// IMPORTANT: the native method has not yet been imported and verified
    /// with that precondition. Until then its name conveys no certification.
    #[cfg_attr(verus_only, verus_verify)]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures match result {
            Ok(meas) => {
                &&& meas.input_domain == input_domain
                &&& meas.input_metric == input_metric
                &&& meas.output_measure == output_measure
            },
            Err(_) => true,
        },
    ))]
    pub(crate) fn new_certified<F, P>(
        input_domain: DI,
        input_metric: MI,
        output_measure: MO,
        function: F,
        privacy_map: P,
    ) -> Fallible<Self>
    where
        F: Fn(&DI::Carrier) -> Fallible<TO> + 'static,
        P: Fn(&MI::Distance) -> MO::Distance + 'static,
    {
        (input_domain.clone(), input_metric.clone()).check_space()?;
        let function = Function::new_fallible(function);
        let privacy_map = PrivacyMap::new(privacy_map);
        Ok(Self {
            input_domain,
            input_metric,
            output_measure,
            function,
            privacy_map,
        })
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


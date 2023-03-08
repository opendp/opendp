use crate::{
    core::{Domain, Measure, Measurement, Metric, PrivacyMap},
    measures::{FixedSmoothedMaxDivergence, SmoothedMaxDivergence},
    error::Fallible,
};

#[cfg(feature = "ffi")]
mod ffi;

/// Fix the delta parameter in the privacy map of a `measurement` with a `SmoothedMaxDivergence` output measure.
/// 
/// # Arguments
/// * `measurement` - a measurement with a privacy curve to be fixed
/// * `delta` - parameter to fix the privacy curve with
/// 
/// # Generics
/// * `DI` - Input Domain
/// * `TO` - Output Type
/// * `MI` - Input Metric. 
/// * `MO` - Output Measure of the input argument. Must be SmoothedMaxDivergence<Q>
pub fn make_fix_delta<DI, TO, MI, MO>(
    measurement: &Measurement<DI, TO, MI, MO>,
    delta: MO::Atom,
) -> Fallible<Measurement<DI, TO, MI, MO::FixedMeasure>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + FixDeltaMeasure,
{
    let Measurement {
        input_domain,
        function,
        input_metric,
        output_measure,
        privacy_map,
    } = measurement.clone();

    Ok(Measurement::new(
        input_domain,
        function,
        input_metric,
        output_measure.new_fixed_measure()?,
        PrivacyMap::new_fallible(move |d_in| {
            // find the smallest epsilon at the given delta
            let curve = privacy_map.eval(d_in)?;
            output_measure.fix_delta(&curve, &delta)
        }),
    ))
}

pub trait FixDeltaMeasure: Measure {
    type Atom;
    type FixedMeasure: Measure;

    // This fn is used for FFI support
    fn new_fixed_measure(&self) -> Fallible<Self::FixedMeasure>;
    
    fn fix_delta(
        &self,
        curve: &Self::Distance,
        delta: &Self::Atom,
    ) -> Fallible<<Self::FixedMeasure as Measure>::Distance>;
}

impl<Q: Clone> FixDeltaMeasure for SmoothedMaxDivergence<Q> {
    type Atom = Q;
    type FixedMeasure = FixedSmoothedMaxDivergence<Q>;

    fn new_fixed_measure(&self) -> Fallible<Self::FixedMeasure> {
        Ok(FixedSmoothedMaxDivergence::default())
    }
    fn fix_delta(&self, curve: &Self::Distance, delta: &Q) -> Fallible<(Q, Q)> {
        curve.epsilon(delta).map(|v| (v, delta.clone()))
    }
}

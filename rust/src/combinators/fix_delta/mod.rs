use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MaxDivergence, PrivacyProfile, SmoothedMaxDivergence},
    traits::InfSub,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

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
/// * `MO` - Output Measure of the input argument. Must be `SmoothedMaxDivergence`
pub fn make_fix_delta<DI, TO, MI, MO>(
    m: &Measurement<DI, TO, MI, MO>,
    delta: f64,
) -> Fallible<Measurement<DI, TO, MI, MO::FixedMeasure>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + FixDeltaMeasure,
    (DI, MI): MetricSpace,
{
    let privacy_map = m.privacy_map.clone();
    let output_measure: MO = m.output_measure.clone();

    m.with_map(
        m.input_metric.clone(),
        m.output_measure.new_fixed_measure()?,
        PrivacyMap::new_fallible(move |d_in| {
            // find the smallest epsilon at the given delta
            let profile = privacy_map.eval(d_in)?;
            output_measure.fix_delta(&profile, delta)
        }),
    )
}

pub trait FixDeltaMeasure: Measure {
    type FixedMeasure: Measure;

    // This fn is used for FFI support
    fn new_fixed_measure(&self) -> Fallible<Self::FixedMeasure>;

    fn fix_delta(
        &self,
        curve: &Self::Distance,
        delta: f64,
    ) -> Fallible<<Self::FixedMeasure as Measure>::Distance>;
}

impl FixDeltaMeasure for SmoothedMaxDivergence {
    type FixedMeasure = Approximate<MaxDivergence>;

    fn new_fixed_measure(&self) -> Fallible<Self::FixedMeasure> {
        Ok(Approximate::default())
    }
    fn fix_delta(&self, profile: &Self::Distance, delta: f64) -> Fallible<(f64, f64)> {
        profile.epsilon(delta).map(|v| (v, delta.clone()))
    }
}

impl FixDeltaMeasure for Approximate<SmoothedMaxDivergence> {
    type FixedMeasure = Approximate<MaxDivergence>;

    fn new_fixed_measure(&self) -> Fallible<Self::FixedMeasure> {
        Ok(Approximate::default())
    }
    fn fix_delta(
        &self,
        (curve, fixed_delta): &(PrivacyProfile, f64),
        delta: f64,
    ) -> Fallible<(f64, f64)> {
        let remaining_delta = delta.neg_inf_sub(&fixed_delta)?;
        curve.epsilon(remaining_delta).map(|v| (v, delta.clone()))
    }
}

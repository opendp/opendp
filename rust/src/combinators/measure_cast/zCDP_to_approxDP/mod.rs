use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, PrivacyProfile, SmoothedMaxDivergence, ZeroConcentratedDivergence},
};

use self::cdp_delta::cdp_delta;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

mod cdp_delta;

/// Constructs a new output measurement where the output measure
/// is casted from `ZeroConcentratedDivergence` to `SmoothedMaxDivergence`.
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `TO` - Output Type
/// * `MI` - Input Metric
/// * `MO` - Privacy Measure
pub fn make_zCDP_to_approxDP<DI, MI, MO, TO>(
    meas: Measurement<DI, MI, MO, TO>,
) -> Fallible<Measurement<DI, MI, MO::ApproxMeasure, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + ConcentratedMeasure,
    (DI, MI): MetricSpace,
{
    let privacy_map = meas.privacy_map.clone();
    Measurement::new(
        meas.input_domain.clone(),
        meas.input_metric.clone(),
        MO::ApproxMeasure::default(),
        meas.function.clone(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let d_mid = privacy_map.eval(d_in)?;

            MO::convert(d_mid)
        }),
    )
}

pub trait ConcentratedMeasure: Measure {
    type ApproxMeasure: Measure;

    fn convert(d_mid: Self::Distance) -> Fallible<<Self::ApproxMeasure as Measure>::Distance>;
}

impl ConcentratedMeasure for ZeroConcentratedDivergence {
    type ApproxMeasure = SmoothedMaxDivergence;

    fn convert(rho: Self::Distance) -> Fallible<<Self::ApproxMeasure as Measure>::Distance> {
        Ok(PrivacyProfile::new(move |epsilon: f64| {
            cdp_delta(rho, epsilon)
        }))
    }
}

impl ConcentratedMeasure for Approximate<ZeroConcentratedDivergence> {
    type ApproxMeasure = Approximate<SmoothedMaxDivergence>;

    fn convert(
        (rho, delta): Self::Distance,
    ) -> Fallible<<Self::ApproxMeasure as Measure>::Distance> {
        Ok((
            PrivacyProfile::new(move |epsilon: f64| cdp_delta(rho, epsilon)),
            delta,
        ))
    }
}

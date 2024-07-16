use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{SMDCurve, SmoothedMaxDivergence, ZeroConcentratedDivergence},
};

use self::cdp_delta::cdp_delta;

#[cfg(feature = "ffi")]
mod ffi;

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
pub fn make_zCDP_to_approxDP<DI, TO, MI>(
    meas: Measurement<DI, TO, MI, ZeroConcentratedDivergence>,
) -> Fallible<Measurement<DI, TO, MI, SmoothedMaxDivergence>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = meas.privacy_map.clone();
    Measurement::new(
        meas.input_domain.clone(),
        meas.function.clone(),
        meas.input_metric.clone(),
        SmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let rho = privacy_map.eval(d_in)?;

            Ok(SMDCurve::new(move |epsilon: f64| cdp_delta(rho, epsilon)))
        }),
    )
}

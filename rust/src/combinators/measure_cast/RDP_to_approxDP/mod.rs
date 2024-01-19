use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{RenyiDivergence, SMDCurve, SmoothedMaxDivergence},
    traits::Float,
};

use self::rdp_epsilon::rdp_epsilon;

#[cfg(feature = "ffi")]
mod ffi;

mod rdp_epsilon;

/// Constructs a new output measurement where the output measure
/// is casted from `RenyiDivergence<QO>` to `SmoothedMaxDivergence<QO>`.
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `TO` - Output Type
/// * `MI` - Input Metric
/// * `QO` - Output distance type. One of `f32` or `f64`.
pub fn make_RDP_to_approxDP<DI, TO, MI, QO>(
    meas: Measurement<DI, TO, MI, RenyiDivergence<QO>>,
) -> Fallible<Measurement<DI, TO, MI, SmoothedMaxDivergence<QO>>>
where
    DI: Domain,
    MI: 'static + Metric,
    QO: Float,
    (DI, MI): MetricSpace,
{
    let privacy_map = meas.privacy_map.clone();
    Measurement::new(
        meas.input_domain.clone(),
        meas.function.clone(),
        meas.input_metric.clone(),
        SmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let rdp_curve = privacy_map.eval(d_in)?;

            Ok(SMDCurve::new(move |&delta: &QO| {
                rdp_epsilon(rdp_curve.clone(), delta)
            }))
        }),
    )
}

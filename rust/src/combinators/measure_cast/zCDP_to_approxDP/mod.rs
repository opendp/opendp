use crate::{
    core::{Domain, Measurement, Metric, PrivacyMap},
    error::Fallible,
    measures::{SMDCurve, SmoothedMaxDivergence, ZeroConcentratedDivergence},
    traits::Float,
};

use self::cks20::cdp_epsilon;

#[cfg(feature = "ffi")]
mod ffi;

mod cks20;

/// Constructs a new output measurement where the output measure 
/// is casted from `ZeroConcentratedDivergence<QO>` to `SmoothedMaxDivergence<QO>`.
/// 
/// # Arguments
/// * `meas` - a measurement with a privacy curve to be casted
/// 
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
/// * `QO` - Output distance type. One of `f32` or `f64`.
pub fn make_zCDP_to_approxDP<DI, DO, MI, QO>(
    meas: Measurement<DI, DO, MI, ZeroConcentratedDivergence<QO>>,
) -> Fallible<Measurement<DI, DO, MI, SmoothedMaxDivergence<QO>>>
where
    DI: Domain,
    DO: Domain,
    MI: 'static + Metric,
    QO: Float,
{
    let Measurement {
        input_domain,
        output_domain,
        function,
        input_metric,
        privacy_map,
        ..
    } = meas;

    Ok(Measurement::new(
        input_domain,
        output_domain,
        function,
        input_metric,
        SmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let rho = privacy_map.eval(d_in)?;
            if rho.is_sign_negative() {
                return fallible!(FailedRelation, "rho must be non-negative");
            }
            Ok(SMDCurve::new(move |&delta: &QO| cdp_epsilon(rho, delta)))
        }),
    ))
}

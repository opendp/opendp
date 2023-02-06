use crate::{
    core::{Domain, Measurement, Metric, PrivacyMap},
    error::Fallible,
    measures::{SMDCurve, SmoothedMaxDivergence, ZeroConcentratedDivergence},
    traits::Float, interactive::{IntoDyn, Queryable, FromDyn},
};

use self::cdp_epsilon::cdp_epsilon;

#[cfg(feature = "ffi")]
mod ffi;

mod cdp_epsilon;

/// Constructs a new output measurement where the output measure 
/// is casted from `ZeroConcentratedDivergence<QO>` to `SmoothedMaxDivergence<QO>`.
/// 
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be casted
/// 
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
/// * `QO` - Output distance type. One of `f32` or `f64`.
pub fn make_zCDP_to_approxDP<DI, DOQ, DOA, MI, QO>(
    meas: Measurement<DI, DOQ, DOA, MI, ZeroConcentratedDivergence<QO>>,
) -> Fallible<Measurement<DI, DOQ, DOA, MI, SmoothedMaxDivergence<QO>>>
where
    DI: Domain,
    DOQ: Domain,
    DOA: Domain,
    DOA::Carrier: Sized,
    MI: 'static + Metric,
    QO: Float,
    Queryable<DOQ::Carrier, DOA>: IntoDyn + FromDyn
{
    let Measurement {
        input_domain,
        query_domain,
        answer_domain,
        function,
        input_metric,
        privacy_map,
        ..
    } = meas;

    Ok(Measurement {
        input_domain,
        query_domain,
        answer_domain,
        function,
        input_metric,
        output_measure: SmoothedMaxDivergence::default(),
        privacy_map: PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let rho = privacy_map.eval(d_in)?;
            
            Ok(SMDCurve::new(move |&delta: &QO| cdp_epsilon(rho, delta)))
        }),
    })
}

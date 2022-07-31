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

pub fn make_cast_zcdp_approxdp<DI, DO, MI, QO>(
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

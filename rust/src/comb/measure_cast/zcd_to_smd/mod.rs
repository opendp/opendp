use crate::{
    core::{Domain, Measurement, Metric, PrivacyMap, Measure},
    error::Fallible,
    measures::{SMDCurve, SmoothedMaxDivergence, ZeroConcentratedDivergence},
    traits::Float,
};

use self::cks20::cdp_epsilon;

#[cfg(feature = "ffi")]
mod ffi;

mod cks20;

pub trait CastableMeasure<MI: Metric, MO2: Measure>: Measure {
    fn cast_map(privacy_map: PrivacyMap<MI, Self>) -> PrivacyMap<MI, MO2>;
}

impl<MI, QO> CastableMeasure<MI, SmoothedMaxDivergence<QO>> for ZeroConcentratedDivergence<QO>
    where MI: 'static + Metric, QO: Float {
    fn cast_map(privacy_map: PrivacyMap<MI, ZeroConcentratedDivergence<QO>>) -> PrivacyMap<MI, SmoothedMaxDivergence<QO>> {
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let rho = privacy_map.eval(d_in)?;
            if rho.is_sign_negative() {
                return fallible!(FailedRelation, "rho must be non-negative");
            }
            Ok(SMDCurve::new(move |&delta: &QO| cdp_epsilon(rho, delta)))
        })
    }
}

pub fn make_cast_measure<DI, DO, MI, MO1, MO2>(
    measurement: Measurement<DI, DO, MI, MO1>,
) -> Fallible<Measurement<DI, DO, MI, MO2>>
where
    DI: Domain,
    DO: Domain,
    MI: Metric,
    MO1: CastableMeasure<MI, MO2>,
    MO2: Measure
{
    let Measurement {
        input_domain,
        output_domain,
        function,
        input_metric,
        privacy_map,
        ..
    } = measurement;

    Ok(Measurement::new(
        input_domain,
        output_domain,
        function,
        input_metric,
        MO2::default(),
        MO1::cast_map(privacy_map)
    ))
}

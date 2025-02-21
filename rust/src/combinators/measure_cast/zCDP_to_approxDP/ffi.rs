use opendp_derive::bootstrap;

use crate::{
    combinators::ConcentratedMeasure,
    core::{FfiResult, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::{Approximate, ZeroConcentratedDivergence},
};

#[bootstrap(features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is casted from `ZeroConcentratedDivergence` to `SmoothedMaxDivergence`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
fn make_zCDP_to_approxDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    fn monomorphize<MO: 'static + ConcentratedMeasure>(
        meas: &AnyMeasurement,
    ) -> Fallible<AnyMeasurement> {
        let privacy_map = meas.privacy_map.clone();
        let meas = Measurement::new(
            meas.input_domain.clone(),
            meas.function.clone(),
            meas.input_metric.clone(),
            meas.output_measure.downcast_ref::<MO>()?.clone(),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in)?.downcast::<MO::Distance>()
            }),
        )?;
        let meas = super::make_zCDP_to_approxDP(meas)?;
        let privacy_map = meas.privacy_map.clone();
        Measurement::new(
            meas.input_domain.clone(),
            meas.function.clone(),
            meas.input_metric.clone(),
            AnyMeasure::new(meas.output_measure.clone()),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                Ok(AnyObject::new(privacy_map.eval(d_in)?))
            }),
        )
    }

    let MO = measurement.output_measure.type_.clone();
    dispatch!(
        monomorphize,
        [(MO, [ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>])],
        (measurement)
    )
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_zCDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    FfiResult::from(make_zCDP_to_approxDP(try_as_ref!(measurement)))
}

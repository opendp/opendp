use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::ZeroConcentratedDivergence,
    traits::Float,
};

#[bootstrap(features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is casted from `ZeroConcentratedDivergence<QO>` to `SmoothedMaxDivergence<QO>`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy curve to be casted
fn make_zCDP_to_approxDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    
    fn monomorphize<QO: Float>(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
        let AnyMeasurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure,
            privacy_map,
        } = measurement.clone();
    
        let measurement = Measurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure: try_!(output_measure.downcast::<ZeroConcentratedDivergence<QO>>()),
            privacy_map: PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in)?.downcast::<QO>()
            }),
        };
    
        let measurement = super::make_zCDP_to_approxDP(measurement)?;
    
        let Measurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure,
            privacy_map,
        } = measurement;
    
        Ok(AnyMeasurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure: AnyMeasure::new(output_measure),
            privacy_map: PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in).map(AnyObject::new)
            }),
        })
    }

    let Q = measurement.output_measure.distance_type.clone();

    dispatch!(monomorphize, [
        (Q, @floats)
    ], (measurement))
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_zCDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    FfiResult::from(make_zCDP_to_approxDP(try_as_ref!(measurement)))
}

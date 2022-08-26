use crate::{ffi::{any::{AnyMeasurement, Downcast, AnyObject, AnyMeasure}, util}, core::{FfiResult, Measurement, PrivacyMap}, combinators::make_zCDP_to_approxDP, measures::ZeroConcentratedDivergence, traits::Float};

#[no_mangle]
pub extern "C" fn opendp_combinators__make_zCDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement

    fn monomorphize<Q: Float>(measurement: &AnyMeasurement) -> FfiResult<*mut AnyMeasurement> {
        let AnyMeasurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure,
            privacy_map
        } = measurement.clone();

        let measurement = Measurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure: try_!(output_measure.downcast::<ZeroConcentratedDivergence<Q>>()),
            privacy_map: PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in)?.downcast::<Q>())
        };

        let measurement = try_!(make_zCDP_to_approxDP(measurement));

        let Measurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure,
            privacy_map
        } = measurement;

        FfiResult::Ok(util::into_raw(AnyMeasurement {
            input_domain,
            output_domain,
            function,
            input_metric,
            output_measure: AnyMeasure::new(output_measure),
            privacy_map: PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in).map(AnyObject::new))
        }))
    }

    let measurement = try_as_ref!(measurement);
    let Q = measurement.output_measure.distance_type.clone();

    dispatch!(monomorphize, [
        (Q, @floats)
    ], (measurement))
}

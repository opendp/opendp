use crate::{
    comb::make_measure_smd,
    core::{FfiResult, PrivacyMap},
    ffi::any::{AnyMeasure, AnyMeasurement, AnyMetric},
};

use super::CastableMeasure;

impl CastableMeasure<AnyMetric, AnyMeasure> for AnyMeasure {
    fn cast_from(
        privacy_map: PrivacyMap<AnyMetric, AnyMeasure>,
    ) -> PrivacyMap<AnyMetric, AnyMeasure> {
        unimplemented!()
    }
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_measure_smd(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement

    make_measure_smd(try_as_ref!(measurement)).into()

    // fn monomorphize<Q: Float>(measurement: &AnyMeasurement) -> FfiResult<*mut AnyMeasurement> {
    //     let AnyMeasurement {
    //         input_domain,
    //         output_domain,
    //         function,
    //         input_metric,
    //         output_measure,
    //         privacy_map
    //     } = measurement.clone();

    //     let measurement = Measurement {
    //         input_domain,
    //         output_domain,
    //         function,
    //         input_metric,
    //         output_measure: try_!(output_measure.downcast::<ZeroConcentratedDivergence<Q>>()),
    //         privacy_map: PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in)?.downcast::<Q>())
    //     };

    //     let measurement = try_!(make_measure_smd(measurement));

    //     let Measurement {
    //         input_domain,
    //         output_domain,
    //         function,
    //         input_metric,
    //         output_measure,
    //         privacy_map
    //     } = measurement;

    //     FfiResult::Ok(util::into_raw(AnyMeasurement {
    //         input_domain,
    //         output_domain,
    //         function,
    //         input_metric,
    //         output_measure: AnyMeasure::new(output_measure),
    //         privacy_map: PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in).map(AnyObject::new))
    //     }))
    // }

    // let measurement = try_as_ref!(measurement);
    // let Q = measurement.output_measure.distance_type.clone();

    // dispatch!(monomorphize, [
    //     (Q, @floats)
    // ], (measurement))
}

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::MaxDivergence,
    traits::Float,
};

#[bootstrap(features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is casted from `MaxDivergence<QO>` to `ZeroConcentratedDivergence<QO>`.
///
/// # Citations
/// - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
fn make_pureDP_to_zCDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    fn monomorphize<QO: Float>(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
        let AnyMeasurement {
            input_domain,
            function,
            input_metric,
            output_measure,
            privacy_map,
        } = measurement.clone();

        let measurement = Measurement {
            input_domain,
            function,
            input_metric,
            output_measure: try_!(output_measure.downcast::<MaxDivergence<QO>>()),
            privacy_map: PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in)?.downcast::<QO>()
            }),
        };

        let measurement = super::make_pureDP_to_zCDP(measurement)?;

        let Measurement {
            input_domain,
            function,
            input_metric,
            output_measure,
            privacy_map,
        } = measurement;

        Ok(AnyMeasurement {
            input_domain,
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
pub extern "C" fn opendp_combinators__make_pureDP_to_zCDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    FfiResult::from(make_pureDP_to_zCDP(try_as_ref!(measurement)))
}

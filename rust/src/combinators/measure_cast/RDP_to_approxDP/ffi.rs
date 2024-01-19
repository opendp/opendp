use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap},
    error::Fallible,
    ffi::{
        any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
        util::{Type, TypeContents},
    },
    measures::RenyiDivergence,
    traits::Float,
};

#[bootstrap(features("contrib"), dependencies("$get_dependencies(measurement)"))]
/// Constructs a new output measurement where the output measure
/// is casted from `ZeroConcentratedDivergence<QO>` to `SmoothedMaxDivergence<QO>`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
fn make_RDP_to_approxDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    fn monomorphize<QO: Float>(m: &AnyMeasurement) -> Fallible<AnyMeasurement> {
        let privacy_map = m.privacy_map.clone();
        let measurement = Measurement::new(
            m.input_domain.clone(),
            m.function.clone(),
            m.input_metric.clone(),
            try_!(m.output_measure.clone().downcast::<RenyiDivergence<QO>>()),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in)?.downcast::<Function<QO, QO>>()
            }),
        )?;

        let m = super::make_RDP_to_approxDP(measurement)?;

        let privacy_map = m.privacy_map.clone();
        m.with_map(
            m.input_metric.clone(),
            AnyMeasure::new(m.output_measure.clone()),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in).map(AnyObject::new)
            }),
        )
    }

    let Q = measurement.output_measure.distance_type.clone();
    let Q_atom = match Q.contents {
        TypeContents::GENERIC { name, args } if name == "Function" => {
            if args.len() != 2 || args[0] != args[1] {
                return err!(FFI, "expected a function with two homogeneous generics").into();
            }
            try_!(Type::of_id(&args[0]))
        }
        _ => return err!(FFI, "expected Function").into(),
    };

    dispatch!(monomorphize, [
        (Q_atom, @floats)
    ], (measurement))
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_RDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    FfiResult::from(make_RDP_to_approxDP(try_as_ref!(measurement)))
}

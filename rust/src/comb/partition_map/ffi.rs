use crate::{
    core::{FfiResult, Measurement, Transformation},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast, AnyMeasurement, IntoAnyFunctionExt, AnyDomain},
        util::{AnyTransformationPtr, AnyMeasurementPtr},
    },
};

use super::{make_partition_map_trans, make_partition_map_meas};

#[no_mangle]
pub extern "C" fn opendp_comb__make_partition_map_trans(
    transformations: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let trans_ptrs =
        try_!(try_as_ref!(transformations).downcast_ref::<Vec<AnyTransformationPtr>>());

    let transformations: Vec<&AnyTransformation> =
        try_!(trans_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    let trans = try_!(make_partition_map_trans(transformations));

    // don't wrap the input metric, output metric and stability map!
    Ok(Transformation::new(
        AnyDomain::new(trans.input_domain),
        AnyDomain::new(trans.output_domain),
        trans.function.into_any(),
        trans.input_metric,
        trans.output_metric,
        trans.stability_map
    )).into()
}


#[no_mangle]
pub extern "C" fn opendp_comb__make_partition_map_meas(
    measurements: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let meas_ptrs =
        try_!(try_as_ref!(measurements).downcast_ref::<Vec<AnyMeasurementPtr>>());

    let measurements: Vec<&AnyMeasurement> =
        try_!(meas_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    let meas = try_!(make_partition_map_meas(measurements));

    // don't wrap the input metric, output measure and privacy map!
    Ok(Measurement::new(
        AnyDomain::new(meas.input_domain),
        AnyDomain::new(meas.output_domain),
        meas.function.into_any(),
        meas.input_metric,
        meas.output_measure,
        meas.privacy_map
    )).into()
}
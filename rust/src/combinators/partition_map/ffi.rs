use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement, Transformation, PrivacyMap},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast, AnyMeasurement, IntoAnyFunctionExt, AnyDomain, AnyMetric, AnyMeasure, IntoAnyStabilityMapExt},
        util::{AnyTransformationPtr, AnyMeasurementPtr},
    }, error::Fallible, domains::ProductDomain,
    metrics::ProductMetric
};

#[bootstrap(features("contrib"))]
/// Construct the parallel execution of [`transformation0`, `transformation1`, ...]. Returns a Transformation.
/// 
/// # Arguments
/// * `transformations` - A list of transformations to apply, one to each element.
fn make_partition_map_trans(
    transformations: Vec<&AnyTransformation>,
) -> Fallible<Transformation<ProductDomain<AnyDomain>, ProductDomain<AnyDomain>, ProductMetric<AnyMetric>, ProductMetric<AnyMetric>>> {
    super::make_partition_map_trans(transformations)
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_partition_map_trans(
    transformations: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let trans_ptrs =
        try_!(try_as_ref!(transformations).downcast_ref::<Vec<AnyTransformationPtr>>());

    let transformations: Vec<&AnyTransformation> =
        try_!(trans_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    let trans = try_!(make_partition_map_trans(transformations));

    Ok(Transformation::new(
        AnyDomain::new(trans.input_domain),
        AnyDomain::new(trans.output_domain),
        trans.function.into_any(),
        AnyMetric::new(trans.input_metric),
        AnyMetric::new(trans.output_metric),
        trans.stability_map.into_any()
    )).into()
}

#[bootstrap(features("contrib"))]
/// Construct the parallel composition of [`measurement0`, `measurement1`, ...]. Returns a Measurement.
/// 
/// # Arguments
/// * `measurements` - A list of measuerements to apply, one to each element.
fn make_partition_map_meas(
    measurements: Vec<&AnyMeasurement>,
) -> Fallible<Measurement<ProductDomain<AnyDomain>, ProductDomain<AnyDomain>, ProductMetric<AnyMetric>, AnyMeasure>> {
    super::make_partition_map_meas(measurements)
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
    let privacy_map = meas.privacy_map;

    // don't wrap the input metric, output measure and privacy map!
    Ok(Measurement::new(
        AnyDomain::new(meas.input_domain),
        AnyDomain::new(meas.output_domain),
        meas.function.into_any(),
        AnyMetric::new(meas.input_metric),
        meas.output_measure,
        PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in))
    )).into()
}
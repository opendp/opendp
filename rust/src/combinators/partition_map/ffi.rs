use opendp_derive::bootstrap;
use num::Zero;

use crate::{
    core::{FfiResult, Measurement, Transformation, PrivacyMap},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast, AnyMeasurement, IntoAnyFunctionExt, AnyDomain, AnyMetric, IntoAnyStabilityMapExt, AnyMeasure},
        util::{AnyTransformationPtr, AnyMeasurementPtr},
    },
    error::Fallible, 
    traits::{TotalOrd, ExactIntCast, InfMul}, 
    measures::{MaxDivergence, FixedSmoothedMaxDivergence, ZeroConcentratedDivergence, SmoothedMaxDivergence}, domains::ProductDomain, metrics::{ProductMetric, IntDistance},
};

use super::ParallelCompositionMeasure;

#[bootstrap(
    features("contrib"),
    arguments(transformations(rust_type = "Vec<AnyTransformationPtr>"))
)]
/// Construct the parallel execution of [`transformation0`, `transformation1`, ...]. Returns a Transformation.
/// 
/// # Arguments
/// * `transformations` - A list of transformations to apply, one to each element.
fn make_parallel_transformation(
    transformations: Vec<&AnyTransformation>,
) -> Fallible<Transformation<ProductDomain<AnyDomain>, ProductDomain<AnyDomain>, ProductMetric<AnyMetric>, ProductMetric<AnyMetric>>> {
    super::make_parallel_transformation(transformations)
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_parallel_transformation(
    transformations: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let trans_ptrs =
        try_!(try_as_ref!(transformations).downcast_ref::<Vec<AnyTransformationPtr>>());

    let transformations: Vec<&AnyTransformation> =
        try_!(trans_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    let trans = try_!(make_parallel_transformation(transformations));

    Ok(Transformation::new(
        AnyDomain::new(trans.input_domain),
        AnyDomain::new(trans.output_domain),
        trans.function.into_any(),
        AnyMetric::new(trans.input_metric),
        AnyMetric::new(trans.output_metric),
        trans.stability_map.into_any()
    )).into()
}

#[bootstrap(
    features("contrib"),
    arguments(measurements(rust_type = "Vec<AnyMeasurementPtr>"))
)]
/// Construct the parallel composition of [`measurement0`, `measurement1`, ...]. Returns a Measurement.
/// 
/// # Arguments
/// * `measurements` - A list of measurements to apply, one to each element.
fn make_parallel_composition(
    measurements: Vec<&AnyMeasurement>,
) -> Fallible<Measurement<ProductDomain<AnyDomain>, ProductDomain<AnyDomain>, ProductMetric<AnyMetric>, AnyMeasure>> {
    super::make_parallel_composition(measurements)
}

impl ParallelCompositionMeasure for AnyMeasure {
    fn compose(&self, d_i: Vec<AnyObject>, partition_limit: IntDistance) -> crate::error::Fallible<Self::Distance> {
        fn monomorphize1<Q: 'static + Zero + Clone + TotalOrd + ExactIntCast<IntDistance> + InfMul>(
            self_: &AnyMeasure, d_i: Vec<AnyObject>, partition_limit: IntDistance
        ) -> Fallible<AnyObject> {

            fn monomorphize2<M: 'static + ParallelCompositionMeasure>(
                self_: &AnyMeasure, d_i: Vec<AnyObject>, partition_limit: IntDistance
            ) -> Fallible<AnyObject>
                where M::Distance: Clone {
                let d_i = d_i.into_iter()
                    .map(|d_i| d_i.downcast::<M::Distance>())
                    .collect::<Fallible<Vec<M::Distance>>>()?;
                self_.downcast_ref::<M>()?.compose(d_i, partition_limit).map(AnyObject::new)
            }
            dispatch!(monomorphize2, [
                (self_.type_, [MaxDivergence<Q>, SmoothedMaxDivergence<Q>, FixedSmoothedMaxDivergence<Q>, ZeroConcentratedDivergence<Q>])
            ], (self_, d_i, partition_limit))
        }

        dispatch!(monomorphize1, [(self.distance_type, @floats)], (self, d_i, partition_limit))
    }
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_parallel_composition(
    measurements: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let meas_ptrs =
        try_!(try_as_ref!(measurements).downcast_ref::<Vec<AnyMeasurementPtr>>());

    let measurements: Vec<&AnyMeasurement> =
        try_!(meas_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    let meas = try_!(make_parallel_composition(measurements));
    let privacy_map = meas.privacy_map;

    // don't wrap the input metric, output measure and privacy map!
    Ok(Measurement::new(
        AnyDomain::new(meas.input_domain),
        AnyDomain::new(meas.output_domain),
        meas.function.into_any(),
        AnyMetric::new(meas.input_metric),
        meas.output_measure,
        PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in.downcast_ref()?))
    )).into()
}
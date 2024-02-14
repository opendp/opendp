use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::{ffi::TypedMeasure, FixedSmoothedMaxDivergence, SMDCurve, SmoothedMaxDivergence},
};

use super::FixDeltaMeasure;

#[bootstrap(
    features("contrib"),
    arguments(
        measurement(rust_type = b"null"),
        delta(rust_type = "$get_atom(measurement_output_distance_type(measurement))")
    ),
    dependencies("$get_dependencies(measurement)")
)]
/// Fix the delta parameter in the privacy map of a `measurement` with a SmoothedMaxDivergence output measure.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy curve to be fixed
/// * `delta` - parameter to fix the privacy curve with
fn make_fix_delta(measurement: &AnyMeasurement, delta: &AnyObject) -> Fallible<AnyMeasurement> {
    fn monomorphize<Q: 'static + Clone + Send + Sync>(
        meas: &AnyMeasurement,
        delta: &AnyObject,
    ) -> Fallible<AnyMeasurement> {
        let privacy_map = meas.privacy_map.clone();
        let meas = Measurement::new(
            meas.input_domain.clone(),
            meas.function.clone(),
            meas.input_metric.clone(),
            TypedMeasure::<SMDCurve<Q>>::new(meas.output_measure.clone())?,
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in)?.downcast::<SMDCurve<Q>>()
            }),
        )?;
        let meas = super::make_fix_delta(&meas, delta.downcast_ref::<Q>()?.clone())?;
        let privacy_map = meas.privacy_map.clone();
        Measurement::new(
            meas.input_domain.clone(),
            meas.function.clone(),
            meas.input_metric.clone(),
            meas.output_measure.measure.clone(),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                Ok(AnyObject::new(privacy_map.eval(d_in)?))
            }),
        )
    }

    let Q = delta.type_.clone();
    dispatch!(monomorphize, [(Q, @floats)], (measurement, delta))
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_fix_delta(
    measurement: *const AnyMeasurement,
    delta: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    make_fix_delta(try_as_ref!(measurement), try_as_ref!(delta)).into()
}

impl<Q: 'static + Clone + Send + Sync> FixDeltaMeasure for TypedMeasure<SMDCurve<Q>> {
    type Atom = Q;
    type FixedMeasure = TypedMeasure<(Q, Q)>;

    fn new_fixed_measure(&self) -> Fallible<TypedMeasure<(Q, Q)>> {
        TypedMeasure::new(AnyMeasure::new(FixedSmoothedMaxDivergence::<Q>::default()))
    }
    fn fix_delta(&self, curve: &Self::Distance, delta: &Q) -> Fallible<(Q, Q)> {
        let measure: &SmoothedMaxDivergence<Q> = self.measure.downcast_ref()?;
        measure.fix_delta(curve, delta)
    }
}

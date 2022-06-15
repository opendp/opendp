use crate::{
    core::FfiResult,
    core::{FixedSmoothedMaxDivergence, SMDCurve, SmoothedMaxDivergence},
    error::Fallible,
    ffi::{any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast}, util::Type},
};

use super::{make_fix_delta, FixDeltaMeasure};

impl FixDeltaMeasure for AnyMeasure {
    type Atom = AnyObject;
    type FixedMeasure = AnyMeasure;

    fn new_fixed_measure(&self) -> Fallible<AnyMeasure> {
        fn monomorphize<Q: 'static + Clone>() -> Fallible<AnyMeasure> {
            Ok(AnyMeasure::new(FixedSmoothedMaxDivergence::<Q>::default()))
        }

        let Q = Type::of_id(&self.measure.value.type_id())?.get_atom()?;
        dispatch!(monomorphize, [(Q, @floats)], ())
    }
    fn fix_delta(&self, curve: &Self::Distance, delta: &AnyObject) -> Fallible<AnyObject> {
        fn monomorphize<Q: 'static + Clone>(
            measure: &AnyMeasure,
            curve: &AnyObject,
            delta: &AnyObject,
        ) -> Fallible<AnyObject> {
            let measure: &SmoothedMaxDivergence<Q> = measure.downcast_ref()?;
            let curve: &SMDCurve<Q> = curve.downcast_ref()?;
            let delta: &Q = delta.downcast_ref()?;
            measure.fix_delta(curve, delta).map(AnyObject::new)
        }

        let Q = Type::of_id(&self.measure.value.type_id())?.get_atom()?;
        dispatch!(monomorphize, [(Q, @floats)], (self, curve, delta))
    }
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_fix_delta(
    measurement: *const AnyMeasurement,
    delta: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    // CLONE DELTA (anyobjects can't be cloned)
    let delta = try_as_ref!(delta);
    fn try_clone<T: 'static + Clone>(value: &AnyObject) -> Fallible<AnyObject> {
        value.downcast_ref::<T>().map(|v| AnyObject::new(v.clone()))
    }
    let Q = delta.type_.clone();
    let delta = try_!(dispatch!(try_clone, [
        (Q, @floats)
    ], (delta)));

    // run combinator on measurement
    make_fix_delta(try_as_ref!(measurement), delta).into()
}

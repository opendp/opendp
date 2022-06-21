use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyMeasurement, AnyObject, IntoAnyMeasurementOutExt, Downcast},
        util::AnyMeasurementPtr,
    },
};

use super::make_sequential_composition_static_distances;

#[no_mangle]
pub extern "C" fn opendp_comb__make_sequential_composition_static_distances(
    measurements: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let meas_ptrs = try_!(try_as_ref!(measurements).downcast_ref::<Vec<AnyMeasurementPtr>>());

    let measurements: Vec<&AnyMeasurement> =
        try_!(meas_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    make_sequential_composition_static_distances(measurements)
        .map(IntoAnyMeasurementOutExt::into_any_out)
        .into()
}

#[cfg(test)]
mod tests {
    use crate::comb::tests::make_test_measurement;
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast, IntoAnyMeasurementExt};
    use crate::ffi::util;

    use super::*;

    #[test]
    fn test_make_sequential_composition_static_distances() -> Fallible<()> {
        let measurement0 = util::into_raw(make_test_measurement::<i32>().into_any());
        let measurement1 = util::into_raw(make_test_measurement::<i32>().into_any());
        let measurements = vec![measurement0, measurement1];
        let basic_composition =
            Result::from(opendp_comb__make_sequential_composition_static_distances(
                AnyObject::new_raw(measurements),
            ))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__measurement_invoke(&basic_composition, arg);
        let res: (AnyObject, AnyObject) = Fallible::from(res)?.downcast()?;
        let res: (i32, i32) = (res.0.downcast()?, res.1.downcast()?);
        assert_eq!(res, (999, 999));
        Ok(())
    }
}

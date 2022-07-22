use crate::comb::{make_chain_mt, make_chain_tt, make_chain_tm};
use crate::core::FfiResult;

use crate::ffi::any::{AnyMeasurement, AnyTransformation};

#[no_mangle]
pub extern "C" fn opendp_comb__make_chain_mt(measurement1: *const AnyMeasurement, transformation0: *const AnyTransformation) -> FfiResult<*mut AnyMeasurement> {
    let transformation0 = try_as_ref!(transformation0);
    let measurement1 = try_as_ref!(measurement1);
    make_chain_mt(measurement1, transformation0).into()
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_chain_tt(transformation1: *const AnyTransformation, transformation0: *const AnyTransformation) -> FfiResult<*mut AnyTransformation> {
    let transformation0 = try_as_ref!(transformation0);
    let transformation1 = try_as_ref!(transformation1);
    make_chain_tt(transformation1, transformation0).into()
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_chain_tm(transformation1: *const AnyTransformation, measurement0: *const AnyMeasurement) -> FfiResult<*mut AnyMeasurement> {
    let transformation1 = try_as_ref!(transformation1);
    let measurement0 = try_as_ref!(measurement0);
    make_chain_tm(transformation1, measurement0).into()
}

#[cfg(test)]
mod tests {
    use crate::comb::tests::{make_test_transformation, make_test_measurement};
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast, IntoAnyMeasurementExt, IntoAnyTransformationExt};
    use crate::ffi::util;

    use super::*;

    #[test]
    fn test_make_chain_mt_ffi() -> Fallible<()> {
        let transformation0 = util::into_raw(make_test_transformation::<i32>().into_any());
        let measurement1 = util::into_raw(make_test_measurement::<i32>().into_any());
        let chain = Result::from(opendp_comb__make_chain_mt(measurement1, transformation0))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__measurement_invoke(&chain, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);

        let d_in = AnyObject::new_raw(999u32);
        let d_out = core::opendp_core__measurement_map(&chain, d_in);
        let d_out: f64 = Fallible::from(d_out)?.downcast()?;
        assert_eq!(d_out, 1000.);
        Ok(())
    }

    #[test]
    fn test_make_chain_tt_ffi() -> Fallible<()> {
        let transformation0 = util::into_raw(make_test_transformation::<i32>().into_any());
        let transformation1 = util::into_raw(make_test_transformation::<i32>().into_any());
        let chain = Result::from(opendp_comb__make_chain_tt(transformation1, transformation0))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__transformation_invoke(&chain, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);

        let d_in = AnyObject::new_raw(999u32);
        let d_out = core::opendp_core__transformation_map(&chain, d_in);
        let d_out: u32 = Fallible::from(d_out)?.downcast()?;
        assert_eq!(d_out, 999);
        Ok(())
    }

    #[test]
    fn test_make_chain_tm_ffi() -> Fallible<()> {
        let measurement0 = util::into_raw(make_test_measurement::<i32>().into_any());
        let transformation1 = util::into_raw(make_test_transformation::<i32>().into_any());
        let chain = Result::from(opendp_comb__make_chain_tm(transformation1, measurement0))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__measurement_invoke(&chain, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);

        let d_in = AnyObject::new_raw(999u32);
        let d_out = core::opendp_core__measurement_map(&chain, d_in);
        let d_out: f64 = Fallible::from(d_out)?.downcast()?;
        assert_eq!(d_out, 1000.);
        Ok(())
    }
}

use std::convert::TryFrom;
use std::os::raw::{c_char, c_void, c_uint};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::meas::{make_base_laplace, LaplaceDomain};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace(
    scale: *const c_void,
    granularity: c_uint,
    D: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D>(scale: *const c_void, granularity: usize) -> FfiResult<*mut AnyMeasurement>
    where
        D: 'static + LaplaceDomain,
    {
        let scale = *try_as_ref!(scale as *const D::Atom);
        make_base_laplace::<D>(scale, Some(granularity)).into_any()
    }
    let D = try_!(Type::try_from(D));
    let granularity = granularity as usize;
    dispatch!(monomorphize, [
        (D, [AllDomain<f64>, AllDomain<f32>, VectorDomain<AllDomain<f64>>, VectorDomain<AllDomain<f32>>])
    ], (scale, granularity))
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_base_laplace() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_laplace(
            util::into_raw(0.0) as *const c_void,
            32,
            "AllDomain<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(1.0);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_base_laplace_vec() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_laplace(
            util::into_raw(0.0) as *const c_void,
            32,
            "VectorDomain<AllDomain<f64>>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }
}

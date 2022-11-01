use std::convert::TryFrom;
use std::os::raw::{c_char, c_long, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::measurements::{make_base_laplace, LaplaceDomain};
use crate::traits::samplers::SampleDiscreteLaplaceZ2k;
use crate::traits::{ExactIntCast, Float, FloatBits};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_laplace(
    scale: *const c_void,
    k: c_long,
    D: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D>(scale: *const c_void, k: i32) -> FfiResult<*mut AnyMeasurement>
    where
        D: 'static + LaplaceDomain,
        (D, D::InputMetric): MetricSpace,
        D::Atom: Float + SampleDiscreteLaplaceZ2k,
        i32: ExactIntCast<<D::Atom as FloatBits>::Bits>,
    {
        let scale = *try_as_ref!(scale as *const D::Atom);
        make_base_laplace::<D>(scale, Some(k)).into_any()
    }
    let k = k as i32;
    let D = try_!(Type::try_from(D));
    dispatch!(monomorphize, [
        (D, [AllDomain<f64>, AllDomain<f32>, VectorDomain<AllDomain<f64>>, VectorDomain<AllDomain<f32>>])
    ], (scale, k))
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
        let measurement = Result::from(opendp_measurements__make_base_laplace(
            util::into_raw(0.0) as *const c_void,
            -1078,
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
        let measurement = Result::from(opendp_measurements__make_base_laplace(
            util::into_raw(0.0) as *const c_void,
            -1078,
            "VectorDomain<AllDomain<f64>>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }
}

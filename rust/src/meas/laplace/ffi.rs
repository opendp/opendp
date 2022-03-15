use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use num::Float;

use crate::{err, try_, try_as_ref};
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::dom::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::meas::{LaplaceDomain, make_base_laplace};
use crate::samplers::SampleLaplace;
use crate::traits::{CheckNull, InfCast, InfMul, TotalOrd};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace(
    scale: *const c_void,
    D: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D>(scale: *const c_void) -> FfiResult<*mut AnyMeasurement>
        where D: 'static + LaplaceDomain,
              D::Atom: 'static + Clone + SampleLaplace + Float + InfCast<D::Atom> + CheckNull + TotalOrd + InfMul {
        let scale = *try_as_ref!(scale as *const D::Atom);
        make_base_laplace::<D>(scale).into_any()
    }
    let D = try_!(Type::try_from(D));
    dispatch!(monomorphize, [
        (D, [AllDomain<f64>, AllDomain<f32>, VectorDomain<AllDomain<f64>>, VectorDomain<AllDomain<f32>>])
    ], (scale))
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
        let measurement = Result::from(opendp_meas__make_base_laplace(util::into_raw(0.0) as *const c_void, "AllDomain<f64>".to_char_p()))?;
        let arg = AnyObject::new_raw(1.0);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_base_laplace_vec() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_laplace(util::into_raw(0.0) as *const c_void, "VectorDomain<AllDomain<f64>>".to_char_p()))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }
}

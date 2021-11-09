use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::meas::{GaussianDomain, make_base_gaussian};
use opendp::samplers::SampleGaussian;
use opendp::traits::{InfCast, CheckNull, InfMul, InfAdd, InfLn, InfSqrt};

use crate::any::AnyMeasurement;
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::util::{c_bool, to_bool, Type};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian(
    scale: *const c_void,
    analytic: c_bool,
    D: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D>(scale: *const c_void, analytic: bool) -> FfiResult<*mut AnyMeasurement> where
        D: 'static + GaussianDomain,
        D::Atom: 'static + Clone + SampleGaussian + Float + InfCast<f64> + CheckNull + InfMul + InfAdd + InfLn + InfSqrt,
        f64: InfCast<D::Atom> {
        let scale = *try_as_ref!(scale as *const D::Atom);
        make_base_gaussian::<D>(scale, analytic).into_any()
    }
    let D = try_!(Type::try_from(D));
    let analytic = to_bool(analytic);
    dispatch!(monomorphize, [
        (D, [AllDomain<f64>, AllDomain<f32>, VectorDomain<AllDomain<f64>>, VectorDomain<AllDomain<f32>>])
    ], (scale, analytic))
}


#[cfg(test)]
mod tests {
    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util;
    use crate::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_base_gaussian() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_gaussian(
            util::into_raw(0.0) as *const c_void, 1, "AllDomain<f64>".to_char_p()))?;
        let arg = AnyObject::new_raw(1.0);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_base_gaussian_vec() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_gaussian(
            util::into_raw(0.0) as *const c_void, 0, "VectorDomain<AllDomain<f64>>".to_char_p()))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }
}

use std::convert::TryFrom;
use std::os::raw::{c_char, c_void, c_double};


use crate::{err, try_, try_as_ref};
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, SmoothedMaxDivergence, ZeroConcentratedDivergence};
use crate::core::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::meas::{GaussianDomain, make_base_analytic_gaussian, make_base_gaussian, GaussianMeasure};
use crate::traits::{Float, InfCast};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian(
    scale: *const c_void,
    D: *const c_char,
    MO: *const c_char
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize_1<T>(scale: *const c_void, D: Type, MO: Type) -> FfiResult<*mut AnyMeasurement> where 
        T: Float + InfCast<f64> {
            let scale = *try_as_ref!(scale as *const T);
            fn monomorphize_2<D, MO>(scale: D::Atom) -> FfiResult<*mut AnyMeasurement> where
                D: 'static + GaussianDomain,
                MO: 'static + GaussianMeasure<D::Metric, Atom = D::Atom> {
                make_base_gaussian::<D, MO>(scale).into_any()
            }

            dispatch!(monomorphize_2, [
                (D, [AllDomain<T>, VectorDomain<AllDomain<T>>]),
                (MO, [SmoothedMaxDivergence<T>, ZeroConcentratedDivergence<T>])
            ], (scale))
        }
    let D = try_!(Type::try_from(D));
    let MO = try_!(Type::try_from(MO));
    let T = try_!(D.get_atom());
    dispatch!(monomorphize_1, [
        (T, @floats)
    ], (scale, D, MO))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_analytic_gaussian(
    scale: c_double,
    D: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D>(scale: f64) -> FfiResult<*mut AnyMeasurement> where
        D: 'static + GaussianDomain<Atom=f64> {
        make_base_analytic_gaussian::<D>(scale).into_any()
    }
    let D = try_!(Type::try_from(D));
    dispatch!(monomorphize, [
        (D, [AllDomain<f64>, VectorDomain<AllDomain<f64>>])
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
    fn test_make_base_gaussian() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_analytic_gaussian(
            0.0 as c_double, "AllDomain<f64>".to_char_p()))?;
        let arg = AnyObject::new_raw(1.0);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_base_gaussian_vec() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_gaussian(
            util::into_raw(0.0) as *const c_void, 
            "VectorDomain<AllDomain<f64>>".to_char_p(), 
            "SmoothedMaxDivergence<f64>".to_char_p()))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }
}

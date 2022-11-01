use std::convert::TryFrom;
use std::os::raw::{c_char, c_long, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::measurements::{make_base_gaussian, GaussianDomain, GaussianMeasure};
use crate::measures::ZeroConcentratedDivergence;
use crate::traits::samplers::{CastInternalRational, SampleDiscreteGaussianZ2k};
use crate::traits::{ExactIntCast, Float, FloatBits};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_gaussian(
    scale: *const c_void,
    k: c_long,
    D: *const c_char,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize1<T>(
        scale: *const c_void,
        k: i32,
        D: Type,
        MO: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: Float + CastInternalRational + SampleDiscreteGaussianZ2k,
        i32: ExactIntCast<T::Bits>,
        rug::Rational: TryFrom<T>,
    {
        let scale = *try_as_ref!(scale as *const T);
        fn monomorphize2<D, MO>(scale: D::Atom, k: i32) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + GaussianDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Float + SampleDiscreteGaussianZ2k,
            MO: 'static + GaussianMeasure<D>,
            i32: ExactIntCast<<D::Atom as FloatBits>::Bits>,
        {
            make_base_gaussian::<D, MO>(scale, Some(k)).into_any()
        }

        dispatch!(monomorphize2, [
            (D, [AllDomain<T>, VectorDomain<AllDomain<T>>]),
            (MO, [ZeroConcentratedDivergence<T>])
        ], (scale, k))
    }
    let k = k as i32;
    let D = try_!(Type::try_from(D));
    let MO = try_!(Type::try_from(MO));
    let T = try_!(D.get_atom());
    dispatch!(monomorphize1, [
        (T, @floats)
    ], (scale, k, D, MO))
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
    fn test_make_base_gaussian_vec() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_gaussian(
            util::into_raw(0.0) as *const c_void,
            -1078,
            "VectorDomain<AllDomain<f64>>".to_char_p(),
            "ZeroConcentratedDivergence<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }

    #[test]
    fn test_make_base_gaussian_zcdp() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_gaussian(
            util::into_raw(0.0) as *const c_void,
            -1078,
            "AllDomain<f64>".to_char_p(),
            "ZeroConcentratedDivergence<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(1.0);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0);
        Ok(())
    }
}

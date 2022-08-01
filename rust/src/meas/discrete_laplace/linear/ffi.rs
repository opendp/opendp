use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::{AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util;
use crate::ffi::util::Type;
use crate::meas::{make_base_discrete_laplace_linear, DiscreteLaplaceDomain};
use crate::traits::samplers::SampleDiscreteLaplaceLinear;
use crate::traits::{Float, InfCast, Integer};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_discrete_laplace_linear(
    scale: *const c_void,
    bounds: *const AnyObject,
    D: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D, QO>(
        scale: *const c_void,
        bounds: *const AnyObject,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        D: 'static + DiscreteLaplaceDomain,
        D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<D::Atom>,
    {
        let scale = try_as_ref!(scale as *const QO).clone();
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            Some(try_!(bounds.downcast_ref::<(D::Atom, D::Atom)>()).clone())
        } else {
            None
        };
        make_base_discrete_laplace_linear::<D, QO>(scale, bounds).into_any()
    }
    let D = try_!(Type::try_from(D));
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (D, [
            AllDomain<u8>, AllDomain<u16>, AllDomain<u32>, AllDomain<u64>, AllDomain<u128>,
            AllDomain<i8>, AllDomain<i16>, AllDomain<i32>, AllDomain<i64>, AllDomain<i128>,
            VectorDomain<AllDomain<u8>>, VectorDomain<AllDomain<u16>>, VectorDomain<AllDomain<u32>>,
            VectorDomain<AllDomain<u64>>, VectorDomain<AllDomain<u128>>, VectorDomain<AllDomain<i8>>,
            VectorDomain<AllDomain<i16>>, VectorDomain<AllDomain<i32>>, VectorDomain<AllDomain<i64>>,
            VectorDomain<AllDomain<i128>>
        ]),
        (QO, @floats)
    ], (scale, bounds))
}

#[deprecated(
    since = "0.5.0",
    note = "Use `opendp_meas__make_base_discrete_laplace` instead. For a constant-time algorithm, pass bounds into `opendp_meas__make_base_discrete_laplace_linear`."
)]
#[no_mangle]
pub extern "C" fn opendp_meas__make_base_geometric(
    scale: *const c_void,
    bounds: *const AnyObject,
    D: *const c_char, QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    opendp_meas__make_base_discrete_laplace_linear(scale, bounds, D, QO)
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
    fn test_make_base_discrete_laplace_linear_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_discrete_laplace_linear(
            util::into_raw(0.0) as *const c_void,
            std::ptr::null(),
            "AllDomain<i32>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }

    #[test]
    fn test_constant_time_make_base_discrete_laplace_linear_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_discrete_laplace_linear(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(AnyObject::new((0, 100))),
            "AllDomain<i32>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}

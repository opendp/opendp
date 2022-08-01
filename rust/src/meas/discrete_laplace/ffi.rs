use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::meas::{make_base_discrete_laplace, DiscreteLaplaceDomain};
use crate::traits::samplers::SampleDiscreteLaplaceLinear;
use crate::traits::{Float, InfCast, Integer};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_discrete_laplace(
    scale: *const c_void,
    D: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {

    #[cfg(feature="use-mpfr")]
    fn monomorphize<D, QO>(scale: *const c_void) -> FfiResult<*mut AnyMeasurement>
    where
        D: 'static + DiscreteLaplaceDomain,
        D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<D::Atom> + InfCast<D::Atom>,
        rug::Rational: TryFrom<QO>,
        rug::Integer: From<D::Atom> + az::SaturatingCast<D::Atom>,
    {
        let scale = try_as_ref!(scale as *const QO).clone();
        make_base_discrete_laplace::<D, QO>(scale).into_any()
    }
    #[cfg(not(feature="use-mpfr"))]
    fn monomorphize<D, QO>(scale: *const c_void) -> FfiResult<*mut AnyMeasurement>
    where
        D: 'static + DiscreteLaplaceDomain,
        D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
        QO: Float + InfCast<D::Atom>,
    {
        let scale = try_as_ref!(scale as *const QO).clone();
        make_base_discrete_laplace::<D, QO>(scale).into_any()
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
    fn test_make_base_discrete_laplace_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_discrete_laplace(
            util::into_raw(0.0) as *const c_void,
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

use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use az::SaturatingCast;

use crate::core::FfiResult;
use crate::ffi::any::AnyMeasurement;
use crate::{
    domains::{AllDomain, VectorDomain},
    ffi::util::Type,
    measurements::{make_base_discrete_laplace_cks20, DiscreteLaplaceDomain},
    traits::InfCast,
};
use crate::core::IntoAnyStaticMeasurementFfiResultExt;


#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_laplace_cks20(
    scale: *const c_void,
    D: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QO>(scale: *const c_void, D: Type, QO: Type) -> FfiResult<*mut AnyMeasurement>
    where
        T: crate::traits::Integer,
        QO: crate::traits::Float + InfCast<T>,
        rug::Rational: TryFrom<QO>,
        rug::Integer: From<T> + SaturatingCast<T>,
    {
        fn monomorphize2<D, QO>(scale: QO) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + DiscreteLaplaceDomain,
            D::Atom: crate::traits::Integer,
            QO: crate::traits::Float + InfCast<D::Atom>,
            rug::Rational: TryFrom<QO>,
            rug::Integer: From<D::Atom> + SaturatingCast<D::Atom>,
        {
            make_base_discrete_laplace_cks20::<D, QO>(scale).into_any_static()
        }
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AllDomain<T>, VectorDomain<AllDomain<T>>]),
            (QO, [QO])
        ], (scale))
    }
    let D = try_!(Type::try_from(D));
    let T = try_!(D.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (scale, D, QO))
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
    fn test_make_base_discrete_laplace_cks20_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace_cks20(
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

    #[test]
    fn test_constant_time_make_base_discrete_laplace_cks20_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_laplace_cks20(
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

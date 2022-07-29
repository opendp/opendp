use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use az::SaturatingCast;
use rug::{Rational, Integer};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::meas::{make_base_discrete_gaussian, DiscreteGaussianDomain};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_discrete_gaussian(
    scale: *const c_void,
    D: *const c_char,
    Q: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T>(scale: *const c_void, D: Type, Q: Type) -> FfiResult<*mut AnyMeasurement>
    where
        T: crate::traits::Integer,
        rug::Integer: From<T> + az::SaturatingCast<T>,
    {
        fn monomorphize2<D, Q>(scale: *const c_void) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + DiscreteGaussianDomain<Q>,
            D::Atom: crate::traits::Integer,
            Q: crate::traits::Float,
            Rational: TryFrom<Q>,
            Integer: From<D::Atom> + SaturatingCast<D::Atom>,
        {
            let scale = try_as_ref!(scale as *const Q).clone();
            make_base_discrete_gaussian::<D, Q>(scale).into_any()
        }
        dispatch!(monomorphize2, [
            (D, [VectorDomain<AllDomain<T>>, AllDomain<T>]),
            (Q, @floats)
        ], (scale))
    }
    let D = try_!(Type::try_from(D));
    let Q = try_!(Type::try_from(Q));
    let T = try_!(D.get_atom());
    dispatch!(monomorphize, [
        (T, @integers)
    ], (scale, D, Q))
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
    fn test_make_base_simple_geometric() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_discrete_gaussian(
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
    fn test_make_base_simple_constant_time_geometric() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_discrete_gaussian(
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

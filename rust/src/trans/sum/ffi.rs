use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint};

use num::Zero;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{AlertingAbs, CheckNull, DistanceConstant, ExactIntCast, InfCast, InfDiv, InfSub, SaturatingAdd};
use crate::trans::{make_bounded_sum, make_sized_bounded_sum};

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum(
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        bounds: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant<IntDistance> + Sub<Output=T> + SaturatingAdd + Zero + CheckNull + AlertingAbs,
              IntDistance: InfCast<T> {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_bounded_sum::<T>(bounds).into_any()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T, @numbers)
    ], (bounds))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_sum(
    size: c_uint, bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(size: usize, bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant<IntDistance> + Sub<Output=T> + ExactIntCast<usize> + CheckNull + InfDiv + InfSub,
              for<'a> T: Sum<&'a T>,
              IntDistance: InfCast<T> {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_sum::<T>(size, bounds).into_any()
    }
    let size = size as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], (size, bounds))
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
    fn test_make_bounded_sum() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_bounded_sum(
            util::into_raw(AnyObject::new((0., 10.))),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.0);
        Ok(())
    }

    #[test]
    fn test_make_bounded_sum_n() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_sized_bounded_sum(
            3 as c_uint,
            util::into_raw(AnyObject::new((0., 10.))),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.0);
        Ok(())
    }
}

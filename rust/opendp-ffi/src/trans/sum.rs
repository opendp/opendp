use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use opendp::core::{DatasetMetric};
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::err;
use opendp::traits::{Abs, DistanceConstant};
use opendp::trans::{BoundedSumConstant, make_bounded_sum, make_bounded_sum_n};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum(
    lower: *const c_void, upper: *const c_void,
    MI: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, T>(
        lower: *const c_void, upper: *const c_void
    ) -> FfiResult<*mut AnyTransformation>
        where MI: 'static + DatasetMetric,
              for<'a> T: DistanceConstant + Sub<Output=T> + Abs + Sum<&'a T>,
              MI: BoundedSumConstant<T> {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_bounded_sum::<MI, T>(lower, upper).into_any()
    }
    let MI = try_!(Type::try_from(MI));
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (MI, [HammingDistance, SymmetricDistance]),
        (T, @numbers)
    ], (lower, upper))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum_n(
    lower: *const c_void, upper: *const c_void, n: c_uint,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(lower: *const c_void, upper: *const c_void, n: usize) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant + Sub<Output=T>,
              for<'a> T: Sum<&'a T> {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_bounded_sum_n::<T>(lower, upper, n).into_any()
    }
    let n = n as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], (lower, upper, n))
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
    fn test_make_bounded_sum() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_bounded_sum(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            "SymmetricDistance".to_char_p(),
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
        let transformation = Result::from(opendp_trans__make_bounded_sum_n(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            3 as c_uint,
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.0);
        Ok(())
    }
}

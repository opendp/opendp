use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use num::Float;

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{BoundedMeanConstant, make_bounded_mean};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_mean(
    lower: *const c_void, upper: *const c_void, n: c_uint,
    MI: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, T>(lower: *const c_void, upper: *const c_void, n: usize) -> FfiResult<*mut AnyTransformation>
        where MI: 'static + DatasetMetric,
              T: DistanceConstant + Sub<Output=T> + Float,
              for<'a> T: Sum<&'a T>,
              MI: BoundedMeanConstant<T> {
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        make_bounded_mean::<MI, T>(lower, upper, n).into_any()
    }
    let n = n as usize;
    let MI = try_!(Type::try_from(MI));
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (MI, [HammingDistance, SymmetricDistance]),
        (T, @floats)
    ], (lower, upper, n))
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
    fn test_make_bounded_sum_n() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_bounded_mean(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            3 as c_uint,
            "SymmetricDistance".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 2.0);
        Ok(())
    }
}

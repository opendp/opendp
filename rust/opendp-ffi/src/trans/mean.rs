use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint};

use num::Float;

use opendp::err;
use opendp::traits::{DistanceConstant, InfCast, ExactIntCast, AlertingMul, CheckNull, InfDiv, InfSub};
use opendp::trans::{make_sized_bounded_mean};

use crate::any::{AnyTransformation, AnyObject, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;
use opendp::dist::IntDistance;

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_mean(
    size: c_uint, bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(size: usize, bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant<IntDistance> + Sub<Output=T> + Float + ExactIntCast<usize> + AlertingMul + CheckNull + InfSub + InfDiv,
              for<'a> T: Sum<&'a T>,
              IntDistance: InfCast<T> {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_mean::<T>(size, bounds).into_any()
    }
    let size = size as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @floats)], (size, bounds))
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
        let transformation = Result::from(opendp_trans__make_sized_bounded_mean(
            3 as c_uint,
            util::into_raw(AnyObject::new((0., 10.))),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 2.0);
        Ok(())
    }
}

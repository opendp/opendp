use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint, c_void};

use opendp::err;
use opendp::trans::make_resize_constant;
use opendp::traits::{CheckNull, TotalOrd};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;
use opendp::dom::IntervalDomain;
use std::ops::Bound;

#[no_mangle]
pub extern "C" fn opendp_trans__make_resize_constant_bounded(
    constant: *const c_void, length: c_uint,
    lower: *const c_void, upper: *const c_void,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA>(
        constant: *const c_void, length: usize,
        lower: *const c_void, upper: *const c_void
    ) -> FfiResult<*mut AnyTransformation>
        where TA: 'static + Clone + CheckNull + TotalOrd {
        let constant = try_as_ref!(constant as *const TA).clone();
        let lower = try_as_ref!(lower as *const TA).clone();
        let upper = try_as_ref!(upper as *const TA).clone();
        let atom_domain = try_!(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper)));
        make_resize_constant::<IntervalDomain<TA>>(atom_domain, constant, length).into_any()
    }
    let length = length as usize;
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [(TA, @numbers)], (constant, length, lower, upper))
}


#[cfg(test)]
mod tests {
    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core::opendp_core__transformation_invoke;
    use crate::util;
    use crate::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_resize_constant_bounded(
            util::into_raw(0i32) as *const c_void,
            4 as c_uint,
            util::into_raw(0i32) as *const c_void,
            util::into_raw(10i32) as *const c_void,
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }
}

use std::convert::TryFrom;
use std::ops::Bound;
use std::os::raw::{c_char, c_uint, c_void};

use opendp::dom::{AllDomain, BoundedDomain};
use opendp::err;
use opendp::traits::{CheckNull, TotalOrd};
use opendp::trans::make_resize_constant;

use crate::any::{AnyObject, AnyTransformation};
use crate::any::Downcast;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_trans__make_resize_bounded(
    size: c_uint, lower: *const c_void, upper: *const c_void,
    constant: *const c_void,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA>(
        size: usize, lower: *const c_void, upper: *const c_void,
        constant: *const c_void,
    ) -> FfiResult<*mut AnyTransformation>
        where TA: 'static + Clone + CheckNull + TotalOrd {
        let lower = try_as_ref!(lower as *const TA).clone();
        let upper = try_as_ref!(upper as *const TA).clone();
        let atom_domain = try_!(BoundedDomain::new(Bound::Included(lower), Bound::Included(upper)));
        let constant = try_as_ref!(constant as *const TA).clone();
        make_resize_constant::<BoundedDomain<TA>>(size, atom_domain, constant).into_any()
    }
    let size = size as usize;
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [(TA, @numbers)], (size, lower, upper, constant))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_resize(
    size: c_uint, constant: *const AnyObject,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA>(
        size: usize, constant: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
        where TA: 'static + Clone + CheckNull + TotalOrd {
        let constant = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
        make_resize_constant::<AllDomain<TA>>(size, AllDomain::new(), constant).into_any()
    }
    let size = size as usize;
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [(TA, @numbers)], (size, constant))
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
        let transformation = Result::from(opendp_trans__make_resize(
            4 as c_uint,
            AnyObject::new_raw(0i32),
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }


    #[test]
    fn test_make_resize_bounded() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_resize_bounded(
            4 as c_uint,
            util::into_raw(0i32) as *const c_void,
            util::into_raw(10i32) as *const c_void,
            util::into_raw(0i32) as *const c_void,
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }
}

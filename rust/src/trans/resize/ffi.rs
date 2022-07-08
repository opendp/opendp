use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint, c_void};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::core::{InsertDeleteDistance, SymmetricDistance, IntDistance};
use crate::core::{AllDomain, BoundedDomain};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation};
use crate::ffi::any::Downcast;
use crate::ffi::util::Type;
use crate::traits::{CheckNull, TotalOrd};
use crate::trans::make_resize_constant;
use crate::trans::resize::IsMetricOrdered;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_resize(
    size: c_uint, bounds: *const AnyObject,
    constant: *const c_void,
    TA: *const c_char,
    MI: *const c_char,
    MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA, MI, MO>(
        size: usize, bounds: *const AnyObject,
        constant: *const c_void,
    ) -> FfiResult<*mut AnyTransformation>
        where 
            TA: 'static + Clone + CheckNull + TotalOrd,
            MI: 'static + IsMetricOrdered<Distance=IntDistance>,
            MO: 'static + IsMetricOrdered<Distance=IntDistance>, {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(TA, TA)>()).clone();
        let atom_domain = try_!(BoundedDomain::new_closed(bounds));
        let constant = try_as_ref!(constant as *const TA).clone();
        make_resize_constant::<_, MI, MO>(size, atom_domain, constant).into_any()
    }
    let size = size as usize;
    let TA = try_!(Type::try_from(TA));
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    dispatch!(monomorphize, [
        (TA, @numbers), 
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (MO, [SymmetricDistance, InsertDeleteDistance])
    ], (size, bounds, constant))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_resize(
    size: c_uint, constant: *const AnyObject,
    TA: *const c_char,
    MI: *const c_char,
    MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA, MI, MO>(
        size: usize, constant: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
        where 
            TA: 'static + Clone + CheckNull,
            MI: 'static + IsMetricOrdered<Distance=IntDistance>,
            MO: 'static + IsMetricOrdered<Distance=IntDistance>, {
        let constant = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
        make_resize_constant::<AllDomain<TA>, MI, MO>(size, AllDomain::new(), constant).into_any()
    }
    let size = size as usize;
    let TA = try_!(Type::try_from(TA));
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    dispatch!(monomorphize, [
        (TA, @numbers), 
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (MO, [SymmetricDistance, InsertDeleteDistance])
    ], (size, constant))
}


#[cfg(test)]
mod tests {
    use crate::core::opendp_core__transformation_invoke;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_resize(
            4 as c_uint,
            AnyObject::new_raw(0i32),
            "i32".to_char_p(),
            "SymmetricDistance".to_char_p(),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }


    #[test]
    fn test_make_bounded_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_bounded_resize(
            4 as c_uint,
            util::into_raw(AnyObject::new((0i32, 10))),
            util::into_raw(0i32) as *const c_void,
            "i32".to_char_p(),
            "SymmetricDistance".to_char_p(),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }
}

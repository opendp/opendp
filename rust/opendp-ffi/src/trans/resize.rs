use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint, c_void};

use opendp::err;
use opendp::trans::make_resize_constant;
use opendp::traits::CheckNull;

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_trans__make_resize_constant(
    constant: *const c_void, length: c_uint,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA>(constant: *const c_void, length: usize) -> FfiResult<*mut AnyTransformation>
        where TA: 'static + Clone + CheckNull {
        let constant = try_as_ref!(constant as *const TA).clone();
        make_resize_constant::<TA>(constant, length).into_any()
    }
    let length = length as usize;
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [(TA, @primitives)], (constant, length))
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
        let transformation = Result::from(opendp_trans__make_resize_constant(
            util::into_raw(0i32) as *const c_void,
            4 as c_uint,
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }
}

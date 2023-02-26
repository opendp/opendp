use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{TotalOrd, CheckAtom};
use crate::transformations::make_clamp;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_clamp(
    bounds: *const AnyObject,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize_dataset<TA>(bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
        where TA: 'static + Clone + TotalOrd + CheckAtom {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(TA, TA)>()).clone();
        make_clamp::<TA>(bounds).into_any()
    }
    dispatch!(monomorphize_dataset, [
        (TA, @numbers)
    ], (bounds))
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
    fn test_make_vector_clamp() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_clamp(
            util::into_raw(AnyObject::new((0.0, 10.0))),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![-1.0, 5.0, 11.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0.0, 5.0, 10.0]);
        Ok(())
    }
}

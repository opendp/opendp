use std::convert::TryFrom;
use std::ops::Bound;
use std::os::raw::{c_char, c_void};

use opendp::err;
use opendp::traits::{CheckNull, TotalOrd};
use opendp::trans::{make_clamp, make_unclamp};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;
use std::fmt::Debug;

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp(
    lower: *const c_void, upper: *const c_void,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let T = try_!(Type::try_from(T));

    fn monomorphize_dataset<T>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut AnyTransformation>
        where T: 'static + Clone + TotalOrd + CheckNull + Debug {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_clamp::<T>(lower, upper).into_any()
    }
    dispatch!(monomorphize_dataset, [
        (T, @numbers)
    ], (lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_unclamp(
    lower: *const c_void, upper: *const c_void,
    T: *const c_char
) -> FfiResult<*mut AnyTransformation> {
    let T = try_!(Type::try_from(T));
    fn monomorphize_dataset<T>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut AnyTransformation>
        where T: 'static + Clone + TotalOrd + CheckNull + Debug {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_unclamp::<T>(Bound::Included(lower), Bound::Included(upper)).into_any()
    }
    dispatch!(monomorphize_dataset, [
        (T, @numbers)
    ], (lower, upper))
}


#[cfg(test)]
mod tests {
    use std::os::raw::c_void;

    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::trans::clamp::opendp_trans__make_clamp;
    use crate::util;
    use crate::util::ToCharP;

    #[test]
    fn test_make_vector_clamp() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_clamp(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![-1.0, 5.0, 11.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0.0, 5.0, 10.0]);
        Ok(())
    }

}

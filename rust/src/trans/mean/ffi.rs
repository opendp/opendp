use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use num::Float;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::AbsoluteDistance;
use crate::dom::AllDomain;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::ExactIntCast;
use crate::trans::{
    make_sized_bounded_mean, LipschitzMulDomain, LipschitzMulMetric, MakeSizedBoundedSum,
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_mean(
    size: c_uint,
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(size: usize, bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + MakeSizedBoundedSum + ExactIntCast<usize> + Float,
        AllDomain<T>: LipschitzMulDomain<Atom=T>,
        AbsoluteDistance<T>: LipschitzMulMetric<Distance = T>,
    {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_mean::<T>(size, bounds).into_any()
    }
    let size = size as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @floats)], (size, bounds))
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
    fn test_make_sized_bounded_mean() -> Fallible<()> {
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

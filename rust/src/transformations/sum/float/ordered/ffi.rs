use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};

use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::Float;
use crate::transformations::{
    make_bounded_float_ordered_sum, make_sized_bounded_float_ordered_sum, Pairwise, SaturatingSum,
    Sequential,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_bounded_float_ordered_sum(
    size_limit: c_uint,
    bounds: *const AnyObject,
    S: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        S: Type,
        size_limit: usize,
        bounds: *const AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        T: 'static + Float,
    {
        fn monomorphize2<S>(
            size_limit: usize,
            bounds: (S::Item, S::Item),
        ) -> Fallible<AnyTransformation>
        where
            S: SaturatingSum,
            S::Item: 'static + Float,
        {
            make_bounded_float_ordered_sum::<S>(size_limit, bounds).into_any()
        }
        let bounds = *try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>());
        dispatch!(monomorphize2, [(S, [Sequential<T>, Pairwise<T>])], (size_limit, bounds))
    }
    let size_limit = size_limit as usize;
    let S = try_!(Type::try_from(S));
    let T = try_!(S.get_atom());
    dispatch!(monomorphize, [
        (T, @floats)
    ], (S, size_limit, bounds))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_bounded_float_ordered_sum(
    size: c_uint,
    bounds: *const AnyObject,
    S: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        S: Type,
        size: usize,
        bounds: *const AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        T: 'static + Float,
    {
        fn monomorphize2<S>(size: usize, bounds: (S::Item, S::Item)) -> Fallible<AnyTransformation>
        where
            S: SaturatingSum,
            S::Item: 'static + Float,
        {
            make_sized_bounded_float_ordered_sum::<S>(size, bounds).into_any()
        }
        let bounds = *try_as_ref!(bounds).downcast_ref::<(T, T)>()?;
        dispatch!(monomorphize2, [(S, [Sequential<T>, Pairwise<T>])], (size, bounds))
    }
    let size = size as usize;
    let S = try_!(Type::try_from(S));
    let T = try_!(S.get_atom());
    dispatch!(monomorphize, [(T, @floats)], (S, size, bounds)).into()
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
    fn test_make_bounded_float_ordered_sum_ffi() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_bounded_float_ordered_sum(
            100, // I know the dataset is small; it is no larger than 100
            util::into_raw(AnyObject::new((0., 10.))),
            "Pairwise<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1., 2., 3.]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.);
        Ok(())
    }

    #[test]
    fn test_make_sized_bounded_float_ordered_sum_ffi() -> Fallible<()> {
        let transformation = Result::from(
            opendp_transformations__make_sized_bounded_float_ordered_sum(
                3 as c_uint,
                util::into_raw(AnyObject::new((0., 10.))),
                "Pairwise<f64>".to_char_p(),
            ),
        )?;
        let arg = AnyObject::new_raw(vec![1., 2., 3.]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.);
        Ok(())
    }
}

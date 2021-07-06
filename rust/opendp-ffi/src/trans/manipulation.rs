use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::trans::{make_identity, make_is_equal};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{Type, TypeContents};

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize_scalar<M, T>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              T: 'static + Clone {
        make_identity::<AllDomain<T>, M>(AllDomain::<T>::new(), M::default()).into_any()
    }
    fn monomorphize_vec<M, T>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              T: 'static + Clone {
        make_identity::<VectorDomain<AllDomain<T>>, M>(VectorDomain::new(AllDomain::<T>::new()), M::default()).into_any()
    }
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));
    match &T.contents {
        TypeContents::VEC(element_id) => dispatch!(monomorphize_vec, [
            (M, @dist_dataset),
            (try_!(Type::of_id(element_id)), @primitives)
        ], ()),
        _ => dispatch!(monomorphize_scalar, [
            (M, @dist_dataset),
            (&T, @primitives)
        ], ())
    }
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_is_equal(
    value: *const c_void,
    TI: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TI = try_!(Type::try_from(TI));

    fn monomorphize<TI>(value: *const c_void) -> FfiResult<*mut AnyTransformation> where
        TI: 'static + Clone + PartialEq {
        let value = try_as_ref!(value as *const TI).clone();
        make_is_equal::<TI>(value).into_any()
    }
    dispatch!(monomorphize, [(TI, @primitives)], (value))
}


#[cfg(test)]
mod tests {

    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::{core, util};

    use super::*;
    use crate::util::ToCharP;

    #[test]
    fn test_make_identity() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_identity(
            "SymmetricDistance".to_char_p(),
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(123);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 123);
        Ok(())
    }

    #[test]
    fn test_make_is_equal() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_is_equal(
            util::into_raw(1) as *const c_void,
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<bool> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![true, false, false]);
        Ok(())
    }
}

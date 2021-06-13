use std::convert::TryFrom;
use std::os::raw::c_char;

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::trans::make_identity;

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


#[cfg(test)]
mod tests {

    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;

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
}

use std::convert::TryFrom;
use std::os::raw::{c_char};

use opendp::core::{DatasetMetric};
use opendp::dist::{SubstituteDistance, SymmetricDistance};
use opendp::dom::{AllDomain, VectorDomain, OptionNullDomain, InherentNullDomain, InherentNull};
use opendp::err;
use opendp::trans::{make_identity, make_is_equal, make_is_null};

use crate::any::{AnyTransformation, AnyObject, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{Type, TypeContents};
use opendp::traits::CheckNull;

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize_scalar<M, T>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              T: 'static + Clone + CheckNull {
        make_identity::<AllDomain<T>, M>(AllDomain::<T>::new(), M::default()).into_any()
    }
    fn monomorphize_vec<M, T>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              T: 'static + Clone + CheckNull {
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
    value: *const AnyObject,
    TI: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TI = try_!(Type::try_from(TI));

    fn monomorphize<TI>(value: *const AnyObject) -> FfiResult<*mut AnyTransformation> where
        TI: 'static + Clone + PartialEq + CheckNull {
        let value: TI = try_!(try_as_ref!(value).downcast_ref::<TI>()).clone();
        make_is_equal::<TI>(value).into_any()
    }
    dispatch!(monomorphize, [(TI, @primitives)], (value))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_is_null(
    DIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DIA = try_!(Type::try_from(DIA));
    let T = try_!(DIA.get_atom());

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionNullDomain" => {
            fn monomorphize<T>() -> FfiResult<*mut AnyTransformation>
                where T: 'static + CheckNull {
                make_is_null::<OptionNullDomain<AllDomain<T>>>().into_any()
            }
            dispatch!(monomorphize, [(T, @primitives)], ())
        }
        TypeContents::GENERIC { name, .. } if name == &"InherentNullDomain" => {
            fn monomorphize<T>() -> FfiResult<*mut AnyTransformation>
                where T: 'static + InherentNull {
                make_is_null::<InherentNullDomain<AllDomain<T>>>().into_any()
            }
            dispatch!(monomorphize, [(T, [f64, f32])], ())
        },
        _ => err!(TypeParse, "DA must be an OptionNullDomain<AllDomain<T>> or an InherentNullDomain<AllDomain<T>>").into()
    }
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
            util::into_raw(AnyObject::new(1)) as *const AnyObject,
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<bool> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![true, false, false]);
        Ok(())
    }
}

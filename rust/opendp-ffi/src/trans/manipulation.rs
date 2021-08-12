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
use std::fmt::Debug;

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(
    M: *const c_char, TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize_scalar<M, TA>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              TA: 'static + Clone + CheckNull + Debug {
        make_identity::<AllDomain<TA>, M>(AllDomain::<TA>::new(), M::default()).into_any()
    }
    fn monomorphize_vec<M, TA>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              TA: 'static + Clone + CheckNull + Debug {
        make_identity::<VectorDomain<AllDomain<TA>>, M>(VectorDomain::new(AllDomain::<TA>::new()), M::default()).into_any()
    }
    let M = try_!(Type::try_from(M));
    let TA = try_!(Type::try_from(TA));
    match &TA.contents {
        TypeContents::VEC(element_id) => dispatch!(monomorphize_vec, [
            (M, @dist_dataset),
            (try_!(Type::of_id(element_id)), @primitives)
        ], ()),
        _ => dispatch!(monomorphize_scalar, [
            (M, @dist_dataset),
            (&TA, @primitives)
        ], ())
    }
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_is_equal(
    value: *const AnyObject,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TIA = try_!(Type::try_from(TIA));

    fn monomorphize<TIA>(value: *const AnyObject) -> FfiResult<*mut AnyTransformation> where
        TIA: 'static + Clone + PartialEq + CheckNull + Debug {
        let value: TIA = try_!(try_as_ref!(value).downcast_ref::<TIA>()).clone();
        make_is_equal::<TIA>(value).into_any()
    }
    dispatch!(monomorphize, [(TIA, @primitives)], (value))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_is_null(
    DIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DIA = try_!(Type::try_from(DIA));
    let TIA = try_!(DIA.get_domain_atom());

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionNullDomain" => {
            fn monomorphize<TIA>() -> FfiResult<*mut AnyTransformation>
                where TIA: 'static + CheckNull + Debug {
                make_is_null::<OptionNullDomain<AllDomain<TIA>>>().into_any()
            }
            dispatch!(monomorphize, [(TIA, @primitives)], ())
        }
        TypeContents::GENERIC { name, .. } if name == &"InherentNullDomain" => {
            fn monomorphize<TIA>() -> FfiResult<*mut AnyTransformation>
                where TIA: 'static + InherentNull + Debug {
                make_is_null::<InherentNullDomain<AllDomain<TIA>>>().into_any()
            }
            dispatch!(monomorphize, [(TIA, [f64, f32])], ())
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

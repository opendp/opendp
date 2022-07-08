use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::core::InherentNull;
use crate::err;
use crate::ffi::any::AnyTransformation;
use crate::ffi::util::Type;
use crate::traits::{CheckNull, RoundCast};
use crate::trans::{make_cast, make_cast_default, make_cast_inherent};

#[no_mangle]
pub extern "C" fn opendp_trans__make_cast(
    TIA: *const c_char, TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    fn monomorphize<TIA, TOA>() -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + Clone + CheckNull,
              TOA: 'static + RoundCast<TIA> + CheckNull {
        make_cast::<TIA, TOA>().into_any()
    }
    dispatch!(monomorphize, [(TIA, @primitives), (TOA, @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_default(
    TIA: *const c_char, TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    fn monomorphize<TIA, TOA>() -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + Clone + CheckNull,
              TOA: 'static + RoundCast<TIA> + Default + CheckNull {
        make_cast_default::<TIA, TOA>().into_any()
    }
    dispatch!(monomorphize, [(TIA, @primitives), (TOA, @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_inherent(
    TIA: *const c_char, TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    fn monomorphize<TIA, TOA>() -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + Clone + CheckNull,
              TOA: 'static + RoundCast<TIA> + InherentNull {
        make_cast_inherent::<TIA, TOA>().into_any()
    }
    dispatch!(monomorphize, [(TIA, @primitives), (TOA, @floats)], ())
}


#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_cast() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast(
            "i32".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<Option<f64>> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![Some(1.0), Some(2.0), Some(3.0)]);
        Ok(())
    }

    #[test]
    fn test_make_cast_default() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast_default(
            "String".to_char_p(),
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec!["a".to_string(), "1".to_string()]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0, 1]);
        Ok(())
    }

    #[test]
    fn test_make_cast_inherent() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast_inherent(
            "String".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec!["a".to_string(), "1".to_string()]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert!(res[0].is_nan());
        assert_eq!(res[1], 1.);
        Ok(())
    }
}

use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::core::DatasetMetric;
use crate::dist::{SubstituteDistance, SymmetricDistance};
use crate::dom::{AllDomain, InherentNull, VectorDomain};
use crate::err;
use crate::ffi::any::AnyTransformation;
use crate::ffi::util::Type;
use crate::traits::{CheckNull, RoundCast};
use crate::trans::{DatasetMetricCast, make_cast, make_cast_default, make_cast_inherent, make_cast_metric};

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

// The scope of this function has been reduced in the FFI layer from accepting any arbitrary domain,
//      to assuming the domain is VectorDomain<AllDomain<T>>.
// This is because we don't have an established way of passing arbitrary domains over FFI
#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_metric(
    MI: *const c_char, MO: *const c_char, TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<MI, MO, TA>() -> FfiResult<*mut AnyTransformation>
        where MI: 'static + DatasetMetric,
              MO: 'static + DatasetMetric,
              (MI, MO): DatasetMetricCast,
              TA: 'static + Clone + CheckNull {
        make_cast_metric::<VectorDomain<AllDomain<TA>>, MI, MO>(
            VectorDomain::new_all()
        ).into_any()
    }
    dispatch!(monomorphize, [(MI, @dist_dataset), (MO, @dist_dataset), (TA, @primitives)], ())
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

    #[test]
    fn test_make_cast_metric() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast_metric(
            "SymmetricDistance".to_char_p(),
            "SubstituteDistance".to_char_p(),
            "String".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec!["a".to_string(), "b".to_string()]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<String> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec!["a".to_string(), "b".to_string()]);
        Ok(())
    }
}

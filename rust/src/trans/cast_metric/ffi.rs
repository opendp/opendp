use std::{os::raw::c_char, convert::TryFrom};

use crate::{
    core::{FfiResult, DatasetMetric, IntoAnyTransformationFfiResultExt}, 
    dist::{InsertDeleteDistance, SymmetricDistance},
    ffi::{any::AnyTransformation, util::Type},
    trans::{MetricCast, make_cast_metric}, 
    dom::{VectorDomain, AllDomain}, 
    traits::CheckNull,
};


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
              MO: MetricCast<MI, Vec<TA>>,
              TA: 'static + Clone + CheckNull {
        make_cast_metric::<VectorDomain<AllDomain<TA>>, MI, MO>(
            VectorDomain::new_all()
        ).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, InsertDeleteDistance]), 
        (MO, [SymmetricDistance, InsertDeleteDistance]), 
        (TA, @primitives)
    ], ())

}


#[cfg(test)]
mod tests {
    use crate::{error::Fallible, ffi::{any::{AnyObject, Downcast}, util::ToCharP}};
    use crate::core;

    use super::*;

    #[test]
    fn test_make_cast_metric() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast_metric(
            "SymmetricDistance".to_char_p(),
            "ChangeOneDistance".to_char_p(),
            "String".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec!["a".to_string(), "b".to_string()]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<String> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec!["a".to_string(), "b".to_string()]);
        Ok(())
    }
}

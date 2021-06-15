use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::dom::{InherentNull, AllDomain, VectorDomain};
use opendp::err;
use opendp::traits::CastFrom;
use opendp::trans::{make_cast, make_cast_default, make_is_equal, make_cast_inherent, make_cast_metric, DatasetMetricCast};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{Type};



#[no_mangle]
pub extern "C" fn opendp_trans__make_cast(
    M: *const c_char, TI: *const c_char, TO: *const c_char
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    fn monomorphize<M, TI, TO>() -> FfiResult<*mut AnyTransformation> where
        M: 'static + DatasetMetric,
        TI: 'static + Clone,
        TO: 'static + CastFrom<TI> {
        make_cast::<M, TI, TO>().into_any()
    }
    dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives), (TO, @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_default(
    M: *const c_char, TI: *const c_char, TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    fn monomorphize<M, TI, TO>() -> FfiResult<*mut AnyTransformation> where
        M: 'static + DatasetMetric,
        TI: 'static + Clone,
        TO: 'static + CastFrom<TI> + Default {
        make_cast_default::<M, TI, TO>().into_any()
    }
    dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives), (TO, @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_is_equal(
    value: *const c_void,
    M: *const c_char, TI: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let TI = try_!(Type::try_from(TI));

    fn monomorphize<M, TI>(value: *const c_void) -> FfiResult<*mut AnyTransformation> where
        M: 'static + DatasetMetric,
        TI: 'static + Clone + PartialEq {
        let value = try_as_ref!(value as *const TI).clone();
        make_is_equal::<M, TI>(value).into_any()
    }
    dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives)], (value))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_inherent(
    M: *const c_char, TI: *const c_char, TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    fn monomorphize<M, TI, TO>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              TI: 'static + Clone,
              TO: 'static + CastFrom<TI> + InherentNull {
        make_cast_inherent::<M, TI, TO>().into_any()
    }
    dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives), (TO, @floats)], ())
}

// The scope of this function has been reduced in the FFI layer from accepting any arbitrary domain,
//      to assuming the domain is VectorDomain<AllDomain<T>>.
// This is because we don't have an established way of passing arbitrary domains over FFI
#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_metric(
    MI: *const c_char, MO: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let T = try_!(Type::try_from(T));

    fn monomorphize<MI, MO, T>() -> FfiResult<*mut AnyTransformation>
        where MI: 'static + DatasetMetric,
              MO: 'static + DatasetMetric,
              (MI, MO): DatasetMetricCast,
              T: 'static + Clone {
        make_cast_metric::<VectorDomain<AllDomain<T>>, MI, MO>(
            VectorDomain::new_all()
        ).into_any()
    }
    dispatch!(monomorphize, [(MI, @dist_dataset), (MO, @dist_dataset), (T, @primitives)], ())
}


#[cfg(test)]
mod tests {
    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_cast_vec() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast(
            "SymmetricDistance".to_char_p(),
            "i32".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<Option<f64>> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![Some(1.0), Some(2.0), Some(3.0)]);
        Ok(())
    }
}

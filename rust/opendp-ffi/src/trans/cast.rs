use std::convert::TryFrom;
use std::os::raw::c_char;

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::dom::InherentNull;
use opendp::err;
use opendp::traits::CastFrom;
use opendp::trans::make_cast;

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{Type, to_bool, c_bool};

#[no_mangle]
pub extern "C" fn opendp_trans__make_cast(
    M: *const c_char, TI: *const c_char, TO: *const c_char, inherent: c_bool
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    if to_bool(inherent) {
        fn monomorphize<M, TI, TO>() -> FfiResult<*mut AnyTransformation> where
            M: 'static + DatasetMetric,
            TI: 'static + Clone,
            TO: 'static + CastFrom<TI> + InherentNull {
            make_cast::<M, TI, TO>().into_any()
        }
        dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives), (TO, @floats)], ())
    } else {
        fn monomorphize<M, TI, TO>() -> FfiResult<*mut AnyTransformation> where
            M: 'static + DatasetMetric,
            TI: 'static + Clone,
            TO: 'static + CastFrom<TI> {
            make_cast::<M, TI, TO>().into_any()
        }
        dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives), (TO, @primitives)], ())
    }
}


#[cfg(test)]
mod tests {
    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util::{ToCharP, from_bool};

    use super::*;

    #[test]
    fn test_make_cast_vec() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast(
            "SymmetricDistance".to_char_p(),
            "i32".to_char_p(),
            "f64".to_char_p(),
            from_bool(true),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<Option<f64>> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![Some(1.0), Some(2.0), Some(3.0)]);
        Ok(())
    }
}

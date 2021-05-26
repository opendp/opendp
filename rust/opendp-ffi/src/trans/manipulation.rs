use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use num::One;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::traits::{CastFrom, DistanceConstant};
use opendp::trans::{make_cast_vec, make_clamp, make_clamp_vec, make_identity};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{Type, TypeContents};

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize_scalar<M, T>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              M::Distance: DistanceConstant + One,
              T: 'static + Clone {
        make_identity::<AllDomain<T>, M>(AllDomain::<T>::new(), M::default()).into_any()
    }
    fn monomorphize_vec<M, T>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              M::Distance: DistanceConstant + One,
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
pub extern "C" fn opendp_trans__make_clamp(
    lower: *const c_void, upper: *const c_void,
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<Q>(
        lower: *const c_void, upper: *const c_void,
        M: Type, T: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where Q: DistanceConstant + One {
        fn monomorphize2<M, T>(
            lower: *const c_void, upper: *const c_void,
        ) -> FfiResult<*mut AnyTransformation>
            where M: 'static + SensitivityMetric,
                  T: 'static + Clone + PartialOrd,
                  M::Distance: DistanceConstant + One {
            let lower = try_as_ref!(lower as *const T).clone();
            let upper = try_as_ref!(upper as *const T).clone();
            make_clamp::<M, T>(lower, upper).into_any()
        }
        dispatch!(monomorphize2, [
            (M, [L1Sensitivity<Q>, L2Sensitivity<Q>]),
            (T, @numbers)
        ], (lower, upper))
    }
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));
    let Q = try_!(M.get_sensitivity_distance());

    dispatch!(monomorphize, [(Q, @numbers)], (lower, upper, M, T))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp_vec(
    lower: *const c_void, upper: *const c_void,
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, T>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric + Clone,
              T: 'static + Copy + PartialOrd,
              M::Distance: DistanceConstant + One {
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        make_clamp_vec::<M, T>(lower, upper).into_any()
    }
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(M, @dist_dataset), (T, @numbers)], (lower, upper))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_vec(
    M: *const c_char, TI: *const c_char, TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, TI, TO>() -> FfiResult<*mut AnyTransformation> where
        M: 'static + DatasetMetric<Distance=u32>,
        TI: 'static + Clone,
        TO: 'static + CastFrom<TI> + Default {
        make_cast_vec::<M, TI, TO>().into_any()
    }
    let M = try_!(Type::try_from(M));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives), (TO, @primitives)], ())
}


#[cfg(test)]
mod tests {
    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util;
    use crate::util::ToCharP;

    use super::*;

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
    fn test_make_clamp() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_clamp(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            "L2Sensitivity<f64>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(-1.0);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 0.0);
        Ok(())
    }

    #[test]
    fn test_make_clamp_vec() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_clamp_vec(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            "SymmetricDistance".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![-1.0, 5.0, 11.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0.0, 5.0, 10.0]);
        Ok(())
    }

    #[test]
    fn test_make_cast_vec() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_cast_vec(
            "SymmetricDistance".to_char_p(),
            "i32".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }
}

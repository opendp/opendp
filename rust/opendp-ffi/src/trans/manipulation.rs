use std::os::raw::{c_char, c_void};

use num::One;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::traits::{CastFrom, DistanceConstant};
use opendp::trans::{make_cast_vec, make_clamp, make_clamp_vec, make_identity};

use crate::core::{FfiResult, FfiTransformation};
use crate::util::{Type, TypeContents};
use std::convert::TryFrom;

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(
    M: *const c_char, T: *const c_char
) -> FfiResult<*mut FfiTransformation> {

    fn monomorphize_scalar<M, T>() -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric,
              M::Distance: DistanceConstant + One,
              T: 'static + Clone {
        make_identity::<AllDomain<T>, M>(AllDomain::<T>::new(), M::default()).into()
    }
    fn monomorphize_vec<M, T>() -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric,
              M::Distance: DistanceConstant + One,
              T: 'static + Clone {
        make_identity::<VectorDomain<AllDomain<T>>, M>(VectorDomain::new(AllDomain::<T>::new()), M::default()).into()
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
pub extern "C" fn opendp_trans__make_clamp_vec(
    lower: *const c_void, upper: *const c_void,
    M: *const c_char, T: *const c_char
) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, T>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric + Clone,
              T: 'static + Copy + PartialOrd,
              M::Distance: DistanceConstant + One {
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        make_clamp_vec::<M, T>(lower, upper).into()
    }
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(M, @dist_dataset), (T, @numbers)], (lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp_scalar(
    lower: *const c_void, upper: *const c_void,
    M: *const c_char, T: *const c_char
) -> FfiResult<*mut FfiTransformation> {

    fn monomorphize<Q>(
        lower: *const c_void, upper: *const c_void,
        M: Type, T: Type
    ) -> FfiResult<*mut FfiTransformation>
        where Q: DistanceConstant + One {

        fn monomorphize2<M, T>(
            lower: *const c_void, upper: *const c_void
        ) -> FfiResult<*mut FfiTransformation>
            where M: 'static + SensitivityMetric,
                  T: 'static + Clone + PartialOrd,
                  M::Distance: DistanceConstant + One {
            let lower = try_as_ref!(lower as *const T).clone();
            let upper = try_as_ref!(upper as *const T).clone();
            make_clamp::<M, T>(lower, upper).into()
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
pub extern "C" fn opendp_trans__make_cast_vec(
    M: *const c_char, TI: *const c_char, TO: *const c_char
) -> FfiResult<*mut FfiTransformation> {

    fn monomorphize<M, TI, TO>() -> FfiResult<*mut FfiTransformation> where
        M: 'static + DatasetMetric<Distance=u32>,
        TI: 'static + Clone,
        TO: 'static + CastFrom<TI> + Default {
        make_cast_vec::<M, TI, TO>().into()
    }
    let M = try_!(Type::try_from(M));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [(M, @dist_dataset), (TI, @primitives), (TO, @primitives)], ())
}

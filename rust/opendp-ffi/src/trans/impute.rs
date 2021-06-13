use std::convert::TryFrom;
use std::ops::{Add, Mul, Sub};
use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::dom::{AllDomain, InherentNull, InherentNullDomain, OptionNullDomain};
use opendp::err;
use opendp::samplers::SampleUniform;
use opendp::trans::{ImputableDomain, make_impute_constant, make_impute_uniform_float};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{c_bool, to_bool, Type};

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_uniform_float(
    lower: *const c_void, upper: *const c_void,
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));

    fn monomorphize<M, T>(
        lower: *const c_void, upper: *const c_void,
    ) -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric,
              for<'a> T: 'static + Float + SampleUniform + Clone + Sub<Output=T> + Mul<&'a T, Output=T> + Add<&'a T, Output=T> + InherentNull {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_impute_uniform_float::<M, T>(
            lower, upper,
        ).into_any()
    }
    dispatch!(monomorphize, [(M, @dist_dataset), (T, @floats)], (lower, upper))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_constant(
    constant: *const c_void, inherent: c_bool,
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));

    if to_bool(inherent) {

        fn monomorphize_inherent<M, T>(
            constant: *const c_void
        ) -> FfiResult<*mut AnyTransformation>
            where InherentNullDomain<AllDomain<T>>: ImputableDomain<NonNull=T>,
                  M: 'static + DatasetMetric,
                  T: 'static + Clone + InherentNull {
            let constant = try_as_ref!(constant as *const T).clone();
            make_impute_constant::<InherentNullDomain<AllDomain<T>>, M>(
                constant
            ).into_any()
        }
        dispatch!(monomorphize_inherent, [(M, @dist_dataset), (T, @floats)], (constant))
    } else {

        fn monomorphize_option<M, T>(
            constant: *const c_void
        ) -> FfiResult<*mut AnyTransformation>
            where OptionNullDomain<AllDomain<T>>: ImputableDomain<NonNull=T>,
                  M: 'static + DatasetMetric,
                  T: 'static + Clone {
            let constant = try_as_ref!(constant as *const T).clone();
            make_impute_constant::<OptionNullDomain<AllDomain<T>>, M>(
                constant,
            ).into_any()
        }
        dispatch!(monomorphize_option, [(M, @dist_dataset), (T, @primitives)], (constant))
    }
}
use std::convert::TryFrom;
use std::ops::{Add, Mul, Sub};
use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::dom::{AllDomain, InherentNull, InherentNullDomain, OptionNullDomain};
use opendp::err;
use opendp::samplers::SampleUniform;
use opendp::trans::{ImputableDomain, make_impute_constant, make_impute_uniform_float};

use crate::any::{AnyTransformation, AnyObject, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{Type, TypeContents};
use opendp::traits::CheckNull;
use std::fmt::Debug;

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_uniform_float(
    lower: *const c_void, upper: *const c_void,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let T = try_!(Type::try_from(T));

    fn monomorphize<T>(
        lower: *const c_void, upper: *const c_void,
    ) -> FfiResult<*mut AnyTransformation>
        where for<'a> T: 'static + Float + SampleUniform + Clone + Sub<Output=T> + Mul<&'a T, Output=T> + Add<&'a T, Output=T> + InherentNull + Debug {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_impute_uniform_float::<T>(
            lower, upper,
        ).into_any()
    }
    dispatch!(monomorphize, [(T, @floats)], (lower, upper))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_constant(
    constant: *const AnyObject,
    DA: *const c_char
) -> FfiResult<*mut AnyTransformation> {
    let DA = try_!(Type::try_from(DA));
    let T = try_!(DA.get_atom());

    match &DA.contents {
        TypeContents::GENERIC {name, ..} if name == &"OptionNullDomain" => {
            fn monomorphize<T>(
                constant: *const AnyObject
            ) -> FfiResult<*mut AnyTransformation>
                where OptionNullDomain<AllDomain<T>>: ImputableDomain<Imputed=T>,
                      T: 'static + Clone + CheckNull + Debug {
                let constant: T = try_!(try_as_ref!(constant).downcast_ref::<T>()).clone();
                make_impute_constant::<OptionNullDomain<AllDomain<T>>>(constant).into_any()
            }
            dispatch!(monomorphize, [(T, @primitives)], (constant))
        }
        TypeContents::GENERIC {name, ..} if name == &"InherentNullDomain" => {
            fn monomorphize<T>(
                constant: *const AnyObject
            ) -> FfiResult<*mut AnyTransformation>
                where InherentNullDomain<AllDomain<T>>: ImputableDomain<Imputed=T>,
                      T: 'static + InherentNull + Clone + Debug {
                let constant: T = try_!(try_as_ref!(constant).downcast_ref::<T>()).clone();
                make_impute_constant::<InherentNullDomain<AllDomain<T>>>(constant).into_any()
            }
            dispatch!(monomorphize, [(T, [f64, f32])], (constant))
        },
        _ => err!(TypeParse, "DA must be an OptionNullDomain<AllDomain<T>> or an InherentNullDomain<AllDomain<T>>").into()
    }
}
use std::convert::TryFrom;
use std::ops::{Add, Mul, Sub};
use std::os::raw::c_char;

use num::Float;

use opendp::dom::{AllDomain, InherentNull, InherentNullDomain, OptionNullDomain};
use opendp::err;
use opendp::samplers::SampleUniform;
use opendp::trans::{ImputableDomain, make_impute_constant, make_impute_uniform_float};

use crate::any::{AnyTransformation, AnyObject, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{Type, TypeContents};
use opendp::traits::CheckNull;

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_uniform_float(
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let T = try_!(Type::try_from(T));

    fn monomorphize<T>(
        bounds: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
        where for<'a> T: 'static + Float + SampleUniform + Clone + Sub<Output=T> + Mul<&'a T, Output=T> + Add<&'a T, Output=T> + InherentNull {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_impute_uniform_float::<T>(bounds).into_any()
    }
    dispatch!(monomorphize, [(T, @floats)], (bounds))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_constant(
    constant: *const AnyObject,
    DA: *const c_char
) -> FfiResult<*mut AnyTransformation> {
    let DA = try_!(Type::try_from(DA));
    let T = try_!(DA.get_domain_atom());

    match &DA.contents {
        TypeContents::GENERIC {name, ..} if name == &"OptionNullDomain" => {
            fn monomorphize<T>(
                constant: *const AnyObject
            ) -> FfiResult<*mut AnyTransformation>
                where OptionNullDomain<AllDomain<T>>: ImputableDomain<Imputed=T>,
                      T: 'static + Clone + CheckNull{
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
                      T: 'static + InherentNull + Clone {
                let constant: T = try_!(try_as_ref!(constant).downcast_ref::<T>()).clone();
                make_impute_constant::<InherentNullDomain<AllDomain<T>>>(constant).into_any()
            }
            dispatch!(monomorphize, [(T, [f64, f32])], (constant))
        },
        _ => err!(TypeParse, "DA must be an OptionNullDomain<AllDomain<T>> or an InherentNullDomain<AllDomain<T>>").into()
    }
}
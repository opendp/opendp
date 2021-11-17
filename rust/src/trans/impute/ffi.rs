use std::convert::TryFrom;
use std::ops::{Add, Mul, Sub};
use std::os::raw::c_char;

use num::Float;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dom::{AllDomain, InherentNull, InherentNullDomain, OptionNullDomain};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::samplers::SampleUniform;
use crate::traits::CheckNull;
use crate::trans::{DropNullDomain, ImputeConstantDomain, make_drop_null, make_impute_constant, make_impute_uniform_float};

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_uniform_float(
    bounds: *const AnyObject,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<TA>(
        bounds: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
        where for<'a> TA: 'static + Float + SampleUniform + Clone + Sub<Output=TA> + Mul<&'a TA, Output=TA> + Add<&'a TA, Output=TA> + InherentNull {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(TA, TA)>()).clone();
        make_impute_uniform_float::<TA>(bounds).into_any()
    }
    dispatch!(monomorphize, [(TA, @floats)], (bounds))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_impute_constant(
    constant: *const AnyObject,
    DA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DA = try_!(Type::try_from(DA));
    let TA = try_!(DA.get_atom());

    match &DA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionNullDomain" => {
            fn monomorphize<TA>(
                constant: *const AnyObject
            ) -> FfiResult<*mut AnyTransformation>
                where OptionNullDomain<AllDomain<TA>>: ImputeConstantDomain<Imputed=TA>,
                      TA: 'static + Clone + CheckNull {
                let constant: TA = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
                make_impute_constant::<OptionNullDomain<AllDomain<TA>>>(constant).into_any()
            }
            dispatch!(monomorphize, [(TA, @primitives)], (constant))
        }
        TypeContents::GENERIC { name, .. } if name == &"InherentNullDomain" => {
            fn monomorphize<TA>(
                constant: *const AnyObject
            ) -> FfiResult<*mut AnyTransformation>
                where InherentNullDomain<AllDomain<TA>>: ImputeConstantDomain<Imputed=TA>,
                      TA: 'static + InherentNull + Clone {
                let constant: TA = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
                make_impute_constant::<InherentNullDomain<AllDomain<TA>>>(constant).into_any()
            }
            dispatch!(monomorphize, [(TA, [f64, f32])], (constant))
        }
        _ => err!(TypeParse, "DA must be an OptionNullDomain<AllDomain<T>> or an InherentNullDomain<AllDomain<T>>").into()
    }
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_drop_null(
    DA: *const c_char
) -> FfiResult<*mut AnyTransformation> {
    let DA = try_!(Type::try_from(DA));
    let TA = try_!(DA.get_atom());

    match &DA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionNullDomain" => {
            fn monomorphize<TA>() -> FfiResult<*mut AnyTransformation>
                where OptionNullDomain<AllDomain<TA>>: DropNullDomain<Imputed=TA>,
                      TA: 'static + Clone + CheckNull {
                make_drop_null::<OptionNullDomain<AllDomain<TA>>>().into_any()
            }
            dispatch!(monomorphize, [(TA, @primitives)], ())
        }
        TypeContents::GENERIC { name, .. } if name == &"InherentNullDomain" => {
            fn monomorphize<TA>() -> FfiResult<*mut AnyTransformation>
                where InherentNullDomain<AllDomain<TA>>: DropNullDomain<Imputed=TA>,
                      TA: 'static + InherentNull + Clone {
                make_drop_null::<InherentNullDomain<AllDomain<TA>>>().into_any()
            }
            dispatch!(monomorphize, [(TA, [f64, f32])], ())
        }
        _ => err!(TypeParse, "DA must be an OptionNullDomain<AllDomain<T>> or an InherentNullDomain<AllDomain<T>>").into()
    }
}
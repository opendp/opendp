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
    DA: *const c_char
) -> FfiResult<*mut AnyTransformation> {
    let DA = try_!(Type::try_from(DA));
    let TA = try_!(DA.get_domain_atom());

    match &DA.contents {
        TypeContents::GENERIC {name, ..} if name == &"OptionNullDomain" => {
            fn monomorphize<TA>(
                constant: *const AnyObject
            ) -> FfiResult<*mut AnyTransformation>
                where OptionNullDomain<AllDomain<TA>>: ImputableDomain<Imputed=TA>,
                      TA: 'static + Clone + CheckNull{
                let constant: TA = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
                make_impute_constant::<OptionNullDomain<AllDomain<TA>>>(constant).into_any()
            }
            dispatch!(monomorphize, [(TA, @primitives)], (constant))
        }
        TypeContents::GENERIC {name, ..} if name == &"InherentNullDomain" => {
            fn monomorphize<TA>(
                constant: *const AnyObject
            ) -> FfiResult<*mut AnyTransformation>
                where InherentNullDomain<AllDomain<TA>>: ImputableDomain<Imputed=TA>,
                      TA: 'static + InherentNull + Clone {
                let constant: TA = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
                make_impute_constant::<InherentNullDomain<AllDomain<TA>>>(constant).into_any()
            }
            dispatch!(monomorphize, [(TA, [f64, f32])], (constant))
        },
        _ => err!(TypeParse, "DA must be an OptionNullDomain<AllDomain<T>> or an InherentNullDomain<AllDomain<T>>").into()
    }
}
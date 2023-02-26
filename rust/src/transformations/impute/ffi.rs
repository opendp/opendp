use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::domains::{AtomDomain, OptionDomain};
use crate::err;
use crate::ffi::any::{AnyDomain, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::traits::samplers::SampleUniform;
use crate::traits::{CheckAtom, Float, InherentNull};
use crate::transformations::{
    make_drop_null, make_impute_constant, make_impute_uniform_float, DropNullDomain,
    ImputeConstantDomain,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_impute_uniform_float(
    bounds: *const AnyObject,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<TA>(bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        TA: Float + SampleUniform,
    {
        let bounds = *try_!(try_as_ref!(bounds).downcast_ref::<(TA, TA)>());
        make_impute_uniform_float::<TA>(bounds).into_any()
    }
    dispatch!(monomorphize, [(TA, @floats)], (bounds))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_impute_constant(
    atom_input_domain: *const AnyDomain,
    constant: *const AnyObject,
    DIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DIA = try_!(Type::try_from(DIA));
    let TA = try_!(DIA.get_atom());

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionDomain" => {
            fn monomorphize<TA>(
                atom_input_domain: *const AnyDomain,
                constant: *const AnyObject,
            ) -> FfiResult<*mut AnyTransformation>
            where
                OptionDomain<AtomDomain<TA>>: ImputeConstantDomain<Imputed = TA>,
                TA: 'static + Clone + CheckAtom,
            {
                let atom_input_domain =
                    try_!(try_as_ref!(atom_input_domain)
                        .downcast_ref::<OptionDomain<AtomDomain<TA>>>())
                    .clone();
                let constant: TA = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
                make_impute_constant::<OptionDomain<AtomDomain<TA>>>(atom_input_domain, constant)
                    .into_any()
            }
            dispatch!(monomorphize, [(TA, @primitives)], (atom_input_domain, constant))
        }
        TypeContents::GENERIC { name, .. } if name == &"AtomDomain" => {
            fn monomorphize<TA>(
                atom_input_domain: *const AnyDomain,
                constant: *const AnyObject,
            ) -> FfiResult<*mut AnyTransformation>
            where
                AtomDomain<TA>: ImputeConstantDomain<Imputed = TA>,
                TA: 'static + InherentNull + Clone + CheckAtom,
            {
                let atom_input_domain =
                    try_!(try_as_ref!(atom_input_domain).downcast_ref::<AtomDomain<TA>>()).clone();
                let constant: TA = try_!(try_as_ref!(constant).downcast_ref::<TA>()).clone();
                make_impute_constant::<AtomDomain<TA>>(atom_input_domain, constant).into_any()
            }
            dispatch!(
                monomorphize,
                [(TA, [f64, f32])],
                (atom_input_domain, constant)
            )
        }
        _ => err!(
            TypeParse,
            "DA must be an OptionDomain<AtomDomain<T>> or an AtomDomain<T>"
        )
        .into(),
    }
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_drop_null(
    atom_domain: *const AnyDomain,
    DA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DA = try_!(Type::try_from(DA));
    let TA = try_!(DA.get_atom());

    match &DA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionDomain" => {
            fn monomorphize<TA: CheckAtom>(
                atom_domain: *const AnyDomain,
            ) -> FfiResult<*mut AnyTransformation>
            where
                OptionDomain<AtomDomain<TA>>: DropNullDomain<Imputed = TA>,
                TA: 'static + Clone + CheckAtom,
            {
                let atom_domain =
                    try_!(try_as_ref!(atom_domain).downcast_ref::<OptionDomain<AtomDomain<TA>>>())
                        .clone();
                make_drop_null(atom_domain).into_any()
            }
            dispatch!(monomorphize, [(TA, @primitives)], (atom_domain))
        }
        TypeContents::GENERIC { name, .. } if name == &"AtomDomain" => {
            fn monomorphize<TA: CheckAtom>(
                atom_domain: *const AnyDomain,
            ) -> FfiResult<*mut AnyTransformation>
            where
                AtomDomain<TA>: DropNullDomain<Imputed = TA>,
                TA: 'static + InherentNull + Clone + CheckAtom,
            {
                let atom_domain =
                    try_!(try_as_ref!(atom_domain).downcast_ref::<AtomDomain<TA>>()).clone();
                make_drop_null(atom_domain).into_any()
            }
            dispatch!(monomorphize, [(TA, [f64, f32])], (atom_domain))
        }
        _ => err!(
            TypeParse,
            "DA must be an OptionDomain<AtomDomain<T>> or an AtomDomain<T>"
        )
        .into(),
    }
}

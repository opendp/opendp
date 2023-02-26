use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::domains::AtomDomain;
use crate::err;
use crate::ffi::any::Downcast;
use crate::ffi::any::{AnyDomain, AnyObject, AnyTransformation};
use crate::ffi::util::{Type, TypeContents};
use crate::metrics::{InsertDeleteDistance, IntDistance, SymmetricDistance};
use crate::traits::CheckAtom;
use crate::transformations::resize::IsMetricOrdered;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_resize(
    size: c_uint,
    atom_domain: *const AnyDomain,
    constant: *const AnyObject,
    DA: *const c_char,
    MI: *const c_char,
    MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let size = size as usize;
    let atom_domain = try_as_ref!(atom_domain);
    let constant = try_as_ref!(constant);
    let DA = try_!(Type::try_from(DA));
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));

    if DA != atom_domain.type_ {
        return err!(FFI, "DA must match atom_domain's type").into();
    }

    fn monomorphize_all<MI, MO, T: 'static + CheckAtom + Clone>(
        size: usize,
        atom_domain: &AnyDomain,
        constant: &AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        MI: 'static + IsMetricOrdered<Distance = IntDistance>,
        MO: 'static + IsMetricOrdered<Distance = IntDistance>,
    {
        let atom_domain = try_!(atom_domain.downcast_ref::<AtomDomain<T>>()).clone();
        let constant = try_!(constant.downcast_ref::<T>()).clone();
        super::make_resize::<_, MI, MO>(size, atom_domain, constant).into_any()
    }

    match atom_domain.type_.contents {
        TypeContents::GENERIC {
            name: "AtomDomain", ..
        } => dispatch!(monomorphize_all, [
                (MI, [SymmetricDistance, InsertDeleteDistance]),
                (MO, [SymmetricDistance, InsertDeleteDistance]),
                (atom_domain.carrier_type, @primitives)
            ], (size, atom_domain, constant)),
        _ => err!(
            FFI,
            "VectorDomain constructors only supports the AtomDomain inner domain"
        )
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::opendp_core__transformation_invoke;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_resize(
            4 as c_uint,
            util::into_raw(AnyDomain::new(AtomDomain::<i32>::default())),
            AnyObject::new_raw(0i32),
            "AtomDomain<i32>".to_char_p(),
            "SymmetricDistance".to_char_p(),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }

    #[test]
    fn test_make_bounded_resize() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_resize(
            4 as c_uint,
            util::into_raw(AnyDomain::new(
                AtomDomain::<i32>::new_closed((0i32, 10)).unwrap(),
            )),
            AnyObject::new_raw(0i32),
            "AtomDomain<i32>".to_char_p(),
            "SymmetricDistance".to_char_p(),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3, 0]);
        Ok(())
    }
}

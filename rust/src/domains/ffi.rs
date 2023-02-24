use std::ffi::{c_char};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Domain},
    domains::{AllDomain, BoundedDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyObject, Downcast},
        util::{self, c_bool, into_c_char_p, Type, TypeContents},
    },
    traits::{CheckNull, TotalOrd},
};

#[bootstrap(
    name = "member",
    arguments(this(hint = "Domain"), val(rust_type = "$domain_carrier_type(this)")),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check membership in a `domain`.
///
/// # Arguments
/// * `this` - The domain to check membership in.
/// * `val` - A potential element of the domain.
#[no_mangle]
pub extern "C" fn opendp_domains__member(
    this: *mut AnyDomain,
    val: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let this = try_as_ref!(this);
    let val = try_as_ref!(val);
    let status = try_!(this.member(val));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "domain_debug",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Debug a `domain`.
///
/// # Arguments
/// * `this` - The domain to debug (stringify).
#[no_mangle]
pub extern "C" fn opendp_domains__domain_debug(this: *mut AnyDomain) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(format!("{:?}", this))))
}

#[bootstrap(
    name = "domain_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the type of a `domain`.
///
/// # Arguments
/// * `this` - The domain to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_domains__domain_type(this: *mut AnyDomain) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.type_.descriptor.to_string())))
}

#[bootstrap(
    name = "domain_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the carrier type of a `domain`.
///
/// # Arguments
/// * `this` - The domain to retrieve the carrier type from.
#[no_mangle]
pub extern "C" fn opendp_domains__domain_carrier_type(
    this: *mut AnyDomain,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(returns(c_type = "FfiResult<AnyDomain *>"))]
/// Construct an instance of `AllDomain`.
///
/// # Generics
/// * `T` - The type of the atom.
fn all_domain<T: CheckNull>() -> AllDomain<T> {
    AllDomain::<T>::new()
}

#[no_mangle]
pub extern "C" fn opendp_domains__all_domain(T: *const c_char) -> FfiResult<*mut AnyDomain> {
    fn monomorphize<T: 'static + CheckNull>() -> FfiResult<*mut AnyDomain> {
        Ok(AnyDomain::new(all_domain::<T>())).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @primitives)], ())
}

#[bootstrap(
    generics(T(example = "$get_first(bounds)")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `BoundedDomain`.
/// # Arguments
/// * `bounds` - A tuple of upper/lower bounds.
///
/// # Generics
/// * `T` - The type of the atom.
fn bounded_domain<T: TotalOrd>(bounds: (T, T)) -> Fallible<BoundedDomain<T>> {
    BoundedDomain::<T>::new_closed(bounds)
}

#[no_mangle]
pub extern "C" fn opendp_domains__bounded_domain(
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize<T: 'static + TotalOrd + Clone>(
        bounds: *const AnyObject,
    ) -> Fallible<AnyDomain> {
        let bounds = try_as_ref!(bounds).downcast_ref::<(T, T)>()?.clone();
        Ok(AnyDomain::new(bounded_domain::<T>(bounds)?))
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], (bounds)).into()
}

#[bootstrap(
    name = "vector_domain",
    arguments(size(rust_type = "Option<i32>", default = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `VectorDomain`.
/// # Arguments
/// * `atom_domain` - The inner domain.
#[no_mangle]
pub extern "C" fn opendp_domains__vector_domain(
    atom_domain: *const AnyDomain, size: *const AnyObject
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize_all<T: 'static + CheckNull>(atom_domain: &AnyDomain, size: *const AnyObject) -> Fallible<AnyDomain> {
        let size = if let Some(size) = util::as_ref(size) {
            Some(*try_!(size.downcast_ref::<i32>()) as usize)
        } else {
            None
        };
        let atom_domain = atom_domain.downcast_ref::<AllDomain<T>>()?.clone();
        Ok(AnyDomain::new(VectorDomain::new(atom_domain, size)))
    }
    fn monomorphize_bounded<T: 'static + TotalOrd + Clone>(
        atom_domain: &AnyDomain, size: *const AnyObject
    ) -> Fallible<AnyDomain> {
        let size = if let Some(size) = util::as_ref(size) {
            Some(*try_!(size.downcast_ref::<i32>()) as usize)
        } else {
            None
        };
        let atom_domain = atom_domain.downcast_ref::<BoundedDomain<T>>()?.clone();
        Ok(AnyDomain::new(VectorDomain::new(atom_domain, size)))
    }

    let atom_domain = try_as_ref!(atom_domain);

    match atom_domain.type_.contents {
        TypeContents::GENERIC { name: "AllDomain", .. } => 
            dispatch!(monomorphize_all, [(atom_domain.carrier_type, @primitives)], (atom_domain, size)),
        TypeContents::GENERIC { name: "BoundedDomain", .. } => 
            dispatch!(monomorphize_bounded, [(atom_domain.carrier_type, @numbers)], (atom_domain, size)),
        _ => fallible!(FFI, "VectorDomain constructors only support AllDomain and BoundedDomain atoms")
    }.into()
}

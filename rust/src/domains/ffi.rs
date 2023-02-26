use std::ffi::{c_char};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Domain},
    domains::{AllDomain, VectorDomain, type_name},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyObject, Downcast},
        util::{self, c_bool, into_c_char_p, Type, TypeContents},
    },
    traits::{CheckAtom, Float, Integer},
};

use super::{Bounds, Null};


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

#[bootstrap(
    arguments(
        bounds(rust_type = "Option<(T, T)>", c_type = "AnyObject *"), 
        nullable(rust_type = "bool", c_type = "bool")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `AllDomain`.
///
/// # Generics
/// * `T` - The type of the atom.
fn all_domain<T: CheckAtom>(bounds: Option<Bounds<T>>, nullable: Option<Null<T>>) -> AllDomain<T> {
    AllDomain::<T>::new(bounds, nullable)
}

#[no_mangle]
pub extern "C" fn opendp_domains__all_domain(
    T: *const c_char, 
    bounds: *const AnyObject, 
    nullable: c_bool
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize_float<T: 'static + Float>(bounds: *const AnyObject, nullable: bool) -> Fallible<AnyDomain> {
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            let tuple = *bounds.downcast_ref::<(T, T)>()?;
            Some(Bounds::new_closed(tuple)?)
        } else {
            None
        };

        let nullable = nullable.then_some(Null::new());
        Ok(AnyDomain::new(all_domain::<T>(bounds, nullable)))
    }
    fn monomorphize_integer<T: 'static + Integer>(bounds: *const AnyObject, nullable: bool) -> Fallible<AnyDomain> {
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            let tuple = *bounds.downcast_ref::<(T, T)>()?;
            Some(Bounds::new_closed(tuple)?)
        } else {
            None
        };
        if nullable {
            return fallible!(FFI, "integers cannot be null");
        }
        Ok(AnyDomain::new(all_domain::<T>(bounds, None)))
    }
    fn monomorphize_simple<T: 'static + CheckAtom>(bounds: *const AnyObject, nullable: bool) -> Fallible<AnyDomain> {
        if util::as_ref(bounds).is_some() {
            return fallible!(FFI, "{} cannot be bounded", type_name!(T));
        }
        if nullable {
            return fallible!(FFI, "{} cannot be null", type_name!(T));
        }
        Ok(AnyDomain::new(all_domain::<T>(None, None)))
    }
    let T = try_!(Type::try_from(T));
    let nullable = util::to_bool(nullable);

    if let Ok(domain) = dispatch!(monomorphize_float, [(T, @floats)], (bounds, nullable)) {
        Ok(domain)
    } else if let Ok(domain) = dispatch!(monomorphize_integer, [(T, @integers)], (bounds, nullable)) {
        Ok(domain)
    } else {
        dispatch!(monomorphize_simple, [(T, [bool, String])], (bounds, nullable))
    }.into()
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
    fn monomorphize_all<T: 'static + CheckAtom>(atom_domain: &AnyDomain, size: *const AnyObject) -> Fallible<AnyDomain> {
        let size = if let Some(size) = util::as_ref(size) {
            Some(*try_!(size.downcast_ref::<i32>()) as usize)
        } else {
            None
        };
        let atom_domain = atom_domain.downcast_ref::<AllDomain<T>>()?.clone();
        Ok(AnyDomain::new(VectorDomain::new(atom_domain, size)))
    }
    let atom_domain = try_as_ref!(atom_domain);

    match atom_domain.type_.contents {
        TypeContents::GENERIC { name: "AllDomain", .. } => 
            dispatch!(monomorphize_all, [(atom_domain.carrier_type, @primitives)], (atom_domain, size)),
        _ => fallible!(FFI, "VectorDomain constructors only support AllDomain inner domains")
    }.into()
}

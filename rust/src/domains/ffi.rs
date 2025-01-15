use std::{ffi::c_char, fmt::Debug};

use opendp_derive::bootstrap;

use crate::{
    core::{Domain, FfiResult, Function},
    domains::{type_name, AtomDomain, MapDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyObject, CallbackFn, Downcast},
        util::{self, c_bool, into_c_char_p, to_str, ExtrinsicObject, Type, TypeContents},
    },
    traits::{CheckAtom, Float, Hashable, Integer, Primitive},
};

#[cfg(feature = "polars")]
use crate::domains::{CategoricalDomain, DatetimeDomain};

use super::{BitVectorDomain, Bounds, Null, OptionDomain};

#[bootstrap(
    name = "_domain_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_domains___domain_free(this: *mut AnyDomain) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

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
        bounds(
            rust_type = "Option<(T, T)>",
            c_type = "AnyObject *",
            default = b"null"
        ),
        nullable(rust_type = "bool", c_type = "bool", default = false)
    ),
    generics(T(example = "$get_first(bounds)")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `AtomDomain`.
///
/// # Generics
/// * `T` - The type of the atom.
fn atom_domain<T: CheckAtom>(
    bounds: Option<Bounds<T>>,
    nullable: Option<Null<T>>,
) -> AtomDomain<T> {
    AtomDomain::<T>::new(bounds, nullable)
}

#[no_mangle]
pub extern "C" fn opendp_domains__atom_domain(
    bounds: *const AnyObject,
    nullable: c_bool,
    T: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize_float<T: 'static + Float>(
        bounds: *const AnyObject,
        nullable: bool,
    ) -> Fallible<AnyDomain> {
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            let tuple = *bounds.downcast_ref::<(T, T)>()?;
            Some(Bounds::new_closed(tuple)?)
        } else {
            None
        };

        let nullable = nullable.then_some(Null::new());
        Ok(AnyDomain::new(atom_domain::<T>(bounds, nullable)))
    }
    fn monomorphize_integer<T: 'static + Integer>(
        bounds: *const AnyObject,
        nullable: bool,
    ) -> Fallible<AnyDomain> {
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            let tuple = *bounds.downcast_ref::<(T, T)>()?;
            Some(Bounds::new_closed(tuple)?)
        } else {
            None
        };
        if nullable {
            return fallible!(FFI, "integers cannot be null");
        }
        Ok(AnyDomain::new(atom_domain::<T>(bounds, None)))
    }
    fn monomorphize_simple<T: 'static + CheckAtom>(
        bounds: *const AnyObject,
        nullable: bool,
    ) -> Fallible<AnyDomain> {
        if util::as_ref(bounds).is_some() {
            return fallible!(FFI, "{} cannot be bounded", type_name!(T));
        }
        if nullable {
            return fallible!(FFI, "{} cannot be null", type_name!(T));
        }
        Ok(AnyDomain::new(atom_domain::<T>(None, None)))
    }
    let T_ = try_!(Type::try_from(T));
    let nullable = util::to_bool(nullable);

    // This is used to check if the type is in a dispatch set,
    // without constructing an expensive backtrace upon failed match
    fn in_set<T>() -> Option<()> {
        Some(())
    }

    #[cfg(feature = "polars")]
    if let Some(_) = dispatch!(in_set, [(T_, [chrono::NaiveDate, chrono::NaiveTime])]) {
        return dispatch!(
            monomorphize_simple,
            [(T_, [chrono::NaiveDate, chrono::NaiveTime])],
            (bounds, nullable)
        )
        .into();
    };

    if let Some(_) = dispatch!(in_set, [(T_, [f32, f64])]) {
        dispatch!(monomorphize_float, [(T_, [f32, f64])], (bounds, nullable))
    } else if let Some(_) = dispatch!(
        in_set,
        [(
            T_,
            [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize]
        )]
    ) {
        dispatch!(
            monomorphize_integer,
            [(
                T_,
                [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize]
            )],
            (bounds, nullable)
        )
    } else {
        dispatch!(
            monomorphize_simple,
            [(T_, [bool, String])],
            (bounds, nullable)
        )
    }
    .into()
}

#[bootstrap(
    arguments(element_domain(c_type = "AnyDomain *")),
    generics(D(example = "element_domain")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `OptionDomain`.
///
/// # Generics
/// * `D` - The type of the inner domain.
fn option_domain<D: Domain>(element_domain: D) -> OptionDomain<D> {
    OptionDomain::<D>::new(element_domain)
}

#[no_mangle]
pub extern "C" fn opendp_domains__option_domain(
    element_domain: *const AnyDomain,
    D: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize_atom<T: 'static + CheckAtom>(
        element_domain: &AnyDomain,
    ) -> Fallible<AnyDomain> {
        let element_domain = element_domain.downcast_ref::<AtomDomain<T>>()?.clone();
        Ok(AnyDomain::new(option_domain(element_domain)))
    }

    let element_domain = try_as_ref!(element_domain);
    let D = try_!(Type::try_from(D));
    let T = try_!(D.get_atom());

    #[cfg(feature = "polars")]
    if D == Type::of::<CategoricalDomain>() {
        let element_domain = try_!(element_domain.downcast_ref::<CategoricalDomain>()).clone();
        return Ok(AnyDomain::new(option_domain(element_domain))).into();
    }
    #[cfg(feature = "polars")]
    if D == Type::of::<DatetimeDomain>() {
        let element_domain = try_!(element_domain.downcast_ref::<DatetimeDomain>()).clone();
        return Ok(AnyDomain::new(option_domain(element_domain))).into();
    }
    #[cfg(feature = "polars")]
    if T == Type::of::<chrono::NaiveDate>() {
        return monomorphize_atom::<chrono::NaiveDate>(element_domain).into();
    }
    #[cfg(feature = "polars")]
    if T == Type::of::<chrono::NaiveTime>() {
        return monomorphize_atom::<chrono::NaiveTime>(element_domain).into();
    }

    dispatch!(
        monomorphize_atom,
        [(
            T,
            [
                u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64, bool,
                String
            ]
        )],
        (element_domain)
    )
    .into()
}

#[bootstrap(
    name = "vector_domain",
    arguments(
        atom_domain(c_type = "AnyDomain *", rust_type = b"null"),
        size(rust_type = "Option<i32>", default = b"null")
    ),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `VectorDomain`.
///
/// # Arguments
/// * `atom_domain` - The inner domain.
#[no_mangle]
pub extern "C" fn opendp_domains__vector_domain(
    atom_domain: *const AnyDomain,
    size: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize_all<T: 'static + CheckAtom>(
        atom_domain: &AnyDomain,
        size: *const AnyObject,
    ) -> Fallible<AnyDomain> {
        let atom_domain = atom_domain.downcast_ref::<AtomDomain<T>>()?.clone();
        let mut vector_domain = VectorDomain::new(atom_domain);
        if let Some(size) = util::as_ref(size) {
            vector_domain = vector_domain.with_size(*try_!(size.downcast_ref::<i32>()) as usize)
        };
        Ok(AnyDomain::new(vector_domain))
    }
    fn monomorphize_user_domain(
        user_domain: &AnyDomain,
        size: *const AnyObject,
    ) -> Fallible<AnyDomain> {
        let user_domain = user_domain.downcast_ref::<ExtrinsicDomain>()?.clone();
        let mut vector_domain = VectorDomain::new(user_domain);
        if let Some(size) = util::as_ref(size) {
            vector_domain = vector_domain.with_size(*try_!(size.downcast_ref::<i32>()) as usize)
        };
        Ok(AnyDomain::new(vector_domain))
    }
    let atom_domain = try_as_ref!(atom_domain);

    match atom_domain.type_.contents {
        TypeContents::GENERIC { name: "AtomDomain", .. } => 
            dispatch!(monomorphize_all, [(atom_domain.carrier_type, @primitives)], (atom_domain, size)),
        TypeContents::PLAIN("ExtrinsicDomain") => monomorphize_user_domain(atom_domain, size),
        _ => fallible!(FFI, "Inner domain of VectorDomain must be AtomDomain or ExtrinsicDomain (created through foreign language bindings)")
    }.into()
}

#[bootstrap(
    name = "bitvector_domain",
    arguments(max_weight(rust_type = "Option<u32>", default = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Construct an instance of `BitVectorDomain`.
///
/// # Arguments
/// * `max_weight` - The maximum number of positive bits.
#[no_mangle]
pub extern "C" fn opendp_domains__bitvector_domain(
    max_weight: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let mut bitvector_domain = BitVectorDomain::new();
    if let Some(max_weight) = util::as_ref(max_weight) {
        let max_weight = *try_!(max_weight.downcast_ref::<u32>()) as usize;
        bitvector_domain = bitvector_domain.with_max_weight(max_weight)
    };
    Ok(AnyDomain::new(bitvector_domain)).into()
}

#[bootstrap(name = "map_domain", returns(c_type = "FfiResult<AnyDomain *>"))]
/// Construct an instance of `MapDomain`.
///
/// # Arguments
/// * `key_domain` - domain of keys in the hashmap
/// * `value_domain` - domain of values in the hashmap
#[no_mangle]
pub extern "C" fn opendp_domains__map_domain(
    key_domain: *const AnyDomain,
    value_domain: *const AnyDomain,
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize<K: Hashable, V: Primitive>(
        key_domain: &AnyDomain,
        value_domain: &AnyDomain,
    ) -> Fallible<AnyDomain> {
        let key_domain = key_domain.downcast_ref::<AtomDomain<K>>()?.clone();
        let value_domain = value_domain.downcast_ref::<AtomDomain<V>>()?.clone();
        let map_domain = MapDomain::new(key_domain, value_domain);
        Ok(AnyDomain::new(map_domain))
    }
    fn monomorphize_extrinsic<K: Hashable>(
        key_domain: &AnyDomain,
        value_domain: &AnyDomain,
    ) -> Fallible<AnyDomain> {
        let key_domain = key_domain.downcast_ref::<AtomDomain<K>>()?.clone();
        let value_domain = value_domain.downcast_ref::<ExtrinsicDomain>()?.clone();
        let map_domain = MapDomain::new(key_domain, value_domain);
        Ok(AnyDomain::new(map_domain))
    }
    let key_domain = try_as_ref!(key_domain);
    let value_domain = try_as_ref!(value_domain);

    match (&key_domain.type_.contents, &value_domain.type_.contents) {
        (TypeContents::GENERIC { name: "AtomDomain", .. }, TypeContents::GENERIC { name: "AtomDomain", .. }) => 
            dispatch!(monomorphize, [(key_domain.carrier_type, @hashable), (value_domain.carrier_type, @primitives)], (key_domain, value_domain)),
        (TypeContents::GENERIC { name: "AtomDomain", .. }, TypeContents::PLAIN("ExtrinsicDomain")) => 
            dispatch!(monomorphize_extrinsic, [(key_domain.carrier_type, @hashable)], (key_domain, value_domain)),
        _ => fallible!(FFI, "Value domain of MapDomain must be AtomDomain or ExtrinsicDomain (created through foreign language bindings)"),
    }.into()
}

/// A struct containing the essential metadata shared by extrinsic elements:
/// UserDomain, UserMetric, UserMeasure.
pub struct ExtrinsicElement {
    /// The name of the element, used for display and partial equality
    pub identifier: String,
    /// Data stored inside the element native to a foreign (extrinsic) language
    pub value: ExtrinsicObject,
}

impl Clone for ExtrinsicElement {
    fn clone(&self) -> Self {
        (self.value.count)(self.value.ptr, true);
        Self {
            identifier: self.identifier.clone(),
            value: self.value.clone(),
        }
    }
}

impl ExtrinsicElement {
    pub fn new(identifier: String, value: ExtrinsicObject) -> Self {
        (value.count)(value.ptr, true);
        ExtrinsicElement { value, identifier }
    }
}

impl Debug for ExtrinsicElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier)
    }
}
impl PartialEq for ExtrinsicElement {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}
impl Drop for ExtrinsicElement {
    fn drop(&mut self) {
        (self.value.count)(self.value.ptr, false);
    }
}

/// Rust does not directly manipulate the data behind pointers,
/// the bindings language enforces Send.
unsafe impl Send for ExtrinsicElement {}
/// Rust does not directly manipulate the data behind pointers,
/// the bindings language enforces Sync.
unsafe impl Sync for ExtrinsicElement {}

#[derive(Clone)]
pub struct ExtrinsicDomain {
    pub element: ExtrinsicElement,
    pub member: Function<ExtrinsicObject, bool>,
}

impl std::fmt::Debug for ExtrinsicDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.element)
    }
}

impl PartialEq for ExtrinsicDomain {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

impl Domain for ExtrinsicDomain {
    type Carrier = ExtrinsicObject;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.member.eval(val)
    }
}

#[bootstrap(
    name = "user_domain",
    features("honest-but-curious"),
    arguments(
        identifier(c_type = "char *", rust_type = b"null"),
        member(rust_type = "bool"),
        descriptor(default = b"null", rust_type = "ExtrinsicObject")
    ),
    dependencies("c_member")
)]
/// Construct a new UserDomain.
/// Any two instances of an UserDomain are equal if their string descriptors are equal.
/// Contains a function used to check if any value is a member of the domain.
///
/// # Arguments
/// * `identifier` - A string description of the data domain.
/// * `member` - A function used to test if a value is a member of the data domain.
/// * `descriptor` - Additional constraints on the domain.
///
/// # Why honest-but-curious?
/// The identifier must uniquely identify this domain.
/// If the identifier is not uniquely identifying,
/// then two different domains with the same identifier will chain,
/// which can violate transformation stability.
///
/// In addition, the member function must:
/// 1. be a pure function
/// 2. be sound (only return true if its input is a member of the domain).
#[no_mangle]
pub extern "C" fn opendp_domains__user_domain(
    identifier: *mut c_char,
    member: *const CallbackFn,
    descriptor: *mut ExtrinsicObject,
) -> FfiResult<*mut AnyDomain> {
    let identifier = try_!(to_str(identifier)).to_string();
    let descriptor = try_as_ref!(descriptor).clone();
    let element = ExtrinsicElement::new(identifier, descriptor);
    let member = try_as_ref!(member).clone();
    let member = Function::new_fallible(move |arg: &ExtrinsicObject| -> Fallible<bool> {
        let c_res = (member.callback)(AnyObject::new_raw(arg.clone()));
        Fallible::from(util::into_owned(c_res)?)?.downcast::<bool>()
    });

    Ok(AnyDomain::new(ExtrinsicDomain { element, member })).into()
}

#[bootstrap(
    name = "_extrinsic_domain_descriptor",
    returns(c_type = "FfiResult<ExtrinsicObject *>")
)]
/// Retrieve the descriptor value stored in an extrinsic domain.
///
/// # Arguments
/// * `domain` - The ExtrinsicDomain to extract the descriptor from
#[no_mangle]
pub extern "C" fn opendp_domains___extrinsic_domain_descriptor(
    domain: *mut AnyDomain,
) -> FfiResult<*mut ExtrinsicObject> {
    let domain = try_!(try_as_ref!(domain).downcast_ref::<ExtrinsicDomain>()).clone();
    FfiResult::Ok(util::into_raw(domain.element.value.clone()))
}

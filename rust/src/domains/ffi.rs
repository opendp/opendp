use std::{ffi::c_char, fmt::Debug};

use opendp_derive::bootstrap;

use crate::{
    core::{Domain, FfiResult, Function},
    domains::{AtomDomain, MapDomain, VectorDomain, type_name},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyObject, CallbackFn, Downcast, wrap_func},
        util::{self, ExtrinsicObject, Type, TypeContents, as_ref, c_bool, into_c_char_p, to_str},
    },
    traits::{CheckAtom, Float, Hashable, Integer, Primitive},
};

#[cfg(feature = "polars")]
use crate::domains::{ArrayDomain, CategoricalDomain, DatetimeDomain, EnumDomain};

use super::{BitVectorDomain, Bounds, NaN, OptionDomain};

#[bootstrap(
    name = "_domain_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___domain_free(this: *mut AnyDomain) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "_member",
    arguments(this(hint = "Domain"), val(rust_type = "$domain_carrier_type(this)")),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check membership in a `domain`.
///
/// # Arguments
/// * `this` - The domain to check membership in.
/// * `val` - A potential element of the domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___member(
    this: *mut AnyDomain,
    val: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let this = try_as_ref!(this);
    let val = try_as_ref!(val);
    let status = try_!(this.member(val));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "_domain_equal",
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check whether two domains are equal.
///
/// # Arguments
/// * `left` - Domain to compare.
/// * `right` - Domain to compare.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___domain_equal(
    left: *mut AnyDomain,
    right: *const AnyDomain,
) -> FfiResult<*mut c_bool> {
    let status = try_as_ref!(left) == try_as_ref!(right);
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__domain_carrier_type(
    this: *mut AnyDomain,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    rust_path = "domains/struct.AtomDomain",
    arguments(
        bounds(
            rust_type = "Option<(T, T)>",
            c_type = "AnyObject *",
            default = b"null"
        ),
        nan(rust_type = "Option<bool>", c_type = "AnyObject *", default = b"null")
    ),
    generics(T(example = "$get_first(bounds)")),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "AtomDomain")
)]
/// Construct an instance of `AtomDomain`.
///
/// The domain defaults to unbounded if `bounds` is `None`,
/// If `T` is float, `nan` defaults to `true`.
///
/// # Arguments
/// * `bounds` - Optional bounds of elements in the domain, if the data type is numeric.
/// * `nan` - Whether the domain may contain NaN, if the data type is float.
///
/// # Generics
/// * `T` - The type of the atom.
fn atom_domain<T: CheckAtom>(bounds: Option<Bounds<T>>, nan: Option<NaN<T>>) -> AtomDomain<T> {
    AtomDomain::<T>::new(bounds, nan)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__atom_domain(
    bounds: *const AnyObject,
    nan: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize_float<T: 'static + Float>(
        bounds: *const AnyObject,
        nan: Option<bool>,
    ) -> Fallible<AnyDomain> {
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            let tuple = *bounds.downcast_ref::<(T, T)>()?;
            Some(Bounds::new_closed(tuple)?)
        } else {
            None
        };
        let nan = nan.unwrap_or(true).then_some(NaN::new());
        Ok(AnyDomain::new(atom_domain::<T>(bounds, nan)))
    }
    fn monomorphize_integer<T: 'static + Integer>(
        bounds: *const AnyObject,
        nan: Option<bool>,
    ) -> Fallible<AnyDomain> {
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            let tuple = *bounds.downcast_ref::<(T, T)>()?;
            Some(Bounds::new_closed(tuple)?)
        } else {
            None
        };
        if nan.unwrap_or_default() {
            return fallible!(FFI, "integers cannot represent nullity");
        }
        Ok(AnyDomain::new(atom_domain::<T>(bounds, None)))
    }
    fn monomorphize_simple<T: 'static + CheckAtom>(
        bounds: *const AnyObject,
        nan: Option<bool>,
    ) -> Fallible<AnyDomain> {
        if util::as_ref(bounds).is_some() {
            return fallible!(FFI, "{} cannot be bounded", type_name!(T));
        }
        if nan.unwrap_or_default() {
            return fallible!(FFI, "{} cannot be NaN", type_name!(T));
        }
        Ok(AnyDomain::new(atom_domain::<T>(None, None)))
    }
    let T_ = try_!(Type::try_from(T));
    let nan = if let Some(nan) = as_ref(nan) {
        Some(*try_!(nan.downcast_ref::<bool>()))
    } else {
        None
    };

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
            (bounds, nan)
        )
        .into();
    };

    if let Some(_) = dispatch!(in_set, [(T_, [f32, f64])]) {
        dispatch!(monomorphize_float, [(T_, [f32, f64])], (bounds, nan))
    } else if let Some(_) = dispatch!(
        in_set,
        [(
            T_,
            [
                u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize
            ]
        )]
    ) {
        dispatch!(
            monomorphize_integer,
            [(
                T_,
                [
                    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize
                ]
            )],
            (bounds, nan)
        )
    } else {
        dispatch!(monomorphize_simple, [(T_, [bool, String])], (bounds, nan))
    }
    .into()
}

#[bootstrap(
    arguments(domain(rust_type = b"null")),
    generics(T(suppress)),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve bounds from an AtomDomain<T>
///
/// # Generics
/// * `T` - The type of the atom.
fn _atom_domain_get_bounds_closed<T: CheckAtom>(
    domain: &AtomDomain<T>,
) -> Fallible<Option<(T, T)>> {
    domain.bounds.as_ref().map(|b| b.get_closed()).transpose()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___atom_domain_get_bounds_closed(
    domain: *const AnyDomain,
) -> FfiResult<*mut AnyObject> {
    fn monomorphize<T: 'static + CheckAtom>(domain: &AnyDomain) -> Fallible<AnyObject> {
        let domain = domain.downcast_ref::<AtomDomain<T>>()?;
        Ok(AnyObject::new(
            _atom_domain_get_bounds_closed(domain)?.map(AnyObject::new),
        ))
    }
    let domain = try_as_ref!(domain);
    let T = try_!(domain.type_.get_atom());
    dispatch!(
        monomorphize,
        [(T, @numbers)],
        (domain)
    )
    .into()
}

#[bootstrap(
    arguments(domain(rust_type = b"null")),
    generics(T(suppress)),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve whether members of AtomDomain<T> may be NaN.
///
/// # Generics
/// * `T` - The type of the atom.
fn _atom_domain_nan<T: CheckAtom>(domain: &AtomDomain<T>) -> Fallible<bool> {
    Ok(domain.nan())
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___atom_domain_nan(
    domain: *const AnyDomain,
) -> FfiResult<*mut AnyObject> {
    fn monomorphize<T: 'static + CheckAtom>(domain: &AnyDomain) -> Fallible<AnyObject> {
        let domain = domain.downcast_ref::<AtomDomain<T>>()?;
        _atom_domain_nan(domain).map(AnyObject::new)
    }
    let domain = try_as_ref!(domain);
    let T = try_!(domain.type_.get_atom());
    dispatch!(
        monomorphize,
        [(T, @primitives)],
        (domain)
    )
    .into()
}

#[bootstrap(
    rust_path = "domains/struct.OptionDomain",
    arguments(element_domain(c_type = "AnyDomain *")),
    generics(D(example = "element_domain")),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "OptionDomain")
)]
/// Construct an instance of `OptionDomain`.
///
/// # Generics
/// * `D` - The type of the inner domain.
fn option_domain<D: Domain>(element_domain: D) -> OptionDomain<D> {
    OptionDomain::<D>::new(element_domain)
}

#[unsafe(no_mangle)]
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
    if D == Type::of::<EnumDomain>() {
        let element_domain = try_!(element_domain.downcast_ref::<EnumDomain>()).clone();
        return Ok(AnyDomain::new(option_domain(element_domain))).into();
    }
    #[cfg(feature = "polars")]
    if D == Type::of::<ArrayDomain>() {
        let element_domain = try_!(element_domain.downcast_ref::<ArrayDomain>()).clone();
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

#[bootstrap(name = "_option_domain_get_element_domain")]
/// Retrieve the element domain of the option domain.
///
/// # Arguments
/// * `option_domain` - The option domain from which to retrieve the element domain
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___option_domain_get_element_domain(
    option_domain: *const AnyDomain,
) -> FfiResult<*mut AnyDomain> {
    fn monomorphize_atom<T: 'static + CheckAtom>(option_domain: &AnyDomain) -> Fallible<AnyDomain> {
        let option_domain = option_domain
            .downcast_ref::<OptionDomain<AtomDomain<T>>>()?
            .clone();
        Ok(AnyDomain::new(option_domain.element_domain.clone()))
    }

    let option_domain = try_as_ref!(option_domain);
    let D = option_domain.type_.clone();
    let T = try_!(D.get_atom());

    #[cfg(feature = "polars")]
    if T == Type::of::<CategoricalDomain>() {
        let option_domain =
            try_!(option_domain.downcast_ref::<OptionDomain<CategoricalDomain>>()).clone();
        return Ok(AnyDomain::new(option_domain.element_domain)).into();
    }
    #[cfg(feature = "polars")]
    if T == Type::of::<DatetimeDomain>() {
        let option_domain =
            try_!(option_domain.downcast_ref::<OptionDomain<DatetimeDomain>>()).clone();
        return Ok(AnyDomain::new(option_domain.element_domain)).into();
    }
    #[cfg(feature = "polars")]
    if T == Type::of::<chrono::NaiveDate>() {
        return monomorphize_atom::<chrono::NaiveDate>(option_domain).into();
    }
    #[cfg(feature = "polars")]
    if T == Type::of::<chrono::NaiveTime>() {
        return monomorphize_atom::<chrono::NaiveTime>(option_domain).into();
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
        (option_domain)
    )
    .into()
}

#[bootstrap(
    name = "vector_domain",
    arguments(
        atom_domain(c_type = "AnyDomain *", rust_type = b"null"),
        size(rust_type = "Option<i32>", default = b"null")
    ),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "VectorDomain")
)]
/// Construct an instance of `VectorDomain`.
///
/// # Arguments
/// * `atom_domain` - The inner domain.
#[unsafe(no_mangle)]
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

#[bootstrap(name = "_vector_domain_get_element_domain")]
/// Retrieve the element domain of the vector domain.
///
/// # Arguments
/// * `vector_domain` - The vector domain from which to retrieve the element domain
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___vector_domain_get_element_domain(
    vector_domain: *const AnyDomain,
) -> FfiResult<*mut AnyDomain> {
    let vector_domain = try_as_ref!(vector_domain);
    let D = vector_domain.type_.clone();
    let T = try_!(D.get_atom());

    if T == Type::of::<ExtrinsicDomain>() {
        let vector_domain =
            try_!(vector_domain.downcast_ref::<VectorDomain<ExtrinsicDomain>>()).clone();
        return Ok(AnyDomain::new(vector_domain.element_domain)).into();
    }

    fn monomorphize_atom<T: 'static + CheckAtom>(vector_domain: &AnyDomain) -> Fallible<AnyDomain> {
        let option_domain = vector_domain
            .downcast_ref::<VectorDomain<AtomDomain<T>>>()?
            .clone();
        Ok(AnyDomain::new(option_domain.element_domain.clone()))
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
        (vector_domain)
    )
    .into()
}

#[bootstrap(name = "_vector_domain_get_size")]
/// Retrieve the size of vectors in the vector domain.
///
/// # Arguments
/// * `vector_domain` - The vector domain from which to retrieve the size
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___vector_domain_get_size(
    vector_domain: *const AnyDomain,
) -> FfiResult<*mut AnyObject> {
    let vector_domain = try_as_ref!(vector_domain);
    let D = vector_domain.type_.clone();
    let T = try_!(D.get_atom());

    if T == Type::of::<ExtrinsicDomain>() {
        let vector_domain =
            try_!(vector_domain.downcast_ref::<VectorDomain<ExtrinsicDomain>>()).clone();
        return Ok(AnyObject::new(vector_domain.size.map(AnyObject::new))).into();
    }

    fn monomorphize_atom<T: 'static + CheckAtom>(vector_domain: &AnyDomain) -> Fallible<AnyObject> {
        let option_domain = vector_domain
            .downcast_ref::<VectorDomain<AtomDomain<T>>>()?
            .clone();
        Ok(AnyObject::new(option_domain.size.map(AnyObject::new)))
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
        (vector_domain)
    )
    .into()
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[derive(Clone)]
pub struct ExtrinsicElement {
    /// The name of the element, used for display and partial equality
    pub identifier: String,
    /// Data stored inside the element native to a foreign (extrinsic) language
    pub value: ExtrinsicObject,
}

impl ExtrinsicElement {
    pub fn new(identifier: String, value: ExtrinsicObject) -> Self {
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
    )
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
///
/// 1. be a pure function
/// 2. be sound (only return true if its input is a member of the domain).
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__user_domain(
    identifier: *mut c_char,
    member: *const CallbackFn,
    descriptor: *mut ExtrinsicObject,
) -> FfiResult<*mut AnyDomain> {
    let identifier = try_!(to_str(identifier)).to_string();
    let descriptor = try_as_ref!(descriptor).clone();
    let element = ExtrinsicElement::new(identifier, descriptor);

    let member_closure = wrap_func(try_as_ref!(member).clone());
    let member_function = Function::new_fallible(move |arg: &ExtrinsicObject| -> Fallible<bool> {
        member_closure(&AnyObject::new(arg.clone()))?.downcast::<bool>()
    });

    Ok(AnyDomain::new(ExtrinsicDomain {
        element,
        member: member_function,
    }))
    .into()
}

#[bootstrap(
    name = "_extrinsic_domain_descriptor",
    returns(c_type = "FfiResult<ExtrinsicObject *>")
)]
/// Retrieve the descriptor value stored in an extrinsic domain.
///
/// # Arguments
/// * `domain` - The ExtrinsicDomain to extract the descriptor from
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___extrinsic_domain_descriptor(
    domain: *mut AnyDomain,
) -> FfiResult<*mut ExtrinsicObject> {
    let domain = try_!(try_as_ref!(domain).downcast_ref::<ExtrinsicDomain>()).clone();
    FfiResult::Ok(util::into_raw(domain.element.value.clone()))
}

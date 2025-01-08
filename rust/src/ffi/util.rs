use std::any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ffi::{c_void, CString};
use std::ffi::{CStr, IntoStringError, NulError};
use std::os::raw::c_char;
use std::str::Utf8Error;

use crate::core::Function;
use crate::domains::ffi::ExtrinsicDomain;
use crate::domains::{AtomDomain, BitVector, OptionDomain, VectorDomain};
use crate::error::*;
use crate::ffi::any::{AnyObject, AnyQueryable};
use crate::measures::ffi::ExtrinsicDivergence;
use crate::measures::{
    Approximate, MaxDivergence, PrivacyProfile, RenyiDivergence, SmoothedMaxDivergence,
    ZeroConcentratedDivergence,
};
use crate::metrics::{
    AbsoluteDistance, ChangeOneDistance, DiscreteDistance, HammingDistance, InsertDeleteDistance,
    L1Distance, L2Distance, SymmetricDistance,
};

#[cfg(feature = "polars")]
use crate::polars::{OnceFrame, OnceFrameAnswer, OnceFrameQuery};

use crate::transformations::DataFrameDomain;
use crate::{err, fallible};

use super::any::{AnyDomain, AnyMeasurement, AnyTransformation};

// If untrusted is not enabled, then these structs don't exist.
#[cfg(feature = "untrusted")]
use crate::transformations::{Pairwise, Sequential};
#[cfg(not(feature = "untrusted"))]
use std::marker::PhantomData;
#[cfg(not(feature = "untrusted"))]
pub struct Sequential<T>(PhantomData<T>);
#[cfg(not(feature = "untrusted"))]
pub struct Pairwise<T>(PhantomData<T>);

// If polars is not enabled, then these structs don't exist.
#[cfg(feature = "polars")]
use crate::domains::{
    CategoricalDomain, DatetimeDomain, ExprDomain, ExprPlan, LazyFrameDomain, SeriesDomain,
};
#[cfg(feature = "polars")]
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
#[cfg(feature = "polars")]
use polars::prelude::{DataFrame, DslPlan, Expr, LazyFrame, Series};

pub type RefCountFn = extern "C" fn(*const c_void, bool) -> bool;

#[repr(C)]
pub struct ExtrinsicObject {
    pub(crate) ptr: *const c_void,
    pub(crate) count: RefCountFn,
}

impl Clone for ExtrinsicObject {
    fn clone(&self) -> Self {
        (self.count)(self.ptr, true);
        Self {
            ptr: self.ptr.clone(),
            count: self.count.clone(),
        }
    }
}

impl Drop for ExtrinsicObject {
    fn drop(&mut self) {
        (self.count)(self.ptr, false);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeContents {
    PLAIN(&'static str),
    TUPLE(Vec<TypeId>),
    ARRAY {
        element_id: TypeId,
        len: usize,
    },
    SLICE(TypeId),
    GENERIC {
        name: &'static str,
        args: Vec<TypeId>,
    },
    VEC(TypeId), // This is a convenience specialization of GENERIC, used until we switch to slices.
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub id: TypeId,
    pub descriptor: String,
    pub contents: TypeContents,
}

impl Type {
    pub fn new<S: AsRef<str>>(id: TypeId, descriptor: S, contents: TypeContents) -> Self {
        Self {
            id,
            descriptor: descriptor.as_ref().to_string(),
            contents,
        }
    }

    pub fn of<T: 'static + ?Sized>() -> Self {
        let id = TypeId::of::<T>();
        // First try to find a registered type (which will have the nice descriptor). In lieu of that, create one on the fly.
        let type_ = TYPE_ID_TO_TYPE.get(&id);
        type_.map_or_else(Self::of_unregistered::<T>, Clone::clone)
    }

    fn of_unregistered<T: 'static + ?Sized>() -> Self {
        let descriptor = any::type_name::<T>();
        Self::new(
            TypeId::of::<T>(),
            descriptor,
            TypeContents::PLAIN(descriptor),
        )
    }

    pub fn of_id(id: &TypeId) -> Fallible<Self> {
        TYPE_ID_TO_TYPE
            .get(id)
            .cloned()
            .ok_or_else(|| err!(TypeParse, "unrecognized type id"))
    }
}

impl Type {
    pub fn get_atom(&self) -> Fallible<Type> {
        match &self.contents {
            TypeContents::PLAIN(_) => Ok(self.clone()),
            TypeContents::GENERIC { args, .. } => {
                if args.len() != 1 {
                    return fallible!(
                        TypeParse,
                        "Failed to extract atom type: expected one argument, got {:?} arguments",
                        args.len()
                    );
                }
                Type::of_id(&args[0])?.get_atom()
            }
            _ => fallible!(TypeParse, "Failed to extract atom type: not a generic"),
        }
    }
}
impl ToString for Type {
    fn to_string(&self) -> String {
        let get_id_str = |type_id: &TypeId| {
            Type::of_id(type_id)
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|_| format!("{:?} {:?}", type_id, TypeId::of::<f64>()))
        };

        match &self.contents {
            TypeContents::PLAIN(v) => v.to_string(),
            TypeContents::TUPLE(args) => format!(
                "({})",
                args.iter().map(get_id_str).collect::<Vec<_>>().join(", ")
            ),
            TypeContents::ARRAY { element_id, len } => {
                format!("[{}; {}]", get_id_str(element_id), len)
            }
            TypeContents::SLICE(type_id) => format!("&[{}]", get_id_str(type_id)),
            TypeContents::GENERIC { name, args } => format!(
                "{}<{}>",
                name,
                args.iter().map(get_id_str).collect::<Vec<_>>().join(", ")
            ),
            TypeContents::VEC(v) => format!("Vec<{}>", get_id_str(v)),
        }
    }
}

// Convert `[A B C] i8` -> `A<B<C<i8>>`
// 1. Reverse the array:
// `[A B C] [] i8` -> `[B C] [A] i8` -> `[C] [B A] i8` -> `[] [C B A] i8` ->
// 2. Recursively peel the first element off the reversed array:
// `[] [B A] C<i8>` -> `[] [A] B<C<i8>>` ->
// 3. The final step drops the leading arrays:
// `A<B<C<i8>>`
macro_rules! nest {
    ([$($all:tt)*] $arg:ty) => (nest!(@[$($all)*] [] $arg));

    // move elements in the left array to the right array, in reversed order
    (@[$first:ident $($rest:tt)*] [$($reversed:tt)*] $arg:ty) =>
        (nest!(@[$($rest)*] [$first $($reversed)*] $arg));
    // left array is empty once reversed. Recursively peel off front ident to construct type
    (@[] [$first:ident $($name:ident)+] $arg:ty) => (nest!(@[] [$($name)+] $first<$arg>));
    // base case
    (@[] [$first:ident] $arg:ty) => ($first<$arg>);

    // make TypeContents
    (@contents [$first:ident $($rest:ident)*] $arg:ty) => (TypeContents::GENERIC {
        name: stringify!($first),
        args: vec![TypeId::of::<nest!([$($rest)*] $arg)>()]
    });
}

macro_rules! replace {
    ($from:ident, $to:literal) => {
        $to
    };
}

/// Builds a [`Type`] from a compact invocation, choosing an appropriate [`TypeContents`].
/// * `t!(Foo)` => `TypeContents::PLAIN`
/// * `t!((Foo, Bar))` => `TypeContents::TUPLE`
/// * `t!([Foo; 10])` => `TypeContents::ARRAY`
/// * `t!([Foo])` => `TypeContents::SLICE`
/// * `t!(Foo<Bar>)` => `TypeContents::GENERIC`
/// * `t!(Vec<primitive>)` => `TypeContents::VEC`
macro_rules! t {
    (Vec<$arg:ty>) => {
        Type::new(
            TypeId::of::<Vec<$arg>>(),
            concat!("Vec<", stringify!($arg), ">"),
            TypeContents::VEC(TypeId::of::<$arg>())
        )
    };
    ([$($name:ident)+], $arg:ty) => {
        Type::new(
            TypeId::of::<nest!([$($name)+] $arg)>(),
            concat!($(stringify!($name), "<",)+ stringify!($arg), $(replace!($name, ">")),+),
            nest!(@contents [$($name)+] $arg)
        )
    };
    ($name:ident<$($args:ty),+>) => {
        Type::new(
            TypeId::of::<$name<$($args),+>>(),
            concat!(stringify!($name), "<", stringify!($($args),+), ">"),
            TypeContents::GENERIC { name: stringify!($name), args: vec![$(TypeId::of::<$args>()),+]}
        )
    };
    ([$element:ty]) => {
        Type::new(
            TypeId::of::<[$element]>(),
            concat!("[", stringify!($element), "]"),
            TypeContents::SLICE(TypeId::of::<$element>())
        )
    };
    ([$element:ty; $len:expr]) => {
        Type::new(
            TypeId::of::<[$element;$len]>(),
            concat!("[", stringify!($element), "; ", stringify!($len), "]"),
            TypeContents::ARRAY { element_id: TypeId::of::<$element>(), len: $len }
        )
    };
    (($($elements:ty),+)) => {
        Type::new(
            TypeId::of::<($($elements),+)>(),
            concat!("(", stringify!($($elements),+), ")"),
            TypeContents::TUPLE(vec![$(TypeId::of::<$elements>()),+])
        )
    };
    ($name:ty) => {
        Type::new(TypeId::of::<$name>(), stringify!($name), TypeContents::PLAIN(stringify!($name)))
    };
}
/// Builds a vec of [`Type`] from a compact invocation, dispatching to the appropriate flavor of [`t!`].
macro_rules! type_vec {
    // 2-arg generic: base case   --   out, a, b, init_b
    (@$name:ident [$(($a1:ty, $a2:ty))*] [] $b:tt $init_b:tt) =>
        (vec![$(t!($name<$a1, $a2>)),*]);
    // 2-arg generic: when b empty, strip off an "a" and refill b from init_b
    (@$name:ident $out:tt [$a:tt $(,$at:tt)*] [] $init_b:tt) =>
        (type_vec!{@$name $out [$($at),*] $init_b $init_b});
    // 2-arg generic: strip off a "b" and add a pair to $out that consists of the first "a" and first "b"
    (@$name:ident [$($out:tt)*] [$a:tt $(,$at:tt)*] [$b:tt $(,$bt:tt)*] $init_b:tt) =>
        (type_vec!{@$name [$($out)* ($a, $b)] [$a $(,$at)*] [$($bt),*] $init_b});
    // 2-arg generic: friendly public interface
    ($name:ident, <$($arg1:tt),*>, <$($arg2:tt),*>) =>
        (type_vec!{@$name [] [$($arg1),*] [$($arg2),*] [$($arg2),*]});

    ($name:ident, <$($arg:ty),*>) => { vec![$(t!($name<$arg>)),*] };
    ($path:tt, <$($arg:ty),*>) => { vec![$(t!($path, $arg)),*] };
    ([$($elements:ty),*]) => { vec![$(t!([$elements])),*] };
    ([$($elements:ty),*]; $len:expr) => { vec![$(t!([$elements; $len])),*] };
    (($($elements:ty),*)) => { vec![$(t!(($elements,$elements))),*] };
    ($($names:ty),*) => { vec![$(t!($names)),*] };
}

pub type AnyMeasurementPtr = *const AnyMeasurement;
pub type AnyTransformationPtr = *const AnyTransformation;
pub type AnyDomainPtr = *const AnyDomain;

lazy_static! {
    /// The set of registered types. We don't need everything here, just the ones that will be looked up by descriptor
    /// (i.e., the ones that appear in FFI function generic args).
    static ref TYPES: Vec<Type> = {
        #[cfg(feature = "polars")]
        let polars_types = vec![
            type_vec![DataFrame, LazyFrame, DslPlan, Series, Expr, ExprPlan, OnceFrame, OnceFrameQuery, OnceFrameAnswer],
            type_vec![ExprDomain, LazyFrameDomain, SeriesDomain],

            type_vec![NaiveDate, NaiveTime, NaiveDateTime],
            type_vec![AtomDomain, <NaiveDate, NaiveTime>],
            type_vec![[OptionDomain AtomDomain], <NaiveDate, NaiveTime>],

            type_vec![CategoricalDomain, DatetimeDomain],
            type_vec![OptionDomain, <CategoricalDomain, DatetimeDomain>],

            vec![t!((DslPlan, Expr))],
            type_vec![Vec, <(DslPlan, Expr), SeriesDomain, Expr>],
        ];
        #[cfg(not(feature = "polars"))]
        let polars_types = Vec::new();

        let types: Vec<Type> = vec![
            // data types
            vec![t!(())],
            type_vec![bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String, AnyObject],
            type_vec![(bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String, AnyObject)],
            type_vec![[bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String, AnyObject]; 1], // Arrays are here just for unit tests, unlikely we'll use them.
            type_vec![[bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String, AnyObject]],
            type_vec![Vec, <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String, AnyObject, ExtrinsicObject>],
            type_vec![HashMap, <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, String>, <bool, char, u8, u16, u32, i16, i32, i64, i128, f32, f64, usize, String, AnyObject, ExtrinsicObject>],
            type_vec![ExtrinsicObject, BitVector],
            // OptionDomain<AtomDomain<_>>::Carrier
            type_vec![[Vec Option], <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String, AnyObject>],
            type_vec![Vec, <(f32, f32), (f64, f64), BitVector>],
            // these are used by PartitionDistance. The latter two values are the dtype of the inner metric
            vec![t!((u32, u32, u32)), t!((u32, u64, u64)), t!((u32, i32, i32)), t!((u32, i64, i64))],
            vec![t!((u32, usize, usize)), t!((u32, f32, f32)), t!((u32, f64, f64))],
            vec![t!(Option<(f64, AnyObject)>), t!(Option<(f64, ExtrinsicObject)>)],
            vec![t!((f64, AnyObject)), t!((f64, ExtrinsicObject))],
            vec![t!(Function<f64, f64>)],

            type_vec![AnyMeasurementPtr, AnyTransformationPtr, AnyQueryable, AnyMeasurement],
            type_vec![Vec, <AnyMeasurementPtr, AnyTransformationPtr>],

            // sum algorithms
            type_vec![Sequential, <f32, f64>],
            type_vec![Pairwise, <f32, f64>],

            // domains
            type_vec![AtomDomain, <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String>],
            type_vec![[OptionDomain AtomDomain], <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String>],
            type_vec![[VectorDomain AtomDomain], <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String>],
            type_vec![[VectorDomain OptionDomain AtomDomain], <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, String>],
            type_vec![ExtrinsicDomain],
            type_vec![DataFrameDomain, <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, String>],

            // metrics
            type_vec![ChangeOneDistance, SymmetricDistance, InsertDeleteDistance, HammingDistance],
            type_vec![DiscreteDistance],
            type_vec![AbsoluteDistance, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],
            type_vec![L1Distance, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],
            type_vec![L2Distance, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],

            // measures
            type_vec![MaxDivergence, SmoothedMaxDivergence, ZeroConcentratedDivergence, RenyiDivergence, ExtrinsicDivergence],
            type_vec![Approximate, <MaxDivergence, SmoothedMaxDivergence, ZeroConcentratedDivergence, RenyiDivergence, ExtrinsicDivergence>],

            // measure distances
            type_vec![PrivacyProfile],
            vec![t!((PrivacyProfile, f64))]
        ].into_iter().chain(polars_types).flatten().collect();
        let descriptors: HashSet<_> = types.iter().map(|e| &e.descriptor).collect();
        assert_eq!(descriptors.len(), types.len(), "detected duplicate TYPES");
        types
    };
}
lazy_static! {
    static ref TYPE_ID_TO_TYPE: HashMap<TypeId, Type> =
        TYPES.iter().map(|e| (e.id, e.clone())).collect();
}
lazy_static! {
    static ref DESCRIPTOR_TO_TYPE: HashMap<&'static str, Type> = {
        TYPES
            .iter()
            .map(|e| (e.descriptor.as_str(), e.clone()))
            .collect()
    };
}

impl TryFrom<&str> for Type {
    type Error = Error;
    fn try_from(value: &str) -> Fallible<Self> {
        let type_ = DESCRIPTOR_TO_TYPE.get(value);
        type_
            .cloned()
            .ok_or_else(|| err!(TypeParse, "failed to parse type: {}", value))
    }
}

impl TryFrom<*const c_char> for Type {
    type Error = Error;
    fn try_from(value: *const c_char) -> Fallible<Self> {
        to_str(value).and_then(Type::try_from)
    }
}

pub fn into_raw<T>(o: T) -> *mut T {
    Box::into_raw(Box::<T>::new(o))
}

pub fn into_owned<T>(p: *mut T) -> Fallible<T> {
    (!p.is_null())
        .then(|| *unsafe { Box::<T>::from_raw(p) })
        .ok_or_else(|| err!(FFI, "attempted to consume a null pointer"))
}

pub fn as_ref<'a, T>(p: *const T) -> Option<&'a T> {
    (!p.is_null()).then(|| unsafe { &*p })
}

pub fn as_mut_ref<'a, T>(p: *mut T) -> Option<&'a mut T> {
    (!p.is_null()).then(|| unsafe { &mut *p })
}

pub fn into_c_char_p(s: String) -> Fallible<*mut c_char> {
    CString::new(s)
        .map(CString::into_raw)
        .map_err(|e: NulError| {
            err!(
                FFI,
                "Nul byte detected when reading C string at position {:?}",
                e.nul_position()
            )
        })
}

pub fn into_string(p: *mut c_char) -> Fallible<String> {
    if p.is_null() {
        return fallible!(FFI, "Attempted to load a string from a null pointer");
    }
    let s = unsafe { CString::from_raw(p) };
    s.into_string()
        .map_err(|e: IntoStringError| err!(FFI, "{:?} ", e.utf8_error()))
}

pub fn to_str<'a>(p: *const c_char) -> Fallible<&'a str> {
    if p.is_null() {
        return fallible!(FFI, "Attempted to load a string from a null pointer");
    }
    let s = unsafe { CStr::from_ptr(p) };
    s.to_str().map_err(|e: Utf8Error| err!(FFI, "{:?}", e))
}

pub fn to_option_str<'a>(p: *const c_char) -> Fallible<Option<&'a str>> {
    if p.is_null() {
        Ok(None)
    } else {
        Some(to_str(p)).transpose()
    }
}

#[allow(non_camel_case_types)]
pub type c_bool = u8; // PLATFORM DEPENDENT!!!

pub fn to_bool(b: c_bool) -> bool {
    b != 0
}

pub fn from_bool(b: bool) -> c_bool {
    if b {
        1
    } else {
        0
    }
}

#[cfg(test)]
pub trait ToCharP {
    fn to_char_p(&self) -> *mut c_char;
}
#[cfg(test)]
impl<S: ToString> ToCharP for S {
    fn to_char_p(&self) -> *mut c_char {
        crate::ffi::util::into_c_char_p(self.to_string()).unwrap_test()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::metrics::L1Distance;

    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_type_of() {
        let i32_t = TypeId::of::<i32>();
        assert_eq!(Type::of::<()>(), Type::new(TypeId::of::<()>(), "()", TypeContents::PLAIN("()")));
        assert_eq!(Type::of::<i32>(), Type::new(i32_t, "i32", TypeContents::PLAIN("i32")));
        assert_eq!(Type::of::<String>(), Type::new(TypeId::of::<String>(), "String", TypeContents::PLAIN("String")));
        assert_eq!(Type::of::<(i32, i32)>(), Type::new(TypeId::of::<(i32, i32)>(), "(i32, i32)", TypeContents::TUPLE(vec![i32_t, i32_t])));
        assert_eq!(Type::of::<[i32; 1]>(), Type::new(TypeId::of::<[i32; 1]>(), "[i32; 1]", TypeContents::ARRAY { element_id: i32_t, len: 1 }));
        assert_eq!(Type::of::<[i32]>(), Type::new(TypeId::of::<[i32]>(), "[i32]", TypeContents::SLICE(i32_t)));
        assert_eq!(Type::of::<L1Distance<i32>>(), Type::new(TypeId::of::<L1Distance<i32>>(), "L1Distance<i32>", TypeContents::GENERIC { name: "L1Distance", args: vec![i32_t] }));
        assert_eq!(Type::of::<Vec<i32>>(), Type::new(TypeId::of::<Vec<i32>>(), "Vec<i32>", TypeContents::VEC(i32_t)));
    }

    #[test]
    #[rustfmt::skip]
    fn test_type_try_from() -> Fallible<()> {
        let i32_t = TypeId::of::<i32>();
        assert_eq!(TryInto::<Type>::try_into("()")?, Type::new(TypeId::of::<()>(), "()", TypeContents::PLAIN("()")));
        assert_eq!(TryInto::<Type>::try_into("i32")?, Type::new(i32_t, "i32", TypeContents::PLAIN("i32")));
        assert_eq!(TryInto::<Type>::try_into("String")?, Type::new(TypeId::of::<String>(), "String", TypeContents::PLAIN("String")));
        assert_eq!(TryInto::<Type>::try_into("(i32, i32)")?, Type::new(TypeId::of::<(i32, i32)>(), "(i32, i32)", TypeContents::TUPLE(vec![i32_t, i32_t])));
        assert_eq!(TryInto::<Type>::try_into("[i32; 1]")?, Type::new(TypeId::of::<[i32; 1]>(), "[i32; 1]", TypeContents::ARRAY { element_id: i32_t, len: 1 }));
        assert_eq!(TryInto::<Type>::try_into("[i32]")?, Type::new(TypeId::of::<[i32]>(), "[i32]", TypeContents::SLICE(i32_t)));
        assert_eq!(TryInto::<Type>::try_into("L1Distance<i32>")?, Type::new(TypeId::of::<L1Distance<i32>>(), "L1Distance<i32>", TypeContents::GENERIC { name: "L1Distance", args: vec![i32_t] }));
        assert_eq!(TryInto::<Type>::try_into("Vec<i32>")?, Type::new(TypeId::of::<Vec<i32>>(), "Vec<i32>", TypeContents::VEC(i32_t)));
        Ok(())
    }
}

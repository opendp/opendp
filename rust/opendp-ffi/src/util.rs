use std::any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ffi::{CStr, IntoStringError, NulError};
use std::ffi::CString;
use std::os::raw::c_char;
use std::str::Utf8Error;

use opendp::{err, fallible};
use opendp::dist::{HammingDistance, L1Distance, L2Distance, SymmetricDistance, AbsoluteDistance};
use opendp::error::*;
use crate::any::AnyObject;

#[derive(Debug, PartialEq, Clone)]
pub enum TypeContents {
    PLAIN(&'static str),
    TUPLE(Vec<TypeId>),
    ARRAY { element_id: TypeId, len: usize },
    SLICE(TypeId),
    GENERIC { name: &'static str, args: Vec<TypeId> },
    VEC(TypeId),  // This is a convenience specialization of GENERIC, used until we switch to slices.
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub id: TypeId,
    pub descriptor: &'static str,
    pub contents: TypeContents,
}

impl Type {
    pub fn new(id: TypeId, descriptor: &'static str, contents: TypeContents) -> Self {
        Self { id, descriptor, contents }
    }

    pub fn of<T: 'static + ?Sized>() -> Self {
        let id = TypeId::of::<T>();
        // First try to find a registered type (which will have the nice descriptor). In lieu of that, create one on the fly.
        let type_ = TYPE_ID_TO_TYPE.get(&id);
        type_.map_or_else(Self::of_unregistered::<T>, Clone::clone)
    }

    fn of_unregistered<T: 'static + ?Sized>() -> Self {
        let descriptor = any::type_name::<T>();
        Self::new(TypeId::of::<T>(), descriptor, TypeContents::PLAIN(descriptor))
    }

    pub fn of_id(id: &TypeId) -> Fallible<Self> {
        TYPE_ID_TO_TYPE.get(id).cloned().ok_or_else(|| err!(TypeParse))
    }

    // Hacky special entry point for composition.
    pub fn new_box_pair(type0: &Type, type1: &Type) -> Self {
        #[allow(clippy::unnecessary_wraps)]
        fn monomorphize<T0: 'static, T1: 'static>(type0: &Type, type1: &Type) -> Fallible<Type> {
            let id = TypeId::of::<(Box<T0>, Box<T1>)>();
            let descriptor = format!("(Box<{}>, Box<{}>)", type0.descriptor, type1.descriptor);
            // Hacky way to get &'static str from String.
            let descriptor = Box::leak(descriptor.into_boxed_str());
            let contents = TypeContents::TUPLE(vec![TypeId::of::<Box<T0>>(), TypeId::of::<Box<T1>>()]);
            Ok(Type::new(id, descriptor, contents))
        }
        dispatch!(
            monomorphize,
            // FIXME: The Box<f64> entries are here for demo use.
            [
                (type0, [bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, (Box<f64>, Box<f64>)]),
                (type1, [bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, (Box<f64>, Box<f64>)])
            ],
            (type0, type1)
        ).unwrap()
    }
}

pub enum MetricClass { Dataset, Sensitivity }

impl Type {
    pub fn get_sensitivity_distance(&self) -> Fallible<Type> {
        if let TypeContents::GENERIC {args, name} = &self.contents {
            if !vec!["L1Distance", "L2Distance", "AbsoluteDistance"].contains(name) {
                return fallible!(TypeParse, "Expected a sensitivity type name, received {:?}", name)
            }
            if args.len() != 1 {
                return fallible!(TypeParse, "Sensitivity must have one generic argument")
            }
            Type::of_id(&args[0])
        } else {
            fallible!(TypeParse, "Expected a sensitivity type that is generic with respect to one distance type- for example, AbsoluteDistance<u32>")
        }
    }
    pub fn get_metric_class(&self) -> Fallible<MetricClass> {
        if self == &Type::of::<HammingDistance>() || self == &Type::of::<SymmetricDistance>() {
            Ok(MetricClass::Dataset)
        } else if let TypeContents::GENERIC { name, .. } = &self.contents {
            if vec!["L1Distance", "L2Distance", "AbsoluteDistance"].contains(name) {
                Ok(MetricClass::Sensitivity)
            } else {
                return fallible!(TypeParse, "Expected a metric type name, received {:?}", name)
            }
        } else {
            fallible!(TypeParse, "Expected a metric type.")
        }
    }
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
    ($name:ident, <$($args:ty),*>) => { vec![$(t!($name<$args>)),*] };
    ([$($elements:ty),*]) => { vec![$(t!([$elements])),*] };
    ([$($elements:ty),*]; $len:expr) => { vec![$(t!([$elements; $len])),*] };
    (($($elements:ty),*)) => { vec![$(t!(($elements,$elements))),*] };
    ($($names:ty),*) => { vec![$(t!($names)),*] };
}

lazy_static! {
    /// The set of registered types. We don't need everything here, just the ones that will be looked up by descriptor
    /// (i.e., the ones that appear in FFI function generic args).
    static ref TYPES: Vec<Type> = {
        let types: Vec<Type> = vec![
            vec![t!(())],
            type_vec![bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, AnyObject],
            type_vec![(bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, AnyObject)],
            type_vec![[bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, AnyObject]; 1], // Arrays are here just for unit tests, unlikely we'll use them.
            type_vec![[bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, AnyObject]],
            type_vec![HammingDistance, SymmetricDistance],
            type_vec![AbsoluteDistance, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],
            type_vec![L1Distance, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],
            type_vec![L2Distance, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],
            type_vec![Vec, <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, AnyObject>],
        ].into_iter().flatten().collect();
        let descriptors: HashSet<_> = types.iter().map(|e| e.descriptor).collect();
        assert_eq!(descriptors.len(), types.len());
        types
    };
}
lazy_static! {
    static ref TYPE_ID_TO_TYPE: HashMap<TypeId, Type> = {
        TYPES.iter().map(|e| (e.id, e.clone())).collect()
    };
}
lazy_static! {
    static ref DESCRIPTOR_TO_TYPE: HashMap<&'static str, Type> = {
        TYPES.iter().map(|e| (e.descriptor, e.clone())).collect()
    };
}

impl TryFrom<&str> for Type {
    type Error = Error;
    fn try_from(value: &str) -> Fallible<Self> {
        let type_ = DESCRIPTOR_TO_TYPE.get(value);
        type_.cloned().ok_or_else(|| err!(TypeParse, "failed to parse type: `{}`", value))
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
    (!p.is_null()).then(|| *unsafe { Box::<T>::from_raw(p) })
        .ok_or_else(|| err!(FFI, "attempted to consume a null pointer"))
}

pub fn as_ref<'a, T>(p: *const T) -> Option<&'a T> {
    (!p.is_null()).then(|| unsafe { &*p })
}

pub fn into_c_char_p(s: String) -> Fallible<*mut c_char> {
    CString::new(s)
        .map(CString::into_raw)
        .map_err(|e: NulError|
            err!(FFI, "Nul byte detected when reading C string at position {:?}", e.nul_position()))
}

pub fn into_string(p: *mut c_char) -> Fallible<String> {
    if p.is_null() {
        return fallible!(FFI, "Attempted to load a string from a null pointer");
    }
    let s = unsafe { CString::from_raw(p) };
    s.into_string().map_err(|e: IntoStringError| err!(FFI, "{:?} ", e.utf8_error()))
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
pub type c_bool = u8;  // PLATFORM DEPENDENT!!!

pub fn to_bool(b: c_bool) -> bool {
    b != 0
}

pub fn from_bool(b: bool) -> c_bool {
    if b {1} else {0}
}


#[cfg(test)]
pub trait ToCharP {
    fn to_char_p(&self) -> *mut c_char;
}
#[cfg(test)]
impl<S: ToString> ToCharP for S {
    fn to_char_p(&self) -> *mut c_char {
        crate::util::into_c_char_p(self.to_string()).unwrap_test()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use opendp::dist::L1Distance;

    use super::*;

    #[test]
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

use std::any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, IntoStringError, NulError};
use std::ffi::CString;
use std::os::raw::c_char;

use opendp::dist::{SymmetricDistance, HammingDistance, L1Sensitivity, L2Sensitivity};
use opendp::error::*;
use opendp::{err, fallible};
use std::str::Utf8Error;

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

    pub fn of_id(id: TypeId) -> Fallible<Self> {
        TYPE_ID_TO_TYPE.get(&id).cloned().ok_or_else(|| err!(TypeParse))
    }

    // Hacky special entry point for composition.
    pub fn new_box_pair(type0: &Type, type1: &Type) -> Self {
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
            type_vec![bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String],
            type_vec![(bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String)],
            type_vec![[bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String]; 1], // Arrays are here just for unit tests, unlikely we'll use them.
            type_vec![[bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String]],
            type_vec![HammingDistance, SymmetricDistance],
            type_vec![L1Sensitivity, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],
            type_vec![L2Sensitivity, <u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64>],
            type_vec![Vec, <bool, char, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String>],
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
        type_.map(|e| e.clone()).ok_or_else(|| err!(TypeParse, "failed to parse type: {:?}", value))
    }
}

#[derive(Debug, PartialEq)]
pub struct TypeArgs(pub(crate) Vec<Type>);

impl TypeArgs {
    pub fn parse(descriptor: *const c_char, count: usize) -> Fallible<TypeArgs> {
        let descriptor = to_str(descriptor)?;
        let type_args: TypeArgs = descriptor.try_into()?;
        if type_args.0.len() != count {
            return fallible!(TypeParse, "expected {:?} arguments, received {:?} arguments", count, type_args.0.len())
        }
        Ok(type_args)
    }
    // pub fn new(args: Vec<Type>) -> TypeArgs {
    //     TypeArgs(args)
    // }
    // pub fn descriptor(&self) -> String {
    //     let arg_descriptors: Vec<_> = self.0.iter().map(|e| e.descriptor).collect();
    //     format!("<{}>", arg_descriptors.join(", "))
    // }
}

impl TryFrom<&str> for TypeArgs {
    type Error = Error;
    fn try_from(value: &str) -> Fallible<Self> {
        if value.starts_with("<") && value.ends_with(">") {
            let value = &value[1..value.len()-1];
            let mut types = Vec::new();
            let mut token_buffer = Vec::new();
            let mut is_parenthesized = false;
            for token in value.split(",") {
                token_buffer.push(token.trim());
                // loose and simple approximation assuming no nested tuples
                if token.contains("(") {
                    is_parenthesized = true;
                }
                if token.contains(")") {
                    is_parenthesized = false;
                }
                if !is_parenthesized {
                    let descriptor: String = token_buffer.join(", ");
                    types.push(descriptor.as_str().try_into()?);
                    token_buffer.clear();
                }
            }
            Ok(TypeArgs(types))
        } else {
            fallible!(TypeParse, "failed to parse type: type ascription must start with '<' and end with '>'")
        }
    }
}


pub fn into_raw<T>(o: T) -> *mut T {
    Box::into_raw(Box::<T>::new(o))
}

pub fn into_box<T, U>(o: T) -> Box<U> {
    let p = into_raw(o) as *mut U;
    unsafe { Box::from_raw(p) }
}

pub fn into_owned<T>(p: *mut T) -> Option<T> {
    (!p.is_null()).then(|| *unsafe { Box::<T>::from_raw(p) })
}

pub fn as_ref<'a, T>(p: *const T) -> Option<&'a T> {
    (!p.is_null()).then(|| unsafe { &*p })
}


// pub fn as_mut<'a, T>(ptr: *mut T) -> &'a mut T {
//     assert!(!ptr.is_null());
//     unsafe { &mut *ptr }
// }

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

pub fn bootstrap(spec: &str) -> *const c_char {
    // FIXME: Leaks string.
    // unwrap is ok because our json strings won't contain null bytes
    into_c_char_p(spec.to_owned()).unwrap_assert()
}

#[allow(non_camel_case_types)]
pub type c_bool = u8;  // PLATFORM DEPENDENT!!!

pub fn to_bool(b: c_bool) -> bool {
    if b != 0 { true } else { false }
}

pub fn from_bool(b: bool) -> c_bool {
    if b {1} else {0}
}


#[cfg(test)]
mod tests {
    use opendp::dist::L1Sensitivity;

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
        assert_eq!(Type::of::<L1Sensitivity<i32>>(), Type::new(TypeId::of::<L1Sensitivity<i32>>(), "L1Sensitivity<i32>", TypeContents::GENERIC { name: "L1Sensitivity", args: vec![i32_t] }));
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
        assert_eq!(TryInto::<Type>::try_into("L1Sensitivity<i32>")?, Type::new(TypeId::of::<L1Sensitivity<i32>>(), "L1Sensitivity<i32>", TypeContents::GENERIC { name: "L1Sensitivity", args: vec![i32_t] }));
        assert_eq!(TryInto::<Type>::try_into("Vec<i32>")?, Type::new(TypeId::of::<Vec<i32>>(), "Vec<i32>", TypeContents::VEC(i32_t)));
        Ok(())
    }

    #[test]
    fn test_type_args_try_from_vec() {
        let parsed: TypeArgs = "<Vec<i32>>".try_into().unwrap_test();
        let explicit = TypeArgs(vec![Type::of::<Vec<i32>>()]);
        assert_eq!(parsed, explicit);
    }

    #[test]
    fn test_type_args_try_from_numbers() {
        let parsed: TypeArgs = "<i32, f64>".try_into().unwrap_test();
        let explicit = TypeArgs(vec![Type::of::<i32>(), Type::of::<f64>()]);
        assert_eq!(parsed, explicit);
    }

    #[test]
    fn test_type_args() {
        let parsed: TypeArgs = "<i32, f32>".try_into().unwrap_test();
        let explicit = TypeArgs(vec![Type::of::<i32>(), Type::of::<f32>()]);
        assert_eq!(parsed, explicit);
    }
}

use std::any;
use std::any::TypeId;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

#[derive(Debug)]
pub struct TypeError;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Type {
    pub id: TypeId,
    pub name: &'static str,
    pub descriptor: &'static str,
}

impl Type {
    pub fn new<T: 'static>() -> Type {
        let descriptor = any::type_name::<T>();
        Self::new_descriptor::<T>(descriptor)
    }

    pub fn new_descriptor<T: 'static>(descriptor: &'static str) -> Type {
        let id = TypeId::of::<T>();
        let name = any::type_name::<T>();
        Type { id, name, descriptor }
    }

    // Hacky special entry point for composition.
    pub fn new_box_pair(type0: &Type, type1: &Type) -> Type {
        fn monomorphize<T0: 'static, T1: 'static>(type0: &Type, type1: &Type) -> Type {
            let descriptor = format!("(Box<{}>, Box<{}>)", type0.descriptor, type1.descriptor);
            // Hacky way to get &'static str from String.
            let descriptor = Box::leak(descriptor.into_boxed_str());
            Type::new_descriptor::<(Box<T0>, Box<T1>)>(descriptor)
        }
        dispatch!(
            monomorphize,
            // FIXME: The Box<f64> entries are here for demo use.
            [(type0, [u32, u64, i32, i64, f32, f64, bool, String, u8, (Box<f64>, Box<f64>)]), (type1, [u32, u64, i32, i64, f32, f64, bool, String, u8, (Box<f64>, Box<f64>)])],
            (type0, type1)
        )
    }
}

macro_rules! descriptor_types {
    ($($type:ty),*) => { vec![$(Type::new_descriptor::<$type>(stringify!($type))),*] }
}
lazy_static! {
    static ref DESCRIPTOR_TO_TYPE: HashMap<String, Type> = {
        descriptor_types![
            bool, char, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, String
        ].into_iter().map(|e| (e.descriptor.to_owned(), e)).collect()
    };
}

impl TryFrom<&str> for Type {
    type Error = TypeError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let type_ = DESCRIPTOR_TO_TYPE.get(value);
        type_.map(|e| e.clone()).ok_or(TypeError)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TypeArgs(pub(crate) Vec<Type>);

impl TypeArgs {
    pub fn expect(descriptor: *const c_char, count: usize) -> TypeArgs {
        let descriptor = to_str(descriptor);
        let type_args: TypeArgs = descriptor.try_into().expect("Bogus type args");
        assert!(type_args.0.len() == count);
        type_args
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
    type Error = TypeError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with("<") && value.ends_with(">") {
            let value = &value[1..value.len()-1];
            let split = value.split(",");
            let types: Result<Vec<_>, _> = split.into_iter().map(|e| e.trim().try_into()).collect();
            Ok(TypeArgs(types?))
        } else {
            Err(TypeError)
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

// pub fn as_raw<T>(o: &T) -> *mut T {
//     o as *mut T
// }

pub fn into_owned<T>(p: *mut T) -> T {
    assert!(!p.is_null());
    *unsafe { Box::<T>::from_raw(p) }
}

pub fn as_ref<'a, T>(p: *const T) -> &'a T {
    assert!(!p.is_null());
    unsafe { &*p }
}

// pub fn as_mut<'a, T>(ptr: *mut T) -> &'a mut T {
//     assert!(!ptr.is_null());
//     unsafe { &mut *ptr }
// }

pub fn into_c_char_p(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

// pub fn into_string(p: *mut c_char) -> String {
//     assert!(!p.is_null());
//     let s = unsafe { CString::from_raw(p) };
//     s.into_string().expect("Bad C string")
// }

pub fn to_str<'a>(p: *const c_char) -> &'a str {
    assert!(!p.is_null());
    let s = unsafe { CStr::from_ptr(p) };
    s.to_str().expect("Bad C string")
}

pub fn to_option_str<'a>(p: *const c_char) -> Option<&'a str> {
    if !p.is_null() {
        Some(to_str(p))
    } else {
        None
    }
}

pub fn bootstrap(spec: &str) -> *const c_char {
    // FIXME: Leaks string.
    into_c_char_p(spec.to_owned())
}

#[allow(non_camel_case_types)]
pub type c_bool = u8;  // PLATFORM DEPENDENT!!!

pub fn to_bool(b: c_bool) -> bool {
    if b != 0 { true } else { false }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type() {
        let parsed: Type = "i32".try_into().unwrap();
        let explicit = Type::new::<i32>();
        assert_eq!(parsed, explicit);
    }

    #[test]
    fn test_type_args() {
        let parsed: TypeArgs = "<i32, f32>".try_into().unwrap();
        let explicit = TypeArgs(vec![Type::new::<i32>(), Type::new::<f32>()]);
        assert_eq!(parsed, explicit);
    }
}

use std::collections::HashMap;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::slice;

use opendp::data::Column;

use crate::core::{FfiObject, FfiOwnership, FfiResult, FfiSlice, FfiError};
use crate::util;
use crate::util::{Type, TypeContents, c_bool, parse_type_args};
use std::fmt::Debug;
use opendp::error::Fallible;
use opendp::{fallible, err};

#[no_mangle]
pub extern "C" fn opendp_data__slice_as_object(type_args: *const c_char, raw: *const FfiSlice) -> FfiResult<*mut FfiObject> {
    fn raw_to_plain<T: Clone>(raw: &FfiSlice) -> Fallible<*const c_void> {
        if raw.len != 1 {
            return fallible!(FFI, "The slice length must be one when creating a scalar from FfiSlice")
        }
        let plain = util::as_ref(raw.ptr as *const T)
            .ok_or_else(|| err!(FFI, "Attempted to follow a null pointer to create an object"))?.clone();
        Ok(util::into_raw(plain) as *const c_void)
    }
    fn raw_to_string(raw: &FfiSlice) -> Fallible<*const c_void> {
        let string = util::to_str(raw.ptr as *const c_char)?.to_owned();
        Ok(util::into_raw(string) as *const c_void)
    }
    fn raw_to_slice<T: Clone>(_raw: &FfiSlice) -> Fallible<*const c_void> {
        // TODO: Need to do some extra wrapping to own the slice here.
        unimplemented!()
    }
    #[allow(clippy::unnecessary_wraps)]
    fn raw_to_vec<T: Clone>(raw: &FfiSlice) -> Fallible<*const c_void> {
        let slice = unsafe { slice::from_raw_parts(raw.ptr as *const T, raw.len) };
        let vec = slice.to_vec();
        Ok(util::into_raw(vec) as *const c_void)
    }
    fn raw_to_tuple<T0: Clone + Debug, T1: Clone + Debug>(raw: &FfiSlice) -> Fallible<*const c_void> {
        if raw.len != 2 {
            return fallible!(FFI, "The slice length must be two when creating a tuple from FfiSlice");
        }
        let slice = unsafe {slice::from_raw_parts(raw.ptr as *const *const c_void, 2)};

        let tuple = util::as_ref(slice[0] as *const T0).cloned()
            .zip(util::as_ref(slice[1] as *const T1).cloned())
            .ok_or_else(|| err!(FFI, "Attempted to follow a null pointer to create a tuple"))?;
        // println!("rust: {:?}", tuple);
        Ok(util::into_raw(tuple) as *const c_void)
    }
    let type_args = try_!(parse_type_args(type_args, 1));
    let type_ = type_args[0].clone();
    let raw = try_as_ref!(raw);
    let val = try_!(match type_.contents {
        TypeContents::PLAIN("String") => {
            raw_to_string(raw)
        },
        TypeContents::SLICE(element_id) => {
            let element = try_!(Type::of_id(&element_id));
            dispatch!(raw_to_slice, [(element, @primitives)], (raw))
        },
        TypeContents::VEC(element_id) => {
            let element = try_!(Type::of_id(&element_id));
            dispatch!(raw_to_vec, [(element, @primitives)], (raw))
        },
        TypeContents::TUPLE(ref element_ids) => {
            if element_ids.len() != 2 {
                return fallible!(FFI, "Only tuples of length 2 are supported").into();
            }
            if let Ok(types) = element_ids.iter().map(Type::of_id).collect::<Fallible<Vec<_>>>() {
                // primitively typed tuples
                dispatch!(raw_to_tuple, [(types[0], @primitives), (types[1], @primitives)], (raw))
            } else {
                // boxy tuples
                dispatch!(raw_to_plain, [(type_, @primitives)], (raw))
            }
        }
        _ => { dispatch!(raw_to_plain, [(type_, @primitives)], (raw)) }
    });
    let val = unsafe { Box::from_raw(val as *mut ()) };
    FfiResult::Ok(util::into_raw(FfiObject::new(type_, val, FfiOwnership::LIBRARY)))
}

#[no_mangle]
pub extern "C" fn opendp_data__object_type(this: *mut FfiObject) -> FfiResult<*mut c_char> {
    let obj = try_as_ref!(this);

    match util::into_c_char_p(obj.type_.descriptor.to_string()) {
        Ok(v) => FfiResult::Ok(v),
        Err(e) => e.into()
    }
}

#[no_mangle]
pub extern "C" fn opendp_data__object_as_slice(obj: *const FfiObject) -> FfiResult<*mut FfiSlice> {
    fn plain_to_raw(obj: &FfiObject) -> FfiResult<*mut FfiSlice> {
        let plain: &c_void = obj.as_ref();
        FfiResult::Ok(FfiSlice::new_raw(plain as *const c_void as *mut c_void, 1))
    }
    fn string_to_raw(obj: &FfiObject) -> FfiResult<*mut FfiSlice> {
        // // FIXME: There's no way to get a CString without copying, so this leaks.
        let string: &String = obj.as_ref();
        FfiResult::Ok(try_!(util::into_c_char_p(string.clone())
            .map(|char_p| FfiSlice::new_raw(char_p as *mut c_void, string.len() + 1))))
    }
    fn slice_to_raw<T>(_obj: &FfiObject) -> FfiResult<*mut FfiSlice> {
        // TODO: Need to get a reference to the slice here.
        unimplemented!()
    }
    fn vec_to_raw<T: 'static>(obj: &FfiObject) -> FfiResult<*mut FfiSlice> {
        let vec: &Vec<T> = obj.as_ref();
        FfiResult::Ok(FfiSlice::new_raw(vec.as_ptr() as *mut c_void, vec.len()))
    }
    fn tuple_to_raw<T0: 'static + Clone + Debug, T1: 'static + Clone + Debug>(obj: &FfiObject) -> FfiResult<*mut FfiSlice> {
        let tuple: &(T0, T1) = obj.as_ref();
        FfiResult::Ok(FfiSlice::new_raw(util::into_raw([
            &tuple.0 as *const T0 as *const c_void,
            &tuple.1 as *const T1 as *const c_void
        ]) as *mut c_void, 2))
    }
    let obj = try_as_ref!(obj);
    match &obj.type_.contents {
        TypeContents::PLAIN("String") => {
            string_to_raw(obj)
        },
        TypeContents::SLICE(element_id) => {
            let element = try_!(Type::of_id(element_id));
            dispatch!(slice_to_raw, [(element, @primitives)], (obj))
        },
        TypeContents::VEC(element_id) => {
            let element = try_!(Type::of_id(element_id));
            dispatch!(vec_to_raw, [(element, @primitives)], (obj))
        },
        TypeContents::TUPLE(element_ids) => {
            if element_ids.len() != 2 {
                return fallible!(FFI, "Only tuples of length 2 are supported").into();
            }
            if let Ok(types) = element_ids.iter().map(Type::of_id).collect::<Fallible<Vec<_>>>() {
                // primitively typed tuples
                dispatch!(tuple_to_raw, [(types[0], @primitives), (types[1], @primitives)], (obj))
            } else {
                // boxy tuples
                plain_to_raw(obj)
            }
        }
        _ => plain_to_raw(obj)
    }
}

#[no_mangle]
pub extern "C" fn opendp_data__object_free(this: *mut FfiObject) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
/// Frees the slice, but not what the slice references!
pub extern "C" fn opendp_data__slice_free(this: *mut FfiSlice) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
pub extern "C" fn opendp_data__str_free(this: *mut c_char) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
pub extern "C" fn opendp_data__bool_free(this: *mut c_bool) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

// TODO: Remove this function once we have composition and/or tuples sorted out.
#[no_mangle]
pub extern "C" fn opendp_data__to_string(this: *const FfiObject) -> FfiResult<*mut c_char> {
    fn monomorphize<T: 'static + std::fmt::Debug>(this: &FfiObject) -> Fallible<*mut c_char> {
        let this = this.as_ref::<T>();
        // FIXME: Figure out how to implement general to_string().
        let string = format!("{:?}", this);
        // FIXME: Leaks string.
        util::into_c_char_p(string)
    }
    let this = try_as_ref!(this);
    let type_arg = &this.type_;
    dispatch!(monomorphize, [(type_arg, [
        u32, u64, i32, i64, f32, f64, bool, String, u8, Column,
        Vec<u32>, Vec<u64>, Vec<i32>, Vec<i64>, Vec<f32>, Vec<f64>, Vec<bool>, Vec<String>, Vec<u8>, Vec<Column>, Vec<Vec<String>>,
        HashMap<String, Column>,
        // FIXME: The following are for Python demo use of compositions. Need to figure this out!!!
        (Box<i32>, Box<f64>),
        (Box<i32>, Box<u32>),
        (Box<(Box<f64>, Box<f64>)>, Box<f64>)
    ])], (this)).map_or_else(
        |e| FfiResult::Err(util::into_raw(FfiError::from(e))),
        FfiResult::Ok)
}

#[no_mangle]
pub extern "C" fn opendp_data__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "to_string", "args": [ ["const FfiObject *", "this"] ], "ret": "const char *" },
    { "name": "slice_as_object", "args": [ ["const char *", "type_args"], ["const void *", "raw"] ], "ret": "FfiResult<const FfiObject *>" },
    { "name": "object_type", "args": [ ["const FfiObject *", "this"] ], "ret": "FfiResult<const char *>" },
    { "name": "object_as_slice", "args": [ ["const FfiObject *", "this"] ], "ret": "FfiResult<const FfiSlice *>" },
    { "name": "object_free", "args": [ ["FfiObject *", "this"] ], "ret": "FfiResult<void *>" },
    { "name": "slice_free", "args": [ ["FfiSlice *", "this"] ], "ret": "FfiResult<void *>" },
    { "name": "str_free", "args": [ ["const char *", "this"] ], "ret": "FfiResult<void *>" },
    { "name": "bool_free", "args": [ ["bool *", "this"] ], "ret": "FfiResult<void *>" }
]
}"#;
    util::bootstrap(spec)
}

#[cfg(test)]
mod tests {
    use crate::util;
    use opendp::error::*;

    use super::*;

    #[test]
    fn test_data_new_number() {
        let val_in = 999;
        let raw_ptr = util::into_raw(val_in) as *mut c_void;
        let raw_len = 1;
        let raw = FfiSlice::new_raw(raw_ptr, raw_len);
        let res = opendp_data__slice_as_object(util::into_c_char_p("<i32>".to_owned()).unwrap_test(), raw);
        match res {
            FfiResult::Ok(obj) => {
                let obj = util::as_ref(obj).unwrap_test();
                let val_out: &i32 = obj.as_ref();
                assert_eq!(&val_in, val_out);
                if let FfiResult::Err(_) = opendp_data__object_free(obj as *const FfiObject as *mut FfiObject) {
                    panic!("Got Err!")
                }
            },
            FfiResult::Err(_) => panic!("Got Err!"),
        }
    }

    #[test]
    fn test_data_new_string() {
        let val_in = "Hello".to_owned();
        let raw_ptr = util::into_c_char_p(val_in.clone()).unwrap_test() as *mut c_void;
        let raw_len = val_in.len() + 1;
        let raw = FfiSlice::new_raw(raw_ptr, raw_len);
        let res = opendp_data__slice_as_object(util::into_c_char_p("<String>".to_owned()).unwrap_test(), raw);
        match res {
            FfiResult::Ok(obj) => {
                let obj = util::as_ref(obj).unwrap_test();
                let val_out: &String = obj.as_ref();
                assert_eq!(&val_in, val_out);
                if let FfiResult::Err(_) = opendp_data__object_free(obj as *const FfiObject as *mut FfiObject) {
                    panic!("Got Err!")
                }
            },
            FfiResult::Err(_) => panic!("Got Err!"),
        }
    }

    #[test]
    fn test_data_new_vec() {
        let val_in = vec![1, 2, 3];
        let raw_ptr = val_in.as_ptr() as *mut c_void;
        let raw_len = val_in.len();
        let raw = FfiSlice::new_raw(raw_ptr, raw_len);
        match opendp_data__slice_as_object(util::into_c_char_p("<Vec<i32>>".to_owned()).unwrap_test(), raw) {
            FfiResult::Ok(obj) => {
                let obj = util::as_ref(obj).unwrap_test();
                let val_out: &Vec<i32> = obj.as_ref();
                assert_eq!(&val_in, val_out);
                if let FfiResult::Err(_) = opendp_data__object_free(obj as *const FfiObject as *mut FfiObject) {
                    panic!("Got Err!")
                }
            },
            FfiResult::Err(_) => panic!("Got Err!"),
        }
    }

    #[test]
    fn test_data_as_raw_number() {
        let val_in = 999;
        let obj = FfiObject::new_raw_from_type(val_in);
        match opendp_data__object_as_slice(obj) {
            FfiResult::Ok(obj) => {
                let raw = util::as_ref(obj).unwrap_test();
                assert_eq!(raw.len, 1);
                let val_out = util::as_ref(raw.ptr as *const i32).unwrap_test();
                assert_eq!(&val_in, val_out);
                if let FfiResult::Err(_) = opendp_data__slice_free(raw as *const FfiSlice as *mut FfiSlice) {
                    panic!("Got Err!")
                }
            },
            FfiResult::Err(_) => panic!("Got Err!"),
        }
        if let FfiResult::Err(_) = opendp_data__object_free(obj) {
            panic!("Got Err!")
        }
    }

    #[test]
    fn test_data_as_raw_string() {
        let val_in = "Hello".to_owned();
        let obj = FfiObject::new_raw_from_type(val_in.clone());
        match opendp_data__object_as_slice(obj) {
            FfiResult::Ok(obj) => {
                let raw = util::as_ref(obj).unwrap_test();
                assert_eq!(raw.len, val_in.len() + 1);
                let val_out = util::to_str(raw.ptr as *const c_char).unwrap_test().to_owned();
                assert_eq!(val_in, val_out);
                if let FfiResult::Err(_) = opendp_data__slice_free(raw as *const FfiSlice as *mut FfiSlice) {
                    panic!("Got Err!")
                }
            },
            FfiResult::Err(_) => panic!("Got Err!"),
        }
        if let FfiResult::Err(_) = opendp_data__object_free(obj) {
            panic!("Got Err!")
        }
    }

    #[test]
    fn test_data_as_raw_vec() {
        let val_in = vec![1, 2, 3];
        let obj = FfiObject::new_raw_from_type(val_in.clone());
        match opendp_data__object_as_slice(obj) {
            FfiResult::Ok(obj) => {
                let raw = util::as_ref(obj).unwrap_test();
                assert_eq!(raw.len, val_in.len());
                let val_out = unsafe { Vec::from_raw_parts(raw.ptr as *mut i32, raw.len, raw.len) };
                assert_eq!(val_in, val_out);
                if let FfiResult::Err(_) = opendp_data__slice_free(raw as *const FfiSlice as *mut FfiSlice) {
                    panic!("Got Err!")
                }
            },
            FfiResult::Err(_) => panic!("Got Err!"),
        }
        if let FfiResult::Err(_) = opendp_data__object_free(obj) {
            panic!("Got Err!")
        }
    }

}
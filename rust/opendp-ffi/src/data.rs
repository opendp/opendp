use std::collections::HashMap;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::slice;

use opendp::data::Column;

use crate::core::{FfiObject, FfiOwnership, FfiResult, FfiSlice};
use crate::util;
use crate::util::{TypeArgs, TypeContents, Type};

#[no_mangle]
pub extern "C" fn opendp_data__object_new(type_args: *const c_char, raw: *const FfiSlice) -> FfiResult<*mut FfiObject> {
    fn raw_to_plain<T: Clone>(raw: &FfiSlice) -> *const c_void {
        assert_eq!(raw.len, 1);
        let plain = util::as_ref(raw.ptr as *const T).clone();
        util::into_raw(plain) as *const c_void
    }
    fn raw_to_string(raw: &FfiSlice) -> *const c_void {
        let string = util::to_str(raw.ptr as *const c_char).to_owned();
        util::into_raw(string) as *const c_void
    }
    fn raw_to_slice<T: Clone>(_raw: &FfiSlice) -> *const c_void {
        // TODO: Need to do some extra wrapping to own the slice here.
        unimplemented!()
    }
    fn raw_to_vec<T: Clone>(raw: &FfiSlice) -> *const c_void {
        let slice = unsafe { slice::from_raw_parts(raw.ptr as *const T, raw.len) };
        let vec = slice.to_vec();
        util::into_raw(vec) as *const c_void
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let type_ = type_args.0[0].clone();
    let raw = util::as_ref(raw);
    let val = match type_.contents {
        TypeContents::PLAIN("String") => {
            raw_to_string(raw)
        },
        TypeContents::SLICE(element_id) => {
            let element = Type::of_id(element_id);
            dispatch!(raw_to_slice, [(element, @primitives)], (raw))
        },
        TypeContents::VEC(element_id) => {
            let element = Type::of_id(element_id);
            dispatch!(raw_to_vec, [(element, @primitives)], (raw))
        },
        _ => { dispatch!(raw_to_plain, [(type_, @primitives)], (raw)) }
    };
    let val = unsafe { Box::from_raw(val as *mut ()) };
    FfiResult::Ok(FfiObject::new(type_, val, FfiOwnership::LIBRARY))
}

#[no_mangle]
pub extern "C" fn opendp_data__object_type(this: *mut FfiObject) -> FfiResult<*mut c_char> {
    let obj = util::as_ref(this);
    FfiResult::Ok(util::into_c_char_p(obj.type_.descriptor.to_string()))
}

#[no_mangle]
pub extern "C" fn opendp_data__object_as_raw(obj: *const FfiObject) -> FfiResult<*mut FfiSlice> {
        fn plain_to_raw(obj: &FfiObject) -> *mut FfiSlice {
            let plain: &c_void = obj.as_ref();
            FfiSlice::new(plain as *const c_void as *mut c_void, 1)
        }
        fn string_to_raw(obj: &FfiObject) -> *mut FfiSlice {
            // // FIXME: There's no way to get a CString without copying, so this leaks.
            let string: &String = obj.as_ref();
            let char_p = util::into_c_char_p(string.clone());
            FfiSlice::new(char_p as *mut c_void, string.len() + 1)
        }
        fn slice_to_raw<T>(_obj: &FfiObject) -> *mut FfiSlice {
            // TODO: Need to get a reference to the slice here.
            unimplemented!()
        }
        fn vec_to_raw<T: 'static>(obj: &FfiObject) -> *mut FfiSlice {
            let vec: &Vec<T> = obj.as_ref();
            FfiSlice::new(vec.as_ptr() as *mut c_void, vec.len())
        }
        let obj = util::as_ref(obj);
        let raw = match obj.type_.contents {
            TypeContents::PLAIN("String") => {
                string_to_raw(obj)
            },
            TypeContents::SLICE(element_id) => {
                let element = Type::of_id(element_id);
                dispatch!(slice_to_raw, [(element, @primitives)], (obj))
            },
            TypeContents::VEC(element_id) => {
                let element = Type::of_id(element_id);
                dispatch!(vec_to_raw, [(element, @primitives)], (obj))
            },
            _ => { plain_to_raw(obj) }
        };
        FfiResult::Ok(raw)
}

#[no_mangle]
pub extern "C" fn opendp_data__object_free(this: *mut FfiObject) {
    util::into_owned(this);
}

#[no_mangle]
/// Frees the slice, but not what the slice references!
pub extern "C" fn opendp_data__slice_free(this: *mut FfiSlice) {
    util::into_owned(this);
}

// TODO: Remove this function once we have composition and/or tuples sorted out.
#[no_mangle]
pub extern "C" fn opendp_data__to_string(this: *const FfiObject) -> *const c_char {
    fn monomorphize<T: 'static + std::fmt::Debug>(this: &FfiObject) -> *const c_char {
        let this = this.as_ref::<T>();
        // FIXME: Figure out how to implement general to_string().
        let string = format!("{:?}", this);
        // FIXME: Leaks string.
        util::into_c_char_p(string)
    }
    let this = util::as_ref(this);
    let type_arg = &this.type_;
    dispatch!(monomorphize, [(type_arg, [
        u32, u64, i32, i64, f32, f64, bool, String, u8, Column,
        Vec<u32>, Vec<u64>, Vec<i32>, Vec<i64>, Vec<f32>, Vec<f64>, Vec<bool>, Vec<String>, Vec<u8>, Vec<Column>, Vec<Vec<String>>,
        HashMap<String, Column>,
        // FIXME: The following are for Python demo use of compositions. Need to figure this out!!!
        (Box<i32>, Box<f64>),
        (Box<i32>, Box<u32>),
        (Box<(Box<f64>, Box<f64>)>, Box<f64>)
    ])], (this))
}

#[no_mangle]
pub extern "C" fn opendp_data__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "to_string", "args": [ ["const FfiObject *", "this"] ], "ret": "const char *" },
    { "name": "object_new", "args": [ ["const char *", "type_args"], ["const void *", "raw"] ], "ret": "FfiResult<const FfiObject *>" },
    { "name": "object_type", "args": [ ["const FfiObject *", "this"] ], "ret": "FfiResult<const char *>" },
    { "name": "object_as_raw", "args": [ ["const FfiObject *", "this"] ], "ret": "FfiResult<const FfiSlice *>" },
    { "name": "object_free", "args": [ ["FfiObject *", "this"] ] },
    { "name": "slice_free", "args": [ ["FfiSlice *", "this"] ] }
]
}"#;
    util::bootstrap(spec)
}

#[cfg(test)]
mod tests {
    use crate::util;

    use super::*;

    #[test]
    fn test_data_new_number() {
        let val_in = 999;
        let raw_ptr = util::into_raw(val_in) as *mut c_void;
        let raw_len = 1;
        let raw = FfiSlice::new(raw_ptr, raw_len);
        let res = opendp_data__object_new(util::into_c_char_p("<i32>".to_owned()), raw);
        match res {
            FfiResult::Ok(obj) => {
                let obj = util::as_ref(obj);
                let val_out: &i32 = obj.as_ref();
                assert_eq!(&val_in, val_out);
                opendp_data__object_free(obj as *const FfiObject as *mut FfiObject)
            },
            FfiResult::Err(_) => assert!(false, "Got Err!"),
        }
    }

    #[test]
    fn test_data_new_string() {
        let val_in = "Hello".to_owned();
        let raw_ptr = util::into_c_char_p(val_in.clone()) as *mut c_void;
        let raw_len = val_in.len() + 1;
        let raw = FfiSlice::new(raw_ptr, raw_len);
        let res = opendp_data__object_new(util::into_c_char_p("<String>".to_owned()), raw);
        match res {
            FfiResult::Ok(obj) => {
                let obj = util::as_ref(obj);
                let val_out: &String = obj.as_ref();
                assert_eq!(&val_in, val_out);
                opendp_data__object_free(obj as *const FfiObject as *mut FfiObject)
            },
            FfiResult::Err(_) => assert!(false, "Got Err!"),
        }
    }

    #[test]
    fn test_data_new_vec() {
        let val_in = vec![1, 2, 3];
        let raw_ptr = val_in.as_ptr() as *mut c_void;
        let raw_len = val_in.len();
        let raw = FfiSlice::new(raw_ptr, raw_len);
        let res = opendp_data__object_new(util::into_c_char_p("<Vec<i32>>".to_owned()), raw);
        match res {
            FfiResult::Ok(obj) => {
                let obj = util::as_ref(obj);
                let val_out: &Vec<i32> = obj.as_ref();
                assert_eq!(&val_in, val_out);
                opendp_data__object_free(obj as *const FfiObject as *mut FfiObject)
            },
            FfiResult::Err(_) => assert!(false, "Got Err!"),
        }
    }

    #[test]
    fn test_data_as_raw_number() {
        let val_in = 999;
        let obj = FfiObject::new_from_type(val_in);
        let res = opendp_data__object_as_raw(obj);
        match res {
            FfiResult::Ok(obj) => {
                let raw = util::as_ref(obj);
                assert_eq!(raw.len, 1);
                let val_out = util::as_ref(raw.ptr as *const i32);
                assert_eq!(&val_in, val_out);
                opendp_data__slice_free(raw as *const FfiSlice as *mut FfiSlice)
            },
            FfiResult::Err(_) => assert!(false, "Got Err!"),
        }
        opendp_data__object_free(obj)
    }

    #[test]
    fn test_data_as_raw_string() {
        let val_in = "Hello".to_owned();
        let obj = FfiObject::new_from_type(val_in.clone());
        let res = opendp_data__object_as_raw(obj);
        match res {
            FfiResult::Ok(obj) => {
                let raw = util::as_ref(obj);
                assert_eq!(raw.len, val_in.len() + 1);
                let val_out = util::to_str(raw.ptr as *const c_char).to_owned();
                assert_eq!(val_in, val_out);
                opendp_data__slice_free(raw as *const FfiSlice as *mut FfiSlice)
            },
            FfiResult::Err(_) => assert!(false, "Got Err!"),
        }
        opendp_data__object_free(obj)
    }

    #[test]
    fn test_data_as_raw_vec() {
        let val_in = vec![1, 2, 3];
        let obj = FfiObject::new_from_type(val_in.clone());
        let res = opendp_data__object_as_raw(obj);
        match res {
            FfiResult::Ok(obj) => {
                let raw = util::as_ref(obj);
                assert_eq!(raw.len, val_in.len());
                let val_out = unsafe { Vec::from_raw_parts(raw.ptr as *mut i32, raw.len, raw.len) };
                assert_eq!(val_in, val_out);
                opendp_data__slice_free(raw as *const FfiSlice as *mut FfiSlice)
            },
            FfiResult::Err(_) => assert!(false, "Got Err!"),
        }
        opendp_data__object_free(obj)
    }

}
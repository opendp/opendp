use std::collections::HashMap;
use std::os::raw::c_char;

use opendp::data::Data;

use crate::core::FfiObject;
use crate::util;

#[no_mangle]
pub extern "C" fn opendp_data__from_string(p: *const c_char) -> *mut FfiObject {
    let s = util::to_str(p).to_owned();
    FfiObject::new(s)
}

#[no_mangle]
pub extern "C" fn opendp_data__to_string(this: *const FfiObject) -> *const c_char {
    fn monomorphize<T: std::fmt::Debug>(this: &FfiObject) -> *const c_char {
        let this = this.as_ref::<T>();
        // FIXME: Figure out how to implement general to_string().
        let string = format!("{:?}", this);
        // FIXME: Leaks string.
        util::into_c_char_p(string)
    }
    let this = util::as_ref(this);
    let type_arg = &this.type_;
    dispatch!(monomorphize, [(type_arg, [
        u32, u64, i32, i64, f32, f64, bool, String, u8, Data,
        Vec<u32>, Vec<u64>, Vec<i32>, Vec<i64>, Vec<f32>, Vec<f64>, Vec<bool>, Vec<String>, Vec<u8>, Vec<Data>, Vec<Vec<String>>,
        HashMap<String, Data>,
        // FIXME: The following are for Python demo use of compositions. Need to figure this out!!!
        (Box<i32>, Box<f64>),
        (Box<i32>, Box<u32>),
        (Box<(Box<f64>, Box<f64>)>, Box<f64>)
    ])], (this))
}

#[no_mangle]
pub extern "C" fn opendp_data__data_free(this: *mut FfiObject) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_data__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "from_string", "args": [ ["const char *", "s"] ], "ret": "void *" },
    { "name": "to_string", "args": [ ["void *", "this"] ], "ret": "const char *" },
    { "name": "data_free", "args": [ ["void *", "this"] ] }
]
}"#;
    util::bootstrap(spec)
}

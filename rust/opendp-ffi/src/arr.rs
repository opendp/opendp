#![allow(non_camel_case_types)]

use std::ffi::c_void;
use std::fmt::Debug;
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::Arc;

use arrow::{array, compute};
use arrow::array::{ArrayRef, Int64Array};
use arrow::ffi::{ArrowArray, FFI_ArrowArray, FFI_ArrowSchema};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FFI_Array {
    pub array: *const FFI_ArrowArray,
    pub schema: *const FFI_ArrowSchema,
    // pub user_data: *const c_void,
}

impl FFI_Array {
    pub fn new(array: *const FFI_ArrowArray, schema: *const FFI_ArrowSchema, user_data: *const c_void) -> Self {
        Self { array, schema }
    }
}

type AllocCB = extern fn (*const FFI_Array);
static mut ALLOC_CB: Option<AllocCB> = None;

#[no_mangle]
pub extern "C" fn opendp__arrow_init(alloc_cb: AllocCB) {
    println!("rust: init");
    unsafe { ALLOC_CB = Some(alloc_cb); }
    // let _res = do_alloc_cb();
}


// rust: before from_ffi, faa strong_count = 1
// rust: after try_from_raw, faa strong_count = 1
// rust: after to_data, faa strong_count = 3
// rust: after make_array, faa strong_count = 3
// rust: after from_ffi, faa strong_count = 2

fn from_ffi(ffi_array: FFI_Array) -> ArrayRef {
    unsafe { array::make_array_from_raw(ffi_array.array, ffi_array.schema) }.unwrap()
}

fn de_arc<T: Debug>(ptr: *const T) -> *const T {
    let arc = unsafe { Arc::from_raw(ptr) };
    let val = Arc::try_unwrap(arc).unwrap();
    Box::into_raw(Box::new(val))
}

fn strong_count<T: Debug>(ptr: *const T) -> usize {
    let arc= unsafe { Arc::from_raw(ptr) };
    let strong_count = Arc::strong_count(&arc);
    std::mem::forget(arc);
    strong_count
}

// fn print_strong_count(tag: &str, ffi_arr: FFI_Array) {
//     let strong_count = strong_count(ffi_arr.array);
//     println!("rust: {}: strong_count = {}", tag, strong_count);
// }

fn to_ffi(array: ArrayRef, ffi_array: FFI_Array) {
    let (temp_array, temp_schema) = array.to_raw().unwrap();
    unsafe { ptr::copy_nonoverlapping(temp_array, ffi_array.array as *mut FFI_ArrowArray, 1); }
    unsafe { ptr::copy_nonoverlapping(temp_schema, ffi_array.schema as *mut FFI_ArrowSchema, 1); }
    unsafe { std::mem::forget(Arc::try_unwrap(Arc::from_raw(temp_array)).unwrap()); }
    unsafe { std::mem::forget(Arc::try_unwrap(Arc::from_raw(temp_schema)).unwrap()); }
}

fn do_alloc_cb() -> FFI_Array {
    println!("rust: do_alloc_cb");
    let alloc_cb = unsafe { ALLOC_CB }.unwrap();
    let ffi_array = FFI_Array::new(ptr::null(), ptr::null(), ptr::null());
    alloc_cb(&ffi_array);
    ffi_array
}

fn to_ffi_cb(array: ArrayRef) -> FFI_Array {
    let (temp_array, temp_schema) = array.to_raw().unwrap();
    let (temp_array, temp_schema) = (de_arc(temp_array), de_arc(temp_schema));
    let ffi_array = do_alloc_cb();
    unsafe { ptr::copy_nonoverlapping(temp_array, ffi_array.array as *mut FFI_ArrowArray, 1); }
    unsafe { ptr::copy_nonoverlapping(temp_schema, ffi_array.schema as *mut FFI_ArrowSchema, 1); }
    ffi_array
}

#[no_mangle]
pub extern "C" fn opendp__arrow_identity(ffi_arg: FFI_Array, ffi_res: FFI_Array) {
    println!("rust: identity");
    let faa = ffi_arg.array;
    println!("rust: before from_ffi, faa strong_count = {:?}", strong_count(faa));

    let arg = from_ffi(ffi_arg);
    println!("rust: after from_ffi, faa strong_count = {:?}", strong_count(faa));
    // println!("rust: arg = {:?}", arg);

    let res = compute::limit(&arg, arg.len());
    println!("rust: after limit, faa strong_count = {:?}", strong_count(faa));
    // println!("rust: res = {:?}", res);

    to_ffi(res, ffi_res);
    println!("rust: DONE");

    println!("rust: after to_ffi, faa strong_count = {:?}", strong_count(faa));
    // drop(arg);
    // println!("rust: after drop, faa strong_count = {:?}", strong_count(faa));

}

#[no_mangle]
pub extern "C" fn opendp__arrow_identity_cb(ffi_arg: FFI_Array) -> FFI_Array {
    let arg = from_ffi(ffi_arg);
    println!("rust: arg = {:?}", arg);

    let res = compute::limit(&arg, arg.len());
    println!("rust: res = {:?}", res);

    let ffi_res = to_ffi_cb(res);
    println!("rust: DONE");
    ffi_res
}

#[no_mangle]
pub extern "C" fn opendp__arrow_sort(ffi_arg: FFI_Array, ffi_res: FFI_Array) {
    let arg = from_ffi(ffi_arg);
    println!("rust: arg = {:?}", arg);

    let res = compute::sort(&arg, None).unwrap();
    println!("rust: res = {:?}", res);

    to_ffi(res, ffi_res);
    println!("rust: DONE");
}

#[no_mangle]
pub extern "C" fn opendp__arrow_add(ffi_arg0: FFI_Array, ffi_arg1: FFI_Array, ffi_res: FFI_Array) {
    let arg0 = from_ffi(ffi_arg0);
    let arg1 = from_ffi(ffi_arg1);
    println!("rust: arg0 = {:?}, arg1 = {:?}", arg0, arg1);

    let arg0 = arg0.as_any().downcast_ref::<Int64Array>().unwrap();
    let arg1 = arg1.as_any().downcast_ref::<Int64Array>().unwrap();
    let res = compute::add(&arg0, &arg1).unwrap();
    let res = Arc::new(res);
    println!("rust, res = {:?}", res);

    to_ffi(res, ffi_res);
    println!("rust: DONE");
}


#[no_mangle]
pub extern "C" fn opendp__arrow_consume(ffi_arg: FFI_Array) {
    let _arg = from_ffi(ffi_arg);
    // println!("rust: DONE");
}

#[no_mangle]
pub extern "C" fn opendp__arrow_produce(ffi_res: FFI_Array) {
    // println!("rust: produce");
    let res = Int64Array::from(vec![Some(999);10000]);
    let res = Arc::new(res);
    to_ffi(res, ffi_res);
    // println!("rust: did to_ffi");
    // println!("rust: DONE");
}


#[no_mangle]
pub extern "C" fn opendp__arrow_alloc() -> FFI_Array {
    let arrow_array = unsafe { ArrowArray::empty() };
    let (ffi_array, ffi_schema) = ArrowArray::into_raw(arrow_array);
    FFI_Array::new(ffi_array, ffi_schema, ptr::null())
}

#[no_mangle]
pub extern "C" fn opendp__arrow_free(ffi_arg: FFI_Array) {
    println!("rust: free");
    // let arrow_array = unsafe { ArrowArray::try_from_raw(ffi_arg.array, ffi_arg.schema) }.unwrap();
    unsafe { Box::from_raw(ffi_arg.array as *mut FFI_ArrowArray); }
    unsafe { Box::from_raw(ffi_arg.schema as *mut FFI_ArrowSchema); }
    println!("rust: free: DONE");
}

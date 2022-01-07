#![allow(non_camel_case_types)]

use std::fmt::Debug;
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
}

impl FFI_Array {
    pub fn new(array: *const FFI_ArrowArray, schema: *const FFI_ArrowSchema) -> Self {
        Self { array, schema }
    }
}

fn from_ffi(ffi_array: FFI_Array) -> ArrayRef {
    unsafe { array::make_array_from_raw(ffi_array.array, ffi_array.schema) }.unwrap()
}

fn _de_arc<T: Debug>(ptr: *const T) -> *const T {
    let arc = unsafe { Arc::from_raw(ptr) };
    let val = Arc::try_unwrap(arc).unwrap();
    Box::into_raw(Box::new(val))
}

fn to_ffi(array: ArrayRef, ffi_array: FFI_Array) {
    let (temp_array, temp_schema) = array.to_raw().unwrap();
    unsafe { ptr::copy_nonoverlapping(temp_array, ffi_array.array as *mut FFI_ArrowArray, 1); }
    unsafe { ptr::copy_nonoverlapping(temp_schema, ffi_array.schema as *mut FFI_ArrowSchema, 1); }
    unsafe { std::mem::forget(Arc::try_unwrap(Arc::from_raw(temp_array)).unwrap()); }
    unsafe { std::mem::forget(Arc::try_unwrap(Arc::from_raw(temp_schema)).unwrap()); }
}

#[no_mangle]
pub extern "C" fn opendp__arrow_identity(ffi_arg: FFI_Array, ffi_res: FFI_Array) {
    let arg = from_ffi(ffi_arg);
    // println!("rust: arg = {:?}", arg);

    let res = compute::limit(&arg, arg.len());
    // println!("rust: res = {:?}", res);

    to_ffi(res, ffi_res);
    // println!("rust: DONE");
}

#[no_mangle]
pub extern "C" fn opendp__arrow_sort(ffi_arg: FFI_Array, ffi_res: FFI_Array) {
    let arg = from_ffi(ffi_arg);
    // println!("rust: arg = {:?}", arg);

    let res = compute::sort(&arg, None).unwrap();
    // println!("rust: res = {:?}", res);

    to_ffi(res, ffi_res);
    // println!("rust: DONE");
}

#[no_mangle]
pub extern "C" fn opendp__arrow_add(ffi_arg0: FFI_Array, ffi_arg1: FFI_Array, ffi_res: FFI_Array) {
    let arg0 = from_ffi(ffi_arg0);
    let arg1 = from_ffi(ffi_arg1);
    // println!("rust: arg0 = {:?}, arg1 = {:?}", arg0, arg1);

    let arg0 = arg0.as_any().downcast_ref::<Int64Array>().unwrap();
    let arg1 = arg1.as_any().downcast_ref::<Int64Array>().unwrap();
    let res = compute::add(&arg0, &arg1).unwrap();
    let res = Arc::new(res);
    // println!("rust, res = {:?}", res);

    to_ffi(res, ffi_res);
    // println!("rust: DONE");
}


#[no_mangle]
pub extern "C" fn opendp__arrow_consume(ffi_arg: FFI_Array) {
    let _arg = from_ffi(ffi_arg);
    // println!("rust: DONE");
}

#[no_mangle]
pub extern "C" fn opendp__arrow_produce(ffi_res: FFI_Array) {
    let res = Int64Array::from(vec![Some(999);10000]);
    let res = Arc::new(res);
    to_ffi(res, ffi_res);
    // println!("rust: DONE");
}


#[no_mangle]
pub extern "C" fn opendp__arrow_alloc() -> FFI_Array {
    let arrow_array = unsafe { ArrowArray::empty() };
    let (ffi_array, ffi_schema) = ArrowArray::into_raw(arrow_array);
    FFI_Array::new(ffi_array, ffi_schema)
}

#[no_mangle]
pub extern "C" fn opendp__arrow_free(ffi_arg: FFI_Array) {
    println!("rust: free");
    // let arrow_array = unsafe { ArrowArray::try_from_raw(ffi_arg.array, ffi_arg.schema) }.unwrap();
    unsafe { Box::from_raw(ffi_arg.array as *mut FFI_ArrowArray); }
    unsafe { Box::from_raw(ffi_arg.schema as *mut FFI_ArrowSchema); }
    println!("rust: free: DONE");
}

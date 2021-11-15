#![allow(non_camel_case_types)]

use arrow::{array, compute};
use arrow::array::{Int64Array, ArrayRef, BooleanArray};
use arrow::ffi::{ArrowArray, FFI_ArrowArray, FFI_ArrowSchema};
use std::sync::Arc;

#[derive(Debug)]
#[repr(C)]
pub struct FFI_ArrowArraySchema {
    pub array: *const FFI_ArrowArray,
    pub schema: *const FFI_ArrowSchema,
}

impl FFI_ArrowArraySchema {
    pub fn new(ffi_array: *const FFI_ArrowArray, ffi_schema: *const FFI_ArrowSchema) -> Self {
        Self { array: ffi_array, schema: ffi_schema }
    }
    pub fn new_raw(ffi_array: *const FFI_ArrowArray, ffi_schema: *const FFI_ArrowSchema) -> *const Self {
        let res = Self::new(ffi_array, ffi_schema);
        Box::into_raw(Box::new(res))
    }
}

fn from_ffi(ffi_array: *const FFI_ArrowArray, ffi_schema: *const FFI_ArrowSchema) -> ArrayRef {
    let arr = unsafe { array::make_array_from_raw(ffi_array, ffi_schema) }.unwrap();
    std::mem::forget(arr.clone());
    arr
}

fn to_ffi(array: ArrayRef) -> *const FFI_ArrowArraySchema {
    let (ffi_array, ffi_schema) = array.to_raw().unwrap();
    FFI_ArrowArraySchema::new_raw(ffi_array, ffi_schema)
}

#[no_mangle]
pub extern "C" fn opendp__arrow_identity(ffi_arg_array: *const FFI_ArrowArray, ffi_arg_schema: *const FFI_ArrowSchema) -> *const FFI_ArrowArraySchema {
    let arg = from_ffi(ffi_arg_array, ffi_arg_schema);
    println!("rust: arg = {:?}", arg);

    // let res = compute::limit(&arg, arg.len());
    // let res = compute::filter(&*arg, &BooleanArray::from(vec![true;arg.len()])).unwrap();
    let res = Arc::new(arrow::array::Int64Array::from(vec![Some(1), Some(2), Some(3)]));
    println!("rust: res = {:?}", res);

    let ffi_res = to_ffi(res);
    println!("rust: DONE");
    ffi_res
}

#[no_mangle]
pub extern "C" fn opendp__arrow_identity_param(ffi_arg_array: *const FFI_ArrowArray, ffi_arg_schema: *const FFI_ArrowSchema, parse: bool, gen: bool) -> *const FFI_ArrowArraySchema {
    if parse {
        unsafe { println!("rust: array = {:?}", &*ffi_arg_array); }
        let _arg = from_ffi(ffi_arg_array, ffi_arg_schema);
        std::mem::forget(_arg)
        // println!("rust: arg = {:?}", arg);
    }

    if gen {
        let res = Arc::new(arrow::array::Int64Array::from(vec![Some(1), Some(2), Some(3)]));
        println!("rust: res = {:?}", res);

        let ffi_res = to_ffi(res);
        println!("rust: DONE");
        ffi_res
    } else {
        let ffi_res = std::ptr::null();
        println!("rust: DONE");
        ffi_res
    }
}

#[no_mangle]
pub extern "C" fn opendp__arrow_sort(ffi_arg_array: *const FFI_ArrowArray, ffi_arg_schema: *const FFI_ArrowSchema) -> *const FFI_ArrowArraySchema {
    let arg = from_ffi(ffi_arg_array, ffi_arg_schema);
    println!("rust: arg = {:?}", arg);

    let res = compute::sort(&arg, None).unwrap();
    println!("rust: res = {:?}", res);

    let ffi_res = to_ffi(res);
    println!("rust: DONE");
    ffi_res
}

#[no_mangle]
pub extern "C" fn opendp__arrow_add(ffi_arg0_array: *const FFI_ArrowArray, ffi_arg0_schema: *const FFI_ArrowSchema, ffi_arg1_array: *const FFI_ArrowArray, ffi_arg1_schema: *const FFI_ArrowSchema) -> *const FFI_ArrowArraySchema {
    let arg0 = from_ffi(ffi_arg0_array, ffi_arg0_schema);
    let arg1 = from_ffi(ffi_arg1_array, ffi_arg1_schema);
    println!("rust: arg0 = {:?}, arg1 = {:?}", arg0, arg1);

    let arg0 = arg0.as_any().downcast_ref::<Int64Array>().unwrap();
    let arg1 = arg1.as_any().downcast_ref::<Int64Array>().unwrap();
    let res = compute::add(&arg0, &arg1).unwrap();
    let res = Arc::new(res);
    println!("rust, res = {:?}", res);

    let ffi_res = to_ffi(res);
    println!("rust: DONE");
    ffi_res
}


#[no_mangle]
pub extern "C" fn opendp__arrow_free(ffi_arg: *mut FFI_ArrowArraySchema) {
    let ffi_arr_sch = unsafe { Box::from_raw(ffi_arg) };
    let _drop = unsafe { ArrowArray::try_from_raw(ffi_arr_sch.array, ffi_arr_sch.schema) };
    println!("rust: DONE");
}

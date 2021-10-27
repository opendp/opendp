#![allow(non_camel_case_types)]

use arrow::{array, compute};
use arrow::array::{Array, Int64Array};
use arrow::ffi::{FFI_ArrowArray, FFI_ArrowSchema};
use std::ptr::null;

#[repr(C)]
pub struct FFI_ArrowArraySchema {
    pub array: *const FFI_ArrowArray,
    pub schema: *const FFI_ArrowSchema,
}

impl FFI_ArrowArraySchema {
    pub fn new_raw(array: *const FFI_ArrowArray, schema: *const FFI_ArrowSchema) -> *const Self {
        let res = Self { array, schema };
        Box::into_raw(Box::new(res))
    }
}


#[no_mangle]
pub extern "C" fn opendp__arrow_alloc() -> *const FFI_ArrowArraySchema {
    FFI_ArrowArraySchema::new_raw(Box::into_raw(Box::new(FFI_ArrowArray::empty())), Box::into_raw(Box::new(FFI_ArrowSchema::empty())))
}


#[no_mangle]
pub extern "C" fn opendp__arrow_identity(in0_array: *const FFI_ArrowArray, in0_schema: *const FFI_ArrowSchema, dry: bool) -> *const FFI_ArrowArraySchema {
    let in0 = unsafe { array::make_array_from_raw(in0_array, in0_schema) }.unwrap();
    println!("rust: in0 = {:?}", in0);

    let res = if !dry {
        let out = in0;
        println!("rust: out = {:?}", out);

        let (out_array, out_schema) = out.to_raw().unwrap();
        FFI_ArrowArraySchema::new_raw(out_array, out_schema)
    } else {
        null()
    };
    println!("rust: DONE");
    res
}


#[no_mangle]
pub extern "C" fn opendp__arrow_sort(in0_array: *const FFI_ArrowArray, in0_schema: *const FFI_ArrowSchema, dry: bool) -> *const FFI_ArrowArraySchema {
    let in0 = unsafe { array::make_array_from_raw(in0_array, in0_schema) }.unwrap();
    println!("rust: in0 = {:?}", in0);

    let res = if !dry {
        let out = compute::sort(&in0, None).unwrap();
        println!("rust: out = {:?}", out);

        let (out_array, out_schema) = out.to_raw().unwrap();
        FFI_ArrowArraySchema::new_raw(out_array, out_schema)
    } else {
        null()
    };
    println!("rust: DONE");
    res
}


#[no_mangle]
pub extern "C" fn opendp__arrow_sum(in0_array: *const FFI_ArrowArray, in0_schema: *const FFI_ArrowSchema, in1_array: *const FFI_ArrowArray, in1_schema: *const FFI_ArrowSchema, dry: bool) -> *const FFI_ArrowArraySchema {
    println!("rust: in0_array = {:?}, in0_schema = {:?}, in1_array = {:?}, in1_schema = {:?}", in0_array, in0_schema, in1_array, in1_schema);
    // let in0 = unsafe { array::make_array_from_raw(in0_array, in0_schema) }.unwrap();
    let in1 = unsafe { array::make_array_from_raw(in1_array, in1_schema) }.unwrap();
    // println!("rust: in0 = {:?}, in1 = {:?}", in0, in1);
    // let in0 = in0.as_any().downcast_ref::<Int64Array>().unwrap();
    // let in1 = in1.as_any().downcast_ref::<Int64Array>().unwrap();
    // let in0 = Int64Array::from(vec![Some(10), Some(20), None, Some(40)]);
    // let in1 = Int64Array::from(vec![Some(10), Some(20), Some(30), None]);
    // println!("rust: in0 = {:?}, in1 = {:?}", in0, in1);

    // let res = if !dry {
    //     let out = compute::add(&in0, &in1).unwrap();
    //     println!("rust, out = {:?}", out);
    //
    //     let (out_array, out_schema) = out.to_raw().unwrap();
    //     println!("rust: out_array = {:?}, out_schema = {:?}", out_array, out_schema);
    //     // std::mem::forget(out);
    //     FFI_ArrowArraySchema::new_raw(out_array, out_schema)
    // } else {
    //     null()
    // };
    // std::mem::forget(in0);
    // std::mem::forget(in1);
    println!("rust: DONE");
    // res
    null()
}

#![allow(non_camel_case_types)]

use arrow::array;
use arrow::compute;
use arrow::ffi::{FFI_ArrowArray, FFI_ArrowSchema};

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
pub extern "C" fn opendp__test_arrow(array: *const FFI_ArrowArray, schema: *const FFI_ArrowSchema) -> *const FFI_ArrowArraySchema {
    unsafe { println!("test_arrow({:?}, {:?})", &*array, &*schema); }
    let array = unsafe { array::make_array_from_raw(array, schema) }.unwrap();
    // println!("array = {:?}", array);
    // // let array = array.as_any().downcast_ref::<Int32Array>().unwrap();
    // let sum = compute::sort(&array, None).unwrap();
    // println!("sum = {:?}", sum);
    // let (sum_array, sum_schema) = sum.to_raw().unwrap();
    // println!("raw = ({:?}, {:?})", sum_array, sum_schema);
    // let res = FFI_ArrowArraySchema::new_raw(sum_array, sum_schema);
    let res = std::ptr::null();
    println!("res = {:?}", res);
    res
}

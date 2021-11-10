use arrow::array;
use arrow::array::{Array, Int32Array};
use arrow::ffi::{FFI_ArrowArray, FFI_ArrowSchema};

unsafe fn _go() {
    let array = Int32Array::from(vec![Some(1), None, Some(3)]);
    let (ffi_array_ptr, ffi_schema_ptr) = array.to_raw().unwrap();
    println!("go: did to_raw()");
    std::mem::drop(array);
    println!("go: did drop()");
    let recreate = true;
    if recreate {
        let _array = array::make_array_from_raw(ffi_array_ptr, ffi_schema_ptr);
    }

    let low = false;
    if low {
        let _ffi_array = Box::from_raw(ffi_array_ptr as *mut FFI_ArrowArray);
        let _ffi_schema = Box::from_raw(ffi_schema_ptr as *mut FFI_ArrowSchema);
    } else {
        let _array = array::make_array_from_raw(ffi_array_ptr, ffi_schema_ptr);
    }
    println!("go: DONE");
}

unsafe fn go2() {
    fn make() -> (*const FFI_ArrowArray, *const FFI_ArrowSchema) {
        let array = Int32Array::from(vec![Some(1), None, Some(3)]);
        array.to_raw().unwrap()
    }
    let (ffi_arr_ptr, ffi_sch_ptr) = make();
    let array = array::make_array_from_raw(ffi_arr_ptr, ffi_sch_ptr);
    println!("go2: array = {:?}", array);
    println!("go2: DONE");
}

fn main() {
    unsafe {
        // go();
        go2();
    }
}

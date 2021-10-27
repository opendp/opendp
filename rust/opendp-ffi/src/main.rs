use arrow::array::{Array, Int32Array, make_array_from_raw};
use arrow::compute::kernels::arithmetic;
use arrow::error::ArrowError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
// create an array natively
    let array = Int32Array::from(vec![Some(1), None, Some(3)]);

// export it
    let (array_ptr, schema_ptr) = array.to_raw()?;

// consumed and used by something else...

// import it
    let array = unsafe { make_array_from_raw(array_ptr, schema_ptr)? };

// perform some operation
    let array = array.as_any().downcast_ref::<Int32Array>().ok_or(
        ArrowError::ParseError("Expects an int32".to_string()),
    )?;
    let array = arithmetic::add(&array, &array)?;

// verify
    assert_eq!(array, Int32Array::from(vec![Some(2), None, Some(6)]));

    Ok(())
}

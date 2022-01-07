use std::ops::Add;
use std::sync::Arc;

use arrow::array::{ArrayRef, PrimitiveArray};
use arrow::compute::kernels::aggregate;
use arrow::datatypes::{ArrowNumericType, DataType, Float64Type, Int32Type};

pub fn sum_ffi(array: ArrayRef) -> ArrayRef {
    fn dispatch<T>(array: ArrayRef) -> ArrayRef
    where
        T: ArrowNumericType,
        T::Native: Add<Output = T::Native>,
    {
        let array: &PrimitiveArray<T> = array.as_any().downcast_ref().unwrap();
        let res = sum(array);
        let mut builder = PrimitiveArray::<T>::builder(1);
        builder.append_option(res).unwrap();
        Arc::new(builder.finish())
    }
    match array.data_type() {
        DataType::Int32 => dispatch::<Int32Type>(array),
        DataType::Float64 => dispatch::<Float64Type>(array),
        _ => panic!(),
    }
}

pub fn sum<T>(array: &PrimitiveArray<T>) -> Option<T::Native>
where
    T: ArrowNumericType,
    T::Native: Add<Output = T::Native>,
{
    aggregate::sum(array)
}

use std::marker::PhantomData;
use std::ops::{Add, Mul};
use std::sync::Arc;

use arrow::array::{ArrayRef, PrimitiveArray};
use arrow::compute::kernels::aggregate;
use arrow::datatypes::{ArrowNumericType, ArrowPrimitiveType, DataType, Float64Type, Int32Type};

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

pub trait Dom {
    type Carrier;
}

pub struct ArrayDom<D: Dom, T: ArrowPrimitiveType<Native = D::Carrier>> {
    _element_domain: D,
    _marker: PhantomData<T>,
}

impl<D: Dom, T: ArrowPrimitiveType<Native = D::Carrier>> Dom for ArrayDom<D, T> {
    type Carrier = PrimitiveArray<T>;
}

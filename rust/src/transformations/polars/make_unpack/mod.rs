use arrow2::array::{
    BooleanArray, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
    UInt16Array, UInt32Array, UInt64Array, UInt8Array, Utf8Array,
};
use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, OptionDomain, SeriesDomain, VectorDomain, PrimitiveDataType},
    error::Fallible,
    traits::CheckAtom,
};

// #[cfg(feature = "ffi")]
// mod ffi;

pub trait ToVec<T> {
    fn to_option_vec(&self) -> Fallible<Vec<Option<T>>>;

    fn to_vec(&self) -> Fallible<Vec<T>> {
        self.to_option_vec()?
            .into_iter()
            .map(|v| v.ok_or_else(|| err!(FailedFunction, "found nullity in data")))
            .collect()
    }
}

macro_rules! impl_to_vec {
    ($ty:ty, $polars_ty:ty, $polars_arr:ty) => {
        impl ToVec<$ty> for ChunkedArray<$polars_ty> {
            fn to_option_vec(&self) -> Fallible<Vec<Option<$ty>>> {
                Ok(self
                    .chunks()
                    .into_iter()
                    .map(|chunk| chunk.as_any().downcast_ref::<$polars_arr>().cloned())
                    .collect::<Option<Vec<$polars_arr>>>()
                    .ok_or_else(|| {
                        err!(
                            FailedFunction,
                            "downcast to {} failed",
                            stringify!($polars_ty)
                        )
                    })?
                    .into_iter()
                    .map(IntoIterator::into_iter)
                    .flatten()
                    .collect())
            }
        }
    };
}

impl_to_vec!(bool, BooleanType, BooleanArray);
impl_to_vec!(u8, UInt8Type, UInt8Array);
impl_to_vec!(u16, UInt16Type, UInt16Array);
impl_to_vec!(u32, UInt32Type, UInt32Array);
impl_to_vec!(u64, UInt64Type, UInt64Array);
impl_to_vec!(i8, Int8Type, Int8Array);
impl_to_vec!(i16, Int16Type, Int16Array);
impl_to_vec!(i32, Int32Type, Int32Array);
impl_to_vec!(i64, Int64Type, Int64Array);
impl_to_vec!(f32, Float32Type, Float32Array);
impl_to_vec!(f64, Float64Type, Float64Array);

impl ToVec<String> for ChunkedArray<Utf8Type> {
    fn to_option_vec(&self) -> Fallible<Vec<Option<String>>> {
        Ok(self
            .chunks()
            .into_iter()
            .map(|chunk| chunk.as_any().downcast_ref::<Utf8Array<i32>>().cloned())
            .collect::<Option<Vec<Utf8Array<i32>>>>()
            .ok_or_else(|| err!(FailedFunction, "downcast to Utf8Array<i32> failed"))?
            .into_iter()
            .map(|a| {
                a.into_iter()
                    .map(|s| s.map(|s| s.to_string()))
                    .collect::<Vec<Option<String>>>()
            })
            .flatten()
            .collect())
    }
}

pub fn item<T: 'static + PrimitiveDataType>(f: LazyFrame) -> Fallible<T>
where
    ChunkedArray<T::Polars>: ToVec<T>, {
    f.collect()?.get_columns()[0]
        .unpack::<T::Polars>()?
        .to_vec()?
        .into_iter()
        .next()
        .ok_or_else(|| err!(FailedFunction, "expected one item"))
}


#[bootstrap(features("contrib"), generics(M(suppress)))]
/// Unpack a Series from a DataFrame
pub fn make_series_to_option_vec<M: Metric, T: 'static + CheckAtom + PrimitiveDataType>(
    input_domain: SeriesDomain,
    input_metric: M,
) -> Fallible<Transformation<SeriesDomain, VectorDomain<OptionDomain<AtomDomain<T>>>, M, M>>
where
    ChunkedArray<T::Polars>: ToVec<T>,
    M::Distance: 'static + Clone,
    T::Polars: PolarsDataType,
    (SeriesDomain, M): MetricSpace,
    (VectorDomain<OptionDomain<AtomDomain<T>>>, M): MetricSpace,
{
    if T::dtype() != input_domain.field.dtype {
        return fallible!(MakeTransformation, "T must match dtype");
    }

    let output_domain = VectorDomain::new(OptionDomain::new(input_domain.atom_domain()?.clone()));

    // (try to extract T from the domain in FFI bindings)
    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(move |arg: &Series| {
            arg.unpack::<T::Polars>()?.to_option_vec()
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(M::Distance::clone),
    )
}


#[bootstrap(features("contrib"), generics(M(suppress)))]
/// Unpack a Series from a DataFrame
pub fn make_series_to_vec<M: Metric, T: 'static + CheckAtom + PrimitiveDataType>(
    input_domain: SeriesDomain,
    input_metric: M,
) -> Fallible<Transformation<SeriesDomain, VectorDomain<AtomDomain<T>>, M, M>>
where
    ChunkedArray<T::Polars>: ToVec<T>,
    M::Distance: 'static + Clone,
    (SeriesDomain, M): MetricSpace,
    (VectorDomain<AtomDomain<T>>, M): MetricSpace,
{
    if T::dtype() != input_domain.field.dtype {
        return fallible!(MakeTransformation, "T must match dtype");
    }

    if input_domain.nullable {
        return fallible!(MakeTransformation, "data must be non-null. Use make_series_to_option_vec.")
    }

    let output_domain = VectorDomain::new(input_domain.atom_domain()?.clone());

    // (try to extract T from the domain in FFI bindings)
    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg: &Series| {
            arg.unpack::<T::Polars>()?.to_vec()
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(M::Distance::clone),
    )
}

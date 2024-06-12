use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use crate::core::MetricSpace;
use crate::error::Fallible;
use crate::transformations::traits::UnboundedMetric;
use crate::{core::Domain, traits::CheckAtom};

use polars::prelude::*;

use crate::domains::{AtomDomain, OptionDomain};

#[cfg(feature = "ffi")]
mod ffi;

/// # Proof Definition
/// `SeriesDomain` is the domain of polars `Series` where:
/// * `field` - Holds Series name and type of underlying data.
/// * `element_domain` - Pointer to the atom domain of underlying data.
/// * `nullable` - Indicates if Series contains null values.
///
/// # Example
/// ```
/// use opendp::domains::AtomDomain;
/// use opendp::domains::SeriesDomain;
/// // Create a SeriesDomain with column `A` and `i32` AtomDomain.
/// let series = SeriesDomain::new("A", AtomDomain::<i32>::default());
/// // Create a SeriesDomain with column `B` and `f64` AtomDomain with bounds `[1.0, 2.0]`.
/// let series_with_bounds = SeriesDomain::new("B", AtomDomain::<f64>::new_closed((1.0, 2.0))?);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone)]
pub struct SeriesDomain {
    pub field: Field,
    pub element_domain: Arc<dyn DynSeriesAtomDomain>,
    pub nullable: bool,
}

impl core::cmp::PartialEq for SeriesDomain {
    fn eq(&self, other: &Self) -> bool {
        self.field.eq(&other.field) && self.element_domain.eq(&self.element_domain)
    }
}

impl Domain for SeriesDomain {
    type Carrier = Series;
    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        if &self.field != &*value.field() {
            return Ok(false);
        }

        macro_rules! atom_member {
            ($ty:ty, $polars_ty:ty) => {{
                let atom_domain = self.atom_domain::<<$ty as ToOwned>::Owned>()?;

                let chunked = value.0.unpack::<$polars_ty>()?;
                if !self.nullable && chunked.null_count() > 0 {
                    return Ok(false);
                }

                for arr in chunked.downcast_iter() {
                    for v in arr.values_iter() {
                        if !atom_domain.member(&v.to_owned())? {
                            return Ok(false);
                        }
                    }
                }
                Ok(true)
            }};
        }

        match self.field.dtype {
            DataType::UInt8 => atom_member!(u8, UInt8Type),
            DataType::UInt16 => atom_member!(u16, UInt16Type),
            DataType::UInt32 => atom_member!(u32, UInt32Type),
            DataType::UInt64 => atom_member!(u64, UInt64Type),
            DataType::Int8 => atom_member!(i8, Int8Type),
            DataType::Int16 => atom_member!(i16, Int16Type),
            DataType::Int32 => atom_member!(i32, Int32Type),
            DataType::Int64 => atom_member!(i64, Int64Type),
            DataType::Float32 => atom_member!(f32, Float32Type),
            DataType::Float64 => atom_member!(f64, Float64Type),
            DataType::Boolean => atom_member!(bool, BooleanType),
            DataType::String => atom_member!(str, StringType),
            _ => return fallible!(NotImplemented, "unsupported dtype: {:?}", self.field.dtype),
        }
    }
}

impl SeriesDomain {
    pub fn new<DA: 'static + SeriesAtomDomain>(name: &str, element_domain: DA) -> Self {
        SeriesDomain {
            field: Field::new(name, DA::Atom::dtype()),
            element_domain: Arc::new(element_domain.atom_domain()),
            nullable: DA::NULLABLE,
        }
    }

    /// Instantiates the broadest possible domain given the limited information available from a field.
    /// The data could have NaNs or nulls, and is not bounded.
    pub fn new_from_field(field: Field) -> Fallible<Self> {
        macro_rules! new_series_domain {
            ($ty:ty, $func:ident) => {
                SeriesDomain::new(
                    field.name.as_str(),
                    OptionDomain::new(AtomDomain::<$ty>::$func()),
                )
            };
        }

        Ok(match field.data_type() {
            DataType::Boolean => new_series_domain!(bool, default),
            DataType::UInt32 => new_series_domain!(u32, default),
            DataType::UInt64 => new_series_domain!(u64, default),
            DataType::Int8 => new_series_domain!(i8, default),
            DataType::Int16 => new_series_domain!(i16, default),
            DataType::Int32 => new_series_domain!(i32, default),
            DataType::Int64 => new_series_domain!(i64, default),
            DataType::Float32 => new_series_domain!(f64, new_nullable),
            DataType::Float64 => new_series_domain!(f64, new_nullable),
            DataType::String => new_series_domain!(String, default),
            dtype => return fallible!(MakeDomain, "unsupported type {}", dtype),
        })
    }

    pub fn drop_bounds(&mut self) -> Fallible<()> {
        macro_rules! drop_bounds {
            ($ty:ty) => {{
                let mut element_domain = (self.element_domain.as_any())
                    .downcast_ref::<AtomDomain<$ty>>()
                    .ok_or_else(|| err!(FailedFunction, "unrecognized element domain"))?
                    .clone();
                element_domain.bounds = None;
                self.element_domain = Arc::new(element_domain) as Arc<dyn DynSeriesAtomDomain>;
            }};
        }

        match self.field.dtype {
            DataType::UInt32 => drop_bounds!(u32),
            DataType::UInt64 => drop_bounds!(u64),
            DataType::Int8 => drop_bounds!(i8),
            DataType::Int16 => drop_bounds!(i16),
            DataType::Int32 => drop_bounds!(i32),
            DataType::Int64 => drop_bounds!(i64),
            DataType::Float32 => drop_bounds!(f32),
            DataType::Float64 => drop_bounds!(f64),
            _ => {
                return fallible!(
                    FailedFunction,
                    "cannot drop bounds on: {:?}",
                    self.field.dtype
                )
            }
        }
        Ok(())
    }

    pub fn atom_domain<T: 'static + CheckAtom>(&self) -> Fallible<&AtomDomain<T>> {
        (self.element_domain.as_any())
            .downcast_ref::<AtomDomain<T>>()
            .ok_or_else(|| err!(FailedCast, "domain downcast failed"))
    }
}

impl Debug for SeriesDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SeriesDomain(\"{}\", {})",
            self.field.name, self.field.dtype
        )
    }
}

impl<D: UnboundedMetric> MetricSpace for (SeriesDomain, D) {
    fn check_space(&self) -> Fallible<()> {
        Ok(())
    }
}

// BEGIN UTILITY TRAITS

/// Common trait for domains that can be used to describe the space of typed elements within a series.
pub trait SeriesAtomDomain: Domain + Send + Sync {
    type Atom: CheckAtom + PrimitiveDataType;
    fn atom_domain(self) -> AtomDomain<Self::Atom>;
    const NULLABLE: bool;
}
impl<T: CheckAtom + PrimitiveDataType> SeriesAtomDomain for AtomDomain<T> {
    type Atom = T;

    fn atom_domain(self) -> AtomDomain<Self::Atom> {
        self
    }

    const NULLABLE: bool = false;
}
impl<T: CheckAtom + PrimitiveDataType> SeriesAtomDomain for OptionDomain<AtomDomain<T>> {
    type Atom = T;

    fn atom_domain(self) -> AtomDomain<Self::Atom> {
        self.element_domain
    }

    const NULLABLE: bool = true;
}

/// Object-safe version of SeriesAtomDomain.
pub trait DynSeriesAtomDomain: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn dyn_partial_eq(&self, other: &dyn DynSeriesAtomDomain) -> bool;
}
impl<D: 'static + SeriesAtomDomain> DynSeriesAtomDomain for D {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn dyn_partial_eq(&self, other: &dyn DynSeriesAtomDomain) -> bool {
        (other.as_any().downcast_ref::<D>()).map_or(false, |a| self == a)
    }
}

impl PartialEq for dyn DynSeriesAtomDomain + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_partial_eq(other)
    }
}

/// Utility trait to construct the Polars runtime data-type indicator from an atomic type.
pub trait NumericDataType:
    NumericNative<PolarsType = Self::NumericPolars> + PrimitiveDataType + Literal
{
    type NumericPolars: PolarsDataType + PolarsNumericType<Native = Self>;
}

pub trait PrimitiveDataType: 'static + Send + Sync {
    type Polars: PolarsDataType;
    fn dtype() -> DataType {
        Self::Polars::get_dtype()
    }
}

macro_rules! impl_dtype_from {
    ($ty:ty, $dt:ty) => {
        impl NumericDataType for $ty {
            type NumericPolars = $dt;
        }
        impl PrimitiveDataType for $ty {
            type Polars = $dt;
        }
    };
}
impl_dtype_from!(u32, UInt32Type);
impl_dtype_from!(u64, UInt64Type);
impl_dtype_from!(i8, Int8Type);
impl_dtype_from!(i16, Int16Type);
impl_dtype_from!(i32, Int32Type);
impl_dtype_from!(i64, Int64Type);
impl_dtype_from!(f32, Float32Type);
impl_dtype_from!(f64, Float64Type);
impl PrimitiveDataType for bool {
    type Polars = BooleanType;
}
impl PrimitiveDataType for String {
    type Polars = StringType;
}
#[cfg(test)]
mod test;

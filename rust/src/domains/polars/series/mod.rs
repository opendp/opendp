use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use crate::core::MetricSpace;
use crate::error::Fallible;
use crate::transformations::traits::UnboundedMetric;
use crate::{core::Domain, traits::CheckAtom};

use chrono::{NaiveDate, NaiveTime};
use polars::prelude::*;

use crate::domains::{AtomDomain, CategoricalDomain, DatetimeDomain, OptionDomain};

use super::EnumDomain;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// # Proof Definition
/// `SeriesDomain` describes a set of polars `Series`.
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
    /// The name of series in the domain.
    pub name: PlSmallStr,
    /// Domain of each element in the series.
    pub element_domain: Arc<dyn DynSeriesElementDomain>,
    /// Indicates if elements can be null.
    pub nullable: bool,
}

impl PartialEq for SeriesDomain {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.element_domain.eq(&other.element_domain)
            && self.nullable == other.nullable
    }
}

impl Domain for SeriesDomain {
    type Carrier = Series;
    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        if &self.name != &value.name() {
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

        match self.dtype() {
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
            _ => return fallible!(NotImplemented, "unsupported dtype: {:?}", self.dtype()),
        }
    }
}

impl SeriesDomain {
    /// # Proof Definition
    /// Returns a series domain spanning all series whose name is `name`
    /// and elements of the series are members of `element_domain`.
    pub fn new<S: Into<PlSmallStr>, DA: 'static + SeriesElementDomain>(
        name: S,
        element_domain: DA,
    ) -> Self {
        SeriesDomain {
            name: name.into(),
            element_domain: Arc::new(element_domain.inner_domain().clone()),
            nullable: DA::NULLABLE,
        }
    }

    /// # Proof Definition
    /// Returns the datatype of rows in members of `self`.
    pub fn dtype(&self) -> DataType {
        self.element_domain.dtype()
    }

    /// # Proof Definition
    /// Modifies `self` such that rows of members are members of `element_domain`.
    pub fn set_element_domain<DA: 'static + SeriesElementDomain<InnerDomain = DA>>(
        &mut self,
        element_domain: DA,
    ) {
        self.element_domain = Arc::new(element_domain);
    }

    fn new_element_domain(dtype: DataType) -> Fallible<Arc<dyn DynSeriesElementDomain>> {
        Ok(match dtype {
            DataType::Boolean => Arc::new(AtomDomain::<bool>::default()),
            DataType::UInt32 => Arc::new(AtomDomain::<u32>::default()),
            DataType::UInt64 => Arc::new(AtomDomain::<u64>::default()),
            DataType::Int8 => Arc::new(AtomDomain::<i8>::default()),
            DataType::Int16 => Arc::new(AtomDomain::<i16>::default()),
            DataType::Int32 => Arc::new(AtomDomain::<i32>::default()),
            DataType::Int64 => Arc::new(AtomDomain::<i64>::default()),
            DataType::Float32 => Arc::new(AtomDomain::<f64>::new_nullable()),
            DataType::Float64 => Arc::new(AtomDomain::<f64>::new_nullable()),
            DataType::String => Arc::new(AtomDomain::<String>::default()),
            DataType::Date => Arc::new(AtomDomain::<NaiveDate>::default()),
            DataType::Datetime(time_unit, time_zone) => Arc::new(DatetimeDomain {
                time_unit,
                time_zone,
            }),
            DataType::Time => Arc::new(AtomDomain::<NaiveTime>::default()),
            DataType::Categorical(_, _) => Arc::new(CategoricalDomain::default()),
            DataType::Enum(mapping, _) => {
                let mapping =
                    mapping.ok_or_else(|| err!(MakeDomain, "EnumDomain requires a mapping"))?;
                let categories = ChunkedArray::<StringType>::from(mapping.get_categories().clone())
                    .into_series();
                Arc::new(EnumDomain::new(categories)?)
            }
            dtype => return fallible!(MakeDomain, "unsupported type {}", dtype),
        })
    }

    pub fn set_dtype(&mut self, dtype: DataType) -> Fallible<()> {
        self.element_domain = Self::new_element_domain(dtype)?;
        Ok(())
    }
    /// Instantiates the broadest possible domain given the limited information available from a field.
    /// The data could have NaNs or nulls, and is not bounded.
    ///
    /// # Proof Definition
    /// Returns a series domain spanning all series
    /// whose name and data type of elements are specified by `field`.
    pub fn new_from_field(field: Field) -> Fallible<Self> {
        Ok(SeriesDomain {
            name: field.name,
            element_domain: Self::new_element_domain(field.dtype)?,
            nullable: true,
        })
    }

    /// # Proof Definition
    /// Removes the bounds domain descriptor from `self`,
    /// and returns an error if the type of elements is not a recognized type.
    pub fn drop_bounds(&mut self) -> Fallible<()> {
        macro_rules! drop_bounds {
            ($ty:ty) => {{
                let mut element_domain = (self.element_domain.as_any())
                    .downcast_ref::<AtomDomain<$ty>>()
                    .ok_or_else(|| {
                        err!(
                            FailedFunction,
                            "unrecognized element domain. Expected AtomDomain<{}>",
                            stringify!($ty)
                        )
                    })?
                    .clone();
                element_domain.bounds = None;
                self.element_domain = Arc::new(element_domain) as Arc<dyn DynSeriesElementDomain>;
            }};
        }

        match self.dtype() {
            DataType::UInt32 => drop_bounds!(u32),
            DataType::UInt64 => drop_bounds!(u64),
            DataType::Int8 => drop_bounds!(i8),
            DataType::Int16 => drop_bounds!(i16),
            DataType::Int32 => drop_bounds!(i32),
            DataType::Int64 => drop_bounds!(i64),
            DataType::Float32 => drop_bounds!(f32),
            DataType::Float64 => drop_bounds!(f64),
            _ => return fallible!(FailedFunction, "cannot drop bounds on: {:?}", self.dtype()),
        }
        Ok(())
    }

    /// # Proof Definition
    /// If the domain of elements is of type `AtomDomain<T>`, then returns the domain as that type,
    /// otherwise returns an error.
    pub fn atom_domain<T: 'static + CheckAtom>(&self) -> Fallible<&AtomDomain<T>> {
        self.element_domain::<AtomDomain<T>>()
    }

    /// # Proof Definition
    /// If the domain of elements is of type `D`, then returns the domain as that type,
    /// otherwise returns an error.
    pub fn element_domain<D: 'static>(&self) -> Fallible<&D> {
        (self.element_domain.as_any())
            .downcast_ref::<D>()
            .ok_or_else(|| err!(FailedCast, "domain downcast failed"))
    }

    pub fn set_non_nan(&mut self) -> Fallible<()> {
        match self.dtype() {
            DataType::Float64 => {
                let atom_domain = self.atom_domain::<f64>()?.clone();
                self.set_element_domain(AtomDomain::<f64>::new(atom_domain.bounds, None));
            }
            DataType::Float32 => {
                let atom_domain = self.atom_domain::<f32>()?.clone();
                self.set_element_domain(AtomDomain::<f32>::new(atom_domain.bounds, None));
            }
            _ => {
                return fallible!(
                    MakeTransformation,
                    "only floating point types can be made non-NaN"
                )
            }
        }
        Ok(())
    }
}

impl Debug for SeriesDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SeriesDomain(\"{}\", {})", self.name, self.dtype())
    }
}

impl<D: UnboundedMetric> MetricSpace for (SeriesDomain, D) {
    fn check_space(&self) -> Fallible<()> {
        Ok(())
    }
}

// BEGIN UTILITY TRAITS

/// Common trait for domains that can be used to describe the space of typed elements within a series.
pub trait SeriesElementDomain: Domain + Send + Sync {
    type InnerDomain: SeriesElementDomain<InnerDomain = Self::InnerDomain>;
    /// # Proof Definition
    /// Returns the [`DataType`] of elements in the series.
    fn dtype(&self) -> DataType;

    /// # Proof Definition
    /// Returns the domain elements in the physical backing store.
    ///
    /// Polars Series represents nullity via a separate validity bit vector,
    /// so that non-null data can be stored contiguously.
    /// This function returns specifically the domain of non-null elements.
    fn inner_domain(&self) -> &Self::InnerDomain;

    /// # Proof Definition
    /// True if Series domains may contain null elements, otherwise False.
    const NULLABLE: bool;
}
impl<T: CheckAtom + PrimitiveDataType> SeriesElementDomain for AtomDomain<T> {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        T::dtype()
    }
    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}
impl<D: SeriesElementDomain<InnerDomain = D>> SeriesElementDomain for OptionDomain<D> {
    type InnerDomain = D;

    fn dtype(&self) -> DataType {
        self.inner_domain().dtype()
    }
    fn inner_domain(&self) -> &D {
        &self.element_domain
    }

    const NULLABLE: bool = true;
}

impl SeriesElementDomain for CategoricalDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        DataType::Categorical(None, Default::default())
    }
    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}

impl SeriesElementDomain for EnumDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        DataType::Enum(None, Default::default())
    }
    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}

impl SeriesElementDomain for DatetimeDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        DataType::Datetime(self.time_unit.clone(), self.time_zone.clone())
    }
    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}

/// Object-safe version of [`SeriesElementDomain`].
pub trait DynSeriesElementDomain: 'static + Send + Sync {
    /// # Proof Definition
    /// Returns the datatype of rows of members in the domain.
    fn dtype(&self) -> DataType;

    /// This method makes it possible to downcast a trait object of Self
    /// (dyn DynSeriesElementDomain) to its concrete type.
    ///
    /// # Proof Definition
    /// Return a reference to `self` as an Any trait object.
    fn as_any(&self) -> &dyn Any;

    /// # Proof Definition
    /// Returns true if `self` and `other` are equal.
    ///
    /// This is used to check if two [`SeriesDomain`] are equal,
    /// because series domain holds a `Box<dyn DynSeriesAtomDomain>`.
    fn dyn_partial_eq(&self, other: &dyn DynSeriesElementDomain) -> bool;
}
impl<D: 'static + SeriesElementDomain> DynSeriesElementDomain for D {
    fn dtype(&self) -> DataType {
        D::dtype(&self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn dyn_partial_eq(&self, other: &dyn DynSeriesElementDomain) -> bool {
        (other.as_any().downcast_ref::<D>()).map_or(false, |a| self == a)
    }
}

impl PartialEq for dyn DynSeriesElementDomain + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_partial_eq(other)
    }
}

/// Utility trait to construct the Polars runtime data-type indicator from an atomic type.
pub trait NumericDataType:
    NumericNative<PolarsType = Self::NumericPolars> + PrimitiveDataType + Literal
{
    /// Polars has defined its own marker types for elementary data types.
    ///
    /// # Proof Definition
    /// `NumericPolars` is the Polars marker type that corresponds to `Self`.
    type NumericPolars: PolarsDataType + PolarsNumericType<Native = Self>;
}

pub trait PrimitiveDataType: 'static + Send + Sync {
    /// Polars has defined its own marker types for elementary data types.
    ///
    /// # Proof Definition
    /// `Polars` is the Polars marker type that corresponds to `Self`.
    type Polars: PolarsDataType;

    /// # Proof Definition
    /// Return an instance of the DataType enum of the variant that corresponds to `Self`.
    ///
    /// A default implementation is provided because Polars already implements this on the marker type (Self::Polars).
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
impl PrimitiveDataType for NaiveDate {
    type Polars = DateType;
}
impl PrimitiveDataType for NaiveTime {
    type Polars = TimeType;
}

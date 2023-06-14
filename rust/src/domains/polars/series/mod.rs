use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

use crate::error::Fallible;
use crate::{core::Domain, traits::CheckAtom};

use polars::prelude::*;

use crate::domains::{AtomDomain, OptionDomain};

#[cfg(feature = "ffi")]
mod ffi;

#[derive(Clone)]
pub struct SeriesDomain {
    pub field: Field,
    pub element_domain: Rc<dyn DynSeriesAtomDomain>,
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
                let atom_domain = self
                    .element_domain
                    .as_any()
                    .downcast_ref::<AtomDomain<<$ty as ToOwned>::Owned>>()
                    .ok_or_else(|| err!(FailedCast, "domain downcast failed"))?;

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
            DataType::Utf8 => atom_member!(str, Utf8Type),
            _ => return fallible!(NotImplemented, "unsupported dtype: {:?}", self.field.dtype),
        }
    }
}

impl SeriesDomain {
    pub fn new<DA: 'static + SeriesAtomDomain>(name: &str, element_domain: DA) -> Self {
        SeriesDomain {
            field: Field::new(name, DA::Atom::dtype()),
            element_domain: Rc::new(element_domain.atom_domain()),
            nullable: DA::NULLABLE,
        }
    }
}

impl Debug for SeriesDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
<<<<<<< HEAD
        write!(f, "SeriesDomain(\"{}\", {})", self.field.name, self.field.dtype)
=======
        write!(
            f,
            "SeriesDomain(\"{}\", {})",
            self.field.name, self.field.dtype
        )
>>>>>>> remotes/origin/773-sum-metrics
    }
}

// BEGIN UTILITY TRAITS

/// Common trait for domains that can be used to describe the space of typed elements within a series.
pub trait SeriesAtomDomain: Domain {
    type Atom: CheckAtom + DataTypeFrom;
    fn atom_domain(self) -> AtomDomain<Self::Atom>;
    const NULLABLE: bool;
}
impl<T: CheckAtom + DataTypeFrom> SeriesAtomDomain for AtomDomain<T> {
    type Atom = T;

    fn atom_domain(self) -> AtomDomain<Self::Atom> {
        self
    }

    const NULLABLE: bool = false;
}
impl<T: CheckAtom + DataTypeFrom> SeriesAtomDomain for OptionDomain<AtomDomain<T>> {
    type Atom = T;

    fn atom_domain(self) -> AtomDomain<Self::Atom> {
        self.element_domain
    }

    const NULLABLE: bool = true;
}

/// Object-safe version of SeriesAtomDomain.
pub trait DynSeriesAtomDomain {
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
pub trait DataTypeFrom {
    fn dtype() -> DataType;
}

macro_rules! impl_dtype_from {
    ($ty:ty, $dt:expr) => {
        impl DataTypeFrom for $ty {
            fn dtype() -> DataType {
                $dt
            }
        }
    };
}
impl_dtype_from!(u8, DataType::UInt8);
impl_dtype_from!(u16, DataType::UInt16);
impl_dtype_from!(u32, DataType::UInt32);
impl_dtype_from!(u64, DataType::UInt64);
impl_dtype_from!(i8, DataType::Int8);
impl_dtype_from!(i16, DataType::Int16);
impl_dtype_from!(i32, DataType::Int32);
impl_dtype_from!(i64, DataType::Int64);
impl_dtype_from!(f32, DataType::Float32);
impl_dtype_from!(f64, DataType::Float64);
impl_dtype_from!(bool, DataType::Boolean);
impl_dtype_from!(String, DataType::Utf8);

<<<<<<< HEAD


=======
>>>>>>> remotes/origin/773-sum-metrics
#[cfg(test)]
mod test_series {
    use crate::domains::OptionDomain;

    use super::*;

    #[test]
    fn test_series_new() -> Fallible<()> {
        let series_domain = SeriesDomain::new("A", AtomDomain::<bool>::default());

        let series = Series::new("A", vec![true; 50]);
        assert!(series_domain.member(&series)?);
        assert!(series_domain == series_domain);
        Ok(())
    }

    #[test]
    fn test_series_bounded() -> Fallible<()> {
        let series_domain = SeriesDomain::new("A", AtomDomain::new_closed((1, 3))?);

        let series = Series::new("A", vec![1; 50]);
        assert!(series_domain.member(&series)?);

        let series = Series::new("A", vec![4; 50]);
        assert!(!series_domain.member(&series)?);

        Ok(())
    }

    #[test]
    fn test_series_nullable() -> Fallible<()> {
        // option domain with non-nullable type
        let series_domain =
            SeriesDomain::new("A", OptionDomain::new(AtomDomain::<bool>::default()));
        {
            let series = Series::new("A", vec![Some(true), Some(false), None]);
            assert!(series_domain.member(&series)?);
        }

        // nullable type without options
        let series_domain = SeriesDomain::new("A", AtomDomain::<f64>::new_nullable());
        {
            // None is not ok, but NaN is ok
            let series = Series::new("A", vec![Some(1.), Some(f64::NAN), None]);
            assert!(!series_domain.member(&series)?);
            // series made with Option::Some are ok
            let series = Series::new("A", vec![Some(1.), Some(f64::NAN)]);
            assert!(series_domain.member(&series)?);
            // series made without options are ok
            let series = Series::new("A", vec![1., f64::NAN]);
            assert!(series_domain.member(&series)?);
        }

        // permit both kinds of nullity
        let series_domain =
            SeriesDomain::new("A", OptionDomain::new(AtomDomain::<f64>::new_nullable()));
        {
            // None and NaN are both ok
            let series = Series::new("A", vec![Some(1.), Some(f64::NAN), None]);
            assert!(series_domain.member(&series)?);
            // doesn't have to have NaN
            let series = Series::new("A", vec![1., 2.]);
            assert!(series_domain.member(&series)?);
        }

        Ok(())
    }
}

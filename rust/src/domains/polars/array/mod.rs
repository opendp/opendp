use std::sync::Arc;

use polars::{prelude::DataType, series::Series};

use crate::{core::Domain, error::Fallible};

use super::{DynSeriesElementDomain, SeriesElementDomain};

#[cfg(feature = "ffi")]
mod ffi;

/// A domain that represents array data in Polars.
///
/// This is a domain that represents a fixed-size array of elements, where each element is
/// a member of `element_domain`.
#[derive(Clone, Debug)]
pub struct ArrayDomain {
    /// Domain of each element in the array.
    pub element_domain: Arc<dyn DynSeriesElementDomain>,
    /// Size of the array.
    pub size: usize,
    /// Whether elements within an array may be null.
    pub nullable: bool,
}

impl PartialEq for ArrayDomain {
    fn eq(&self, other: &Self) -> bool {
        self.element_domain.eq(&other.element_domain)
            && self.size == other.size
            && self.nullable == other.nullable
    }
}

impl ArrayDomain {
    pub fn new(element_domain: impl DynSeriesElementDomain, size: usize) -> Self {
        ArrayDomain {
            element_domain: Arc::new(element_domain),
            size,
            nullable: true,
        }
    }

    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    // The caller has already checked the element dtype and array width.
    fn check_elements(&self, elements: &Series) -> Fallible<bool> {
        if !self.nullable && elements.null_count() > 0 {
            return Ok(false);
        }
        self.element_domain.check_values(elements)
    }
}

impl Domain for ArrayDomain {
    type Carrier = Series;

    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        if value.len() != self.size || !self.element_domain.matches_dtype(value.dtype()) {
            return Ok(false);
        }
        self.check_elements(value)
    }
}

impl SeriesElementDomain for ArrayDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        DataType::Array(Box::new(self.element_domain.dtype()), self.size)
    }

    fn matches_dtype(&self, dtype: &DataType) -> bool {
        match dtype {
            DataType::Array(inner, size) => {
                *size == self.size && self.element_domain.matches_dtype(inner)
            }
            _ => false,
        }
    }

    fn check_values(&self, series: &Series) -> Fallible<bool> {
        for elements in series.array()?.into_iter().flatten() {
            if !self.check_elements(&elements)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}

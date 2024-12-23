use std::sync::Arc;

use polars::series::Series;

use crate::{core::Domain, error::Fallible};

use super::DynSeriesElementDomain;

#[cfg(feature = "ffi")]
mod ffi;

/// A domain that represents enum data.
///
/// Enum data is ostensibly a string,
/// however the data is stored as a vector of indices into an encoding.
/// This gives memory speedups when the number of unique values is small.
///
/// Differs from the ArrayDomain in that the categories are fixed and known.
#[derive(Clone, Debug)]
pub struct ArrayDomain {
    /// Domain of each element in the array.
    pub element_domain: Arc<dyn DynSeriesElementDomain>,
    /// Length of the array.
    pub width: usize,
}

impl PartialEq for ArrayDomain {
    fn eq(&self, other: &Self) -> bool {
        self.element_domain == other.element_domain.clone() && self.width == other.width
    }
}

impl ArrayDomain {
    pub fn new(element_domain: impl DynSeriesElementDomain, width: usize) -> Self {
        ArrayDomain {
            element_domain: Arc::new(element_domain),
            width,
        }
    }
}

impl Domain for ArrayDomain {
    /// This domain is used in conjunction with another domain, like Polars SeriesDomain,
    /// where the carrier type reflects the encoding used to efficiently store categorical data.
    type Carrier = Series;

    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        if value.len() != self.width {
            return fallible!(MakeDomain, "Array length does not match domain length");
        }
        Ok(true)
    }
}

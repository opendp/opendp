use std::sync::Arc;

use polars::series::Series;

use crate::{core::Domain, error::Fallible};

use super::DynSeriesElementDomain;

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
}

impl PartialEq for ArrayDomain {
    fn eq(&self, other: &Self) -> bool {
        self.element_domain == other.element_domain.clone() && self.size == other.size
    }
}

impl ArrayDomain {
    pub fn new(element_domain: impl DynSeriesElementDomain, size: usize) -> Self {
        ArrayDomain {
            element_domain: Arc::new(element_domain),
            size,
        }
    }

    /// True when an Array contains nullable elements at any nesting depth.
    pub(crate) fn has_nullable_elements(&self) -> bool {
        self.element_domain.is_nullable()
            || self
                .element_domain
                .as_any()
                .downcast_ref::<ArrayDomain>()
                .is_some_and(ArrayDomain::has_nullable_elements)
    }
}

impl Domain for ArrayDomain {
    type Carrier = Series;

    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        if value.len() != self.size {
            return fallible!(MakeDomain, "Array length does not match domain length");
        }
        Ok(true)
    }
}

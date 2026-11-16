use std::sync::Arc;

use polars::series::Series;

use crate::{core::Domain, error::Fallible};

use super::{DynSeriesElementDomain, series::member_elements};

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
            return Ok(false);
        }
        member_elements(
            value.name(),
            self.element_domain.as_ref(),
            self.element_domain.is_nullable(),
            value,
        )
    }
}

#[cfg(test)]
mod test {
    use polars::prelude::{NamedFrom, Series};

    use crate::domains::{AtomDomain, OptionDomain};
    use crate::error::Fallible;

    use super::*;

    #[test]
    fn test_array_domain_member_checks_elements() -> Fallible<()> {
        let domain = ArrayDomain::new(AtomDomain::<i32>::new_closed((0, 10))?, 2);

        assert!(domain.member(&Series::new("values".into(), &[1i32, 10]))?);
        assert!(!domain.member(&Series::new("values".into(), &[1i32, 11]))?);
        assert!(!domain.member(&Series::new("values".into(), &[1i64, 2]))?);
        assert!(!domain.member(&Series::new("values".into(), &[1i32]))?);
        assert!(!domain.member(&Series::new("values".into(), &[Some(1i32), None]))?);

        Ok(())
    }

    #[test]
    fn test_array_domain_member_nullable_elements() -> Fallible<()> {
        let domain = ArrayDomain::new(
            OptionDomain::new(AtomDomain::<i32>::new_closed((0, 10))?),
            2,
        );
        assert!(domain.has_nullable_elements());
        assert!(domain.member(&Series::new("values".into(), &[Some(1i32), None]))?);
        assert!(!domain.member(&Series::new("values".into(), &[Some(11i32), None]))?);

        Ok(())
    }
}

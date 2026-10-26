use polars::prelude::PlSmallStr;

use crate::{core::Domain, error::Fallible};

#[cfg(feature = "ffi")]
mod ffi;

/// A domain that represents categorical data.
///
/// Categorical data is ostensibly a string,
/// however the data is stored as a vector of indices into an encoding.
/// The mapping between categories and indices is not fixed.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CategoricalDomain;

impl Domain for CategoricalDomain {
    type Carrier = PlSmallStr;

    fn member(&self, _value: &Self::Carrier) -> Fallible<bool> {
        Ok(true)
    }
}

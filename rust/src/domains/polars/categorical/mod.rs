use polars::prelude::{Categories, DataType, PlSmallStr, Series};

use crate::{core::Domain, error::Fallible};

use super::SeriesElementDomain;

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

impl SeriesElementDomain for CategoricalDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        let categories = Categories::global();
        DataType::Categorical(categories.clone(), categories.mapping())
    }

    fn matches_dtype(&self, dtype: &DataType) -> bool {
        matches!(dtype, DataType::Categorical(_, _))
    }

    fn check_values(&self, _: &Series) -> Fallible<bool> {
        Ok(true)
    }

    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}

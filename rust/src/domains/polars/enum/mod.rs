use std::sync::Arc;

use polars::{
    prelude::{DataType, FrozenCategories, PlSmallStr},
    series::Series,
};

use crate::{core::Domain, error::Fallible};

use super::SeriesElementDomain;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// A domain that represents enum data.
///
/// Enum data is ostensibly a string,
/// however the data is stored as a vector of indices into an encoding.
/// This gives memory speedups when the number of unique values is small.
///
/// Differs from the CategoricalDomain in that the categories are fixed and known.
#[derive(Debug, Clone)]
pub struct EnumDomain {
    /// The encoding used to assign numerical indices to each known possible category.
    categories: Arc<FrozenCategories>,
}

impl EnumDomain {
    pub fn new(categories: Series) -> Fallible<Self> {
        if !categories.dtype().is_string() {
            return fallible!(
                MakeDomain,
                "categories dtype ({}) must be string",
                categories.dtype()
            );
        }

        if categories.null_count() > 0 {
            return fallible!(MakeDomain, "categories must not contain nulls");
        }

        let categories = FrozenCategories::new(categories.str()?.into_iter().flatten())?;
        Ok(Self::from(categories))
    }

    pub fn categories(&self) -> &Arc<FrozenCategories> {
        &self.categories
    }
}

impl From<Arc<FrozenCategories>> for EnumDomain {
    fn from(categories: Arc<FrozenCategories>) -> Self {
        Self { categories }
    }
}

impl Default for EnumDomain {
    fn default() -> Self {
        Self::from(FrozenCategories::new([]).unwrap())
    }
}

impl PartialEq for EnumDomain {
    fn eq(&self, other: &Self) -> bool {
        // Polars interns ordered category sets.
        Arc::ptr_eq(&self.categories, &other.categories)
    }
}

impl Domain for EnumDomain {
    /// This domain is used in conjunction with another domain, like Polars SeriesDomain,
    /// where the carrier type reflects the encoding used to efficiently store categorical data.
    type Carrier = PlSmallStr;

    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        Ok(self.categories.mapping().get_cat(value.as_str()).is_some())
    }
}

impl SeriesElementDomain for EnumDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        DataType::Enum(self.categories.clone(), self.categories.mapping().clone())
    }

    fn check_values(&self, _: &Series) -> Fallible<bool> {
        // A matching Enum dtype already fixes its possible values and encoding.
        Ok(true)
    }

    fn inner_domain(&self) -> &Self {
        self
    }

    const NULLABLE: bool = false;
}

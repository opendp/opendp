use std::collections::HashSet;

use polars::prelude::PlSmallStr;

use crate::{core::Domain, error::Fallible};

/// A domain that represents categorical data.
///
/// Categorical data is ostensibly a string,
/// however the data is stored as a vector of indices into an encoding.
/// This gives memory speedups when the number of unique values is small.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CategoricalDomain {
    /// The encoding used to assign numerical indices to each known possible category.
    categories: Option<Vec<PlSmallStr>>,
}

impl CategoricalDomain {
    /// Only use this constructor if you know both the category set,
    /// as well as how categories are encoded as integers.
    ///
    /// Typically when categorical data is encoded,
    /// indices are assigned by the order encountered in the data, making the encoding data-dependent.
    ///
    /// An example where this can be happen is for categorical data emitted by the Polars cut expression,
    /// where the categories and encoding are pre-determined by the expression (the bin edges).
    pub fn new_with_categories(categories: Vec<PlSmallStr>) -> Fallible<Self> {
        if categories.len() != HashSet::<_>::from_iter(categories.iter()).len() {
            return fallible!(MakeDomain, "categories must be distinct");
        }
        Ok(CategoricalDomain {
            categories: Some(categories),
        })
    }

    pub fn categories(&self) -> Option<&Vec<PlSmallStr>> {
        self.categories.as_ref()
    }
}

impl Domain for CategoricalDomain {
    /// This domain is used in conjunction with another domain, like Polars SeriesDomain,
    /// where the carrier type reflects the encoding used to efficiently store categorical data.
    type Carrier = PlSmallStr;

    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        Ok(self
            .categories
            .as_ref()
            .map(|e| e.contains(value))
            .unwrap_or(true))
    }
}

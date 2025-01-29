use polars::{
    prelude::{is_in, NamedFrom, PlSmallStr},
    series::Series,
};

use crate::{core::Domain, error::Fallible};

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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnumDomain {
    /// The encoding used to assign numerical indices to each known possible category.
    categories: Series,
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

        let n_duplicates = categories.len() - categories.n_unique()?;
        if n_duplicates != 0 {
            return fallible!(
                MakeDomain,
                "categories must be distinct. Found {:?} duplicates.",
                n_duplicates
            );
        }
        Ok(EnumDomain { categories })
    }

    pub fn categories(&self) -> &Series {
        &self.categories
    }
}

impl Domain for EnumDomain {
    /// This domain is used in conjunction with another domain, like Polars SeriesDomain,
    /// where the carrier type reflects the encoding used to efficiently store categorical data.
    type Carrier = PlSmallStr;

    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        Ok(is_in(
            &self.categories,
            &Series::new("".into(), &vec![value.as_str()]),
        )?
        .get(0)
        .unwrap())
    }
}

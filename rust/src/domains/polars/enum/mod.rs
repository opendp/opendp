use polars::{
    prelude::{DataType, FrozenCategories, NamedFrom, PlSmallStr, is_in},
    series::{IntoSeries, Series},
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

        if categories.null_count() > 0 {
            return fallible!(MakeDomain, "categories must not contain nulls");
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
            &Series::new("".into(), &vec![value.as_str()])
                .implode()?
                .into_series(),
            false,
        )?
        .get(0)
        .unwrap())
    }
}

impl SeriesElementDomain for EnumDomain {
    type InnerDomain = Self;

    fn dtype(&self) -> DataType {
        let categories = self.categories().str().unwrap().iter().map(|v| v.unwrap());
        let categories = FrozenCategories::new(categories).unwrap();
        DataType::Enum(categories.clone(), categories.mapping().clone())
    }

    fn matches_dtype(&self, dtype: &DataType) -> bool {
        let DataType::Enum(categories, _) = dtype else {
            return false;
        };
        // Polars compares Enum categories by pointer identity. Physical indices
        // expose the category ordering, so compare the labels in order instead.
        self.categories().str().is_ok_and(|expected| {
            expected
                .into_iter()
                .flatten()
                .eq(categories.categories().non_null_values_iter())
        })
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

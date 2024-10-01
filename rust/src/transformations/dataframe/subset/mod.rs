use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    error::Fallible,
    metrics::SymmetricDistance,
    traits::Hashable,
};

use super::{DataFrame, DataFrameDomain};

#[cfg(feature = "ffi")]
mod ffi;

#[deprecated(note = "Use Polars instead", since = "0.12.0")]
#[bootstrap(features("contrib"))]
/// Make a Transformation that subsets a dataframe by a boolean column.
///
/// # Arguments
/// * `indicator_column` - name of the boolean column that indicates inclusion in the subset
/// * `keep_columns` - list of column names to apply subset to
///
/// # Generics
/// * `TK` - Type of the column name
pub fn make_subset_by<TK: Hashable>(
    indicator_column: TK,
    keep_columns: Vec<TK>,
) -> Fallible<
    Transformation<DataFrameDomain<TK>, DataFrameDomain<TK>, SymmetricDistance, SymmetricDistance>,
> {
    Transformation::new(
        DataFrameDomain::new(),
        DataFrameDomain::new(),
        Function::new_fallible(move |data: &DataFrame<TK>| {
            // the partition to move each row into
            let indicator = (data.get(&indicator_column))
                .ok_or_else(|| {
                    err!(
                        FailedFunction,
                        "{:?} does not exist in the input dataframe",
                        indicator_column
                    )
                })?
                .as_form::<Vec<bool>>()?;

            // where to collect partitioned data
            let mut subsetted = DataFrame::new();

            // iteratively partition each column
            keep_columns.iter().try_for_each(|column_name| {
                // retrieve a Column from the dataframe
                let column = data.get(&column_name).ok_or_else(|| {
                    err!(
                        FailedFunction,
                        "{:?} does not exist in the input dataframe",
                        column_name
                    )
                })?;

                subsetted.insert(column_name.clone(), column.subset(&indicator));

                Fallible::Ok(())
            })?;

            Ok(subsetted)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    )
}

#[cfg(test)]
mod test;

use crate::{
    core::{Function, StabilityMap, Transformation},
    error::Fallible,
    metrics::SymmetricDistance,
    traits::Hashable,
};

use super::{DataFrame, DataFrameDomain};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_filter_by<TK: Hashable>(
    identifier_column: TK,
    keep_columns: Vec<TK>,
) -> Fallible<
    Transformation<DataFrameDomain<TK>, DataFrameDomain<TK>, SymmetricDistance, SymmetricDistance>,
> {
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        DataFrameDomain::new_all(),
        Function::new_fallible(move |data: &DataFrame<TK>| {
            // the partition to move each row into
            let filter = (data.get(&identifier_column))
                .ok_or_else(|| err!(FailedFunction, "{:?} does not exist in the input dataframe"))?
                .as_form::<Vec<bool>>()?;

            // where to collect partitioned data
            let mut filtered = DataFrame::new();

            // iteratively partition each column
            keep_columns.iter().try_for_each(|column_name| {
                // retrieve a Column from the dataframe
                let column = data.get(&column_name).ok_or_else(|| {
                    err!(FailedFunction, "{:?} does not exist in the input dataframe")
                })?;

                filtered.insert(column_name.clone(), column.subset(&filter));

                Fallible::Ok(())
            })?;

            Ok(filtered)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    ))
}

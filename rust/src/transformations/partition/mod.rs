use std::collections::HashMap;

use opendp_derive::bootstrap;

use crate::{traits::Hashable, error::Fallible, core::{Transformation, Function, StabilityMap}, domains::ProductDomain, metrics::SymmetricDistance};

use super::{DataFrameDomain, DataFrame};

#[cfg(feature="ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    generics(TK(default = "String"), TV(example = "$get_first(partition_keys)"))
)]
/// Make a Transformation that partitions a dataframe by a given column.
/// 
/// # Arguments
/// * `identifier_column` - Name of column to split dataframe by.
/// * `partition_keys` - Unique values in the `identifier_column` column.
/// * `keep_columns` - Columns to keep in the partitioned dataframes.
/// 
/// # Generics
/// * `TK` - Type of column names.
/// * `TV` - Type of values in the identifier column.
pub fn make_partition_by<TK: Hashable, TV: Hashable>(
    identifier_column: TK,
    partition_keys: Vec<TV>,
    keep_columns: Vec<TK>
) -> Fallible<Transformation<DataFrameDomain<TK>, ProductDomain<DataFrameDomain<TK>>, SymmetricDistance, SymmetricDistance>> {
    let partition_indexes: HashMap<TV, usize> = partition_keys.iter().cloned().enumerate().map(|(i, k)| (k, i)).collect();
    let num_partitions = partition_keys.len() + 1;
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        ProductDomain::new((0..num_partitions).map(|_| DataFrameDomain::new_all()).collect()),
        Function::new_fallible(move |data: &DataFrame<TK>| {
            // the partition to move each row into
            let partition_ids: Vec<usize> = data.get(&identifier_column)
                .ok_or_else(|| err!(FailedFunction, "{:?} does not exist in the input dataframe"))?
                .as_form::<Vec<TV>>()?
                .iter()
                .map(|v| partition_indexes.get(v).cloned().unwrap_or(partition_keys.len()))
                .collect();

            // where to collect partitioned data
            let mut partitioned_data = std::vec::from_elem(DataFrame::new(), num_partitions);

            // iteratively partition each column
            keep_columns.iter().try_for_each(|column_name| {
                // retrieve a Column from the dataframe
                let column = data.get(&column_name)
                    .ok_or_else(|| err!(FailedFunction, "{:?} does not exist in the input dataframe"))?;

                // partition the column by the partition ids, 
                //    then insert each subset into the respective partitioned dataframe
                column.partition(&partition_ids, num_partitions)
                    .into_iter()
                    .zip(partitioned_data.iter_mut())
                    .for_each(|(subset, df)| {
                        df.insert(column_name.clone(), subset);
                    });
                
                Fallible::Ok(())
            })?;

            Ok(partitioned_data)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1)
    ))
}
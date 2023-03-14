use std::collections::HashMap;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::ProductDomain,
    error::Fallible,
    metrics::{SymmetricDistance, ProductMetric, IntDistance},
    traits::{Hashable, ExactIntCast},
};

use super::{DataFrame, DataFrameDomain, DataFrame2, DataFrameDomain2, SizedDataFrameDomain};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(null_partition(default = false)),
    generics(TK(default = "String"), TV(example = "$get_first(partition_keys)"))
)]
/// Make a Transformation that partitions a dataframe by a given column.
/// 
/// # Arguments
/// * `identifier_column` - Name of column to split dataframe by.
/// * `partition_keys` - Unique values in the `identifier_column` column.
/// * `keep_columns` - Columns to keep in the partitioned dataframes.
/// * `null_partition` - Whether to include a trailing null partition for rows that were not in the `partition_keys`
/// 
/// # Generics
/// * `TC` - Type of column names.
/// * `TK` - Type of values in the identifier column.
pub fn make_sized_partition_by<TC: Hashable, TK: Hashable>(
    identifier_column: TC,
    keep_columns: Vec<TC>,
    partition_number: usize,
    null_partition: bool,
) -> Fallible<
    Transformation<
    DataFrameDomain2<TC>,
        ProductDomain<DataFrameDomain2<TC>>,
        SymmetricDistance,
        ProductMetric<SymmetricDistance>,
    >,
> {
    let true_partitions = partition_number + 1;
    let output_partitions = partition_number + if null_partition { 1 } else { 0 };
    let d_output_partitions = IntDistance::exact_int_cast(output_partitions)?;

    // Create SizedDataFrameDomain with / without null partition
    //let df_domain = SizedDataFrameDomain::new(HashMap::from([(identifier_column, column_categories)]), HashMap::from([(identifier_column, colummn_counts)]));
    
    // Create Product<SizedDataFrameDomain>
    let product_df_domain  = ProductDomain::new(
        (0..output_partitions)
            .map(|v| DataFrameDomain2<TK>::new(true,true))
            .collect(),
    );

    Ok(Transformation::new(
        //df_domain,
        DataFrameDomain2<TK>::new(true,true),
        product_df_domain,
        Function::new_fallible(move |df: &DataFrame2<TC>| {
            let partition_indexes: HashMap<TK, usize> = df.categories_keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, k)| (k, i))
        .collect();

            // the partition to move each row into
            let partition_ids: Vec<usize> = (df.data.get(&identifier_column))
                .ok_or_else(|| err!(FailedFunction, "{:?} does not exist in the input dataframe"))?
                .as_form::<Vec<TV>>()?
                .iter()
                .map(|v| {
                    (partition_indexes.get(v))
                        .cloned()
                        .unwrap_or(df.categories_keys.len()) // Last index for unknown cat
                })
                .collect();

            // where to collect partitioned data
            let mut partitioned_data = std::vec::from_elem(DataFrame2::new(DataFrame::new()), true_partitions);

            // iteratively partition each column
            keep_columns.iter().try_for_each(|column_name| {
                // retrieve a Column from the dataframe
                let column = df.data.get(&column_name).ok_or_else(|| {
                    err!(FailedFunction, "{:?} does not exist in the input dataframe")
                })?;

                // partition the column by the partition ids,
                //    then insert each subset into the respective partitioned dataframe
                column
                    .partition(&partition_ids, true_partitions)
                    .into_iter()
                    .zip(partitioned_data.iter_mut())
                    .for_each(|(subset, df)| {
                        df.data.insert(column_name.clone(), subset);
                    });

                Fallible::Ok(())
            })?;

            if !null_partition {
                partitioned_data.pop(); // Remove last position if no unknown class
            }

            Ok(partitioned_data)
        }),
        SymmetricDistance::default(),
        ProductMetric::new(SymmetricDistance::default()),
        StabilityMap::new(move |d_in: &IntDistance| (*d_in, *d_in.min(&d_output_partitions))),
    ))
}

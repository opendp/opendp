use opendp_derive::bootstrap;

use std::collections::HashMap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::ProductDomain,
    error::Fallible,
    metrics::{SymmetricDistance, ProductMetric, IntDistance},
    traits::{Hashable, ExactIntCast},
    transformations::{DataFrame, SizedDataFrameDomain}
};

// #[cfg(feature = "ffi")]
// mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(null_partition(default = false)),
    generics(CA(default = "String"), TV(example = "$get_first(partition_keys)"))
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
pub fn make_sized_partition_by<TC: Hashable>(
    inputDomain: SizedDataFrameDomain<TC>,
    identifier_column: TC,
    keep_columns: Vec<TC>,
    null_partition: bool,
) -> Fallible<
    Transformation<
    SizedDataFrameDomain<TC>,
        ProductDomain<SizedDataFrameDomain<TC>>,
        SymmetricDistance,
        ProductMetric<SymmetricDistance>,
    >,
> {
    if !inputDomain.categories_keys.contains_key(&identifier_column){
        return fallible!(FailedFunction, "Data frame domain does not list the desired colunm as categorical variable.")
    }

    let partion_keys = inputDomain.categories_keys.get(&identifier_column).unwrap().as_any().downcast_ref::<Vec<TC>>().expect("Domain does not contain partion keys");
    let partion_size = partion_keys.len();
    let true_partitions = partion_size + 1;
    let output_partitions = partion_size + if null_partition { 1 } else { 0 };
    let d_output_partitions = IntDistance::exact_int_cast(output_partitions)?;

    // Create Product<SizedDataFrameDomain>
    let mut product_df_domain  = ProductDomain::new(
        (0..output_partitions)
            .map(|v| inputDomain.clone())
            .collect(),
    );
    (0..output_partitions)
            .map(|v| (0..partion_size).map( |d|
                 if d != v {
                    product_df_domain.inner_domains[v].categories_counts.get(&identifier_column).unwrap()[d] = 0;
                }));

    Ok(Transformation::new(
        inputDomain,
        product_df_domain,
        Function::new_fallible(move |data: &DataFrame<TC>| {
            let partition_indexes: HashMap<TC, usize> = partion_keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, k)| (k, i))
                .collect();

            // the partition to move each row into
            let partition_ids: Vec<usize> = (data.get(&identifier_column))
                .ok_or_else(|| err!(FailedFunction, "{:?} does not exist in the input dataframe"))?
                .as_form::<Vec<TC>>()?
                .iter()
                .map(|v| {
                    (partition_indexes.get(v))
                        .cloned()
                        .unwrap_or(data.len()) // Last index for unknown cat
                })
                .collect();

            // where to collect partitioned data
            let mut partitioned_data = std::vec::from_elem(DataFrame::new(), true_partitions);

            // iteratively partition each column
            keep_columns.iter().try_for_each(|column_name| {
                // retrieve a Column from the dataframe
                let column = data.get(&column_name).ok_or_else(|| {
                    err!(FailedFunction, "{:?} does not exist in the input dataframe")
                })?;

                // partition the column by the partition ids,
                //    then insert each subset into the respective partitioned dataframe
                column
                    .partition(&partition_ids, true_partitions)
                    .into_iter()
                    .zip(partitioned_data.iter_mut())
                    .for_each(|(subset, df)| {
                        data.insert(column_name.clone(), subset);
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


#[cfg(test)]
mod test {
    use crate::transformations::make_create_dataframe;

    use super::*;

    #[test]
    fn test_dataFrame_partition() -> Fallible<()> {

        let transformation = make_create_dataframe(vec!["colA", "colB"]).unwrap();

        let data_string = vec![
            vec!["1".to_owned(), "A".to_owned()],
            vec!["4".to_owned(), "A".to_owned()],
            vec!["2".to_owned(), "B".to_owned()],
            vec!["0".to_owned(), "A".to_owned()],
            vec!["0".to_owned(), "B".to_owned()],
        ];
        let df = transformation.invoke(&data_string).unwrap();

        println!("{:?}", df);

        Ok(())
    }
}
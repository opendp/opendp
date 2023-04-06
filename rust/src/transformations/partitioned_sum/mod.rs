//use opendp_derive::bootstrap;

use std::collections::HashMap;
use polars::prelude::*;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::ProductDomain,
    error::Fallible,
    metrics::{SymmetricDistance, ProductMetric, IntDistance},
    traits::{Hashable, ExactIntCast},
    //transformations::{SizedDataFrame},
    transformations::dataframe::SizedDataFrameDomain,
};

// #[cfg(feature = "ffi")]
// mod ffi;

// #[bootstrap(
//     features("contrib"),
//     arguments(null_partition(default = false)),
//     generics(TC(default = "String")) // NEED TO ADD GENERIC FOR SizedDataFrameDomain<TC>?
// )]
/// Make a Transformation that partitions a dataframe by a given column.
/// 
/// # Arguments
/// * `input_domain` - SizedDataFrameDomain with relevant categories and counts metadata.
/// * `identifier_column` - Name of column to split dataframe by.
/// * `keep_columns` - Columns to keep in the partitioned dataframes.
/// * `null_partition` - Whether to include a trailing null partition for rows that were not in the `partition_keys`
/// 
/// # Generics
/// * `TC` - Type of column names.
pub fn make_sized_partitioned_sum<CA: Hashable>(
    input_domain: SizedDataFrameDomain,
    identifier_column: &str,
    keep_columns: Vec<&str>,
    null_partition: bool,
) -> Fallible<
    Transformation<
    SizedDataFrameDomain,
        ProductDomain<SizedDataFrameDomain>,
        SymmetricDistance,
        ProductMetric<SymmetricDistance>,
    >,
> {
    if !input_domain.categories_counts.contains_key(&identifier_column){
        return fallible!(FailedFunction, "Data frame domain does not list the desired colunm as categorical variable.")
    }

    //Retreive series index
    let cat_column_index = input_domain.categories_keys.iter().position(|s| s.name() == identifier_column).expect("Domain does not contain partion keys").clone();

    // Create vector of partition keys
    let partion_keys: Vec<&str> = input_domain.categories_keys[cat_column_index].utf8().unwrap().into_no_null_iter().collect();
    let partition_indexes: HashMap<&str, usize> = partion_keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, k)| (k, i))
                .collect();


    // Defined output sizes 
    let partion_size = partion_keys.len();
    let true_partitions = partion_size + 1;
    let output_partitions = partion_size + if null_partition { 1 } else { 0 };
    let d_output_partitions = IntDistance::exact_int_cast(output_partitions)?;

    // Create Product<SizedDataFrameDomain>
    let mut vector_partition_domains = vec![SizedDataFrameDomain::Default(); output_partitions];
    for i in 0..output_partitions {
        let mut partition_counts: Vec<usize> = vec![0; partion_size];
        if i < partion_size {
        partition_counts[i] = input_domain.categories_counts.get(&identifier_column).unwrap()[i].clone();
        }
        vector_partition_domains[i].add_categorical_colunm(&input_domain.categories_keys[cat_column_index].clone(), partition_counts).unwrap();
    }
    let product_df_domain = ProductDomain::new(vector_partition_domains);

    Ok(Transformation::new(
        input_domain,
        product_df_domain,
        Function::new_fallible(move |data: &DataFrame| {
            
            // Check if col names exists

            // Turn partition column into categorical

            // Create partition
            let mut sums = data.clone()
                        .lazy()
                        .groupby([&identifier_column.to_string()])
                        .agg([sum(&sum_column.to_string())])
                        .collect()?;



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
    use crate::{transformations::make_create_dataframe, error::ExplainUnwrap};

    use super::*;

    #[test]
    fn test_dataFrame_partition() -> Fallible<()> {

        let transformation = make_create_dataframe(vec!["colA", "colB"]).unwrap();

        let data_string = vec![
            vec!["1".to_owned(), "A".to_owned()],
            vec!["4".to_owned(), "A".to_owned()],
            vec!["2".to_owned(), "B".to_owned()],
            vec!["0".to_owned(), "A".to_owned()],
            vec!["10".to_owned(), "B".to_owned()],
        ];
        //let df = transformation.invoke(&data_string).unwrap();

        //let df_domain = SizedDataFrameDomain::create_categorical_df_domain("colB", vec!["A","B"], vec![3,2]).unwrap();

        //println!("{:?}", df);

        //let partitioner = make_sized_partition_by(df_domain, "colB", vec!["colA"], false).unwrap();

        //let partition = partitioner.invoke(&df).unwrap();

        //println!("{:?}", partition);
        Ok(())
    }
}
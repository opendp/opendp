/* //use opendp_derive::bootstrap;

use std::collections::HashMap;
use polars::prelude::*;

use crate::{
    core::{Function, StabilityMap, Transformation},
    error::Fallible,
    metrics::{SymmetricDistance, IntDistance},
    traits::{Hashable, ExactIntCast},
    //transformations::{SizedDataFrame},
};
use crate::domains::{VectorDomain};

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
pub fn make_sized_partitioned_sum<TC: Hashable>(
    input_domain: SizedDataFrameDomain,
    partition_column: &str,
    sum_column: &str,
    _bounds: (f64, f64), // TO BE CHECKED
    null_partition: bool,
) -> Fallible<
    Transformation<
    SizedDataFrameDomain,
    VectorDomain<AllDomain<f64>>,
        SymmetricDistance,
        ProductMetric<SymmetricDistance>,
    >,
> {
    if !input_domain.categories_counts.contains_key(&partition_column){
        return fallible!(FailedFunction, "Data frame domain does not list the desired colunm as categorical variable.")
    }

    let partition_column_name = partition_column.to_string();
    let sum_column_name = sum_column.to_string();

    //Retreive series index
    let cat_column_index = input_domain.categories_keys.iter().position(|s| s.name() == partition_column_name).expect("Domain does not contain partion keys").clone();

    // Create vector of partition keys
    let partion_keys: Vec<&str> = input_domain.categories_keys[cat_column_index].utf8().unwrap().into_no_null_iter().collect();
    
    // Defined output sizes 
    let partion_size = partion_keys.len();
    let output_partitions = partion_size + if null_partition { 1 } else { 0 };
    let d_output_partitions = IntDistance::exact_int_cast(output_partitions)?;

    // Create Product<SizedDataFrameDomain>
    // let mut vector_partition_domains = vec![SizedDataFrameDomain::Default(); output_partitions];
    // for i in 0..output_partitions {
    //     let mut partition_counts: Vec<usize> = vec![0; partion_size];
    //     if i < partion_size {
    //     partition_counts[i] = input_domain.categories_counts.get(&partition_column_name).unwrap()[i].clone();
    //     }
    //     vector_partition_domains[i].add_categorical_colunm(&input_domain.categories_keys[cat_column_index].clone(), partition_counts).unwrap();
    // }
    //let product_df_domain = ProductDomain::new(vector_partition_domains);

    Ok(Transformation::new(
        input_domain,
        VectorDomain::new(AllDomain::new()),
        Function::new_fallible(move |data: &DataFrame| {
            
            // Check if col names exists
            if !data.get_column_names().contains(&partition_column_name.as_str()) {
                return fallible!(FailedFunction, "Dataframe does not contains the desired partition colunm.")
            }
            if !data.get_column_names().contains(&sum_column_name.as_str()) {
                return fallible!(FailedFunction, "Dataframe does not contains the column to be summed.")
            }

            // Create partition
            let mut cat_sums = data.clone()
                        .lazy()
                        .groupby([partition_column_name.as_str()])
                        .agg([sum(&sum_column_name.as_str())])
                        .collect()
                        .unwrap();


            // Remove unknown classes
            let sums = cat_sums.filter(
                &cat_sums
                .column(&partition_column_name.as_str())
                .unwrap()
                .is_in(&input_domain.categories_keys[cat_column_index])
                .unwrap())
                .unwrap();

            if null_partition {
                let mask = &cat_sums
                .column(&partition_column_name.as_str())
                .unwrap()
                .is_in(&input_domain.categories_keys[cat_column_index])
                .unwrap();
                let unknown_sum: f64 = cat_sums
                                                .filter(&!mask)
                                                .unwrap()
                                                .column("sum")
                                                .unwrap()
                                                .sum()
                                                .unwrap();

                let df_unkown = df!(&partition_column_name.as_str() => &["unknown"],
                                                "sum" => &[unknown_sum]).unwrap();

                let sums = sums.vstack(&df_unkown).unwrap();

            } 

            let vec_sums: Vec<f64> = sums.column("sum").unwrap().f64().unwrap().into_no_null_iter().collect();

            Ok(vec_sums)
        }),
        SymmetricDistance::default(),
        ProductMetric::new(SymmetricDistance::default()), // TO DO !!!
        StabilityMap::new(move |d_in: &IntDistance| (*d_in, *d_in.min(&d_output_partitions))), // TO DO !!!
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
} */
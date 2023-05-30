//use opendp_derive::bootstrap;

//use az::UnwrappedAs;
use polars::prelude::*;
//use std::collections::{BTreeSet};

use crate::{
    core::{Function, StabilityMap, Transformation},
    error::Fallible,
    metrics::{SymmetricDistance, AbsoluteDistance, IntDistance},
    traits::{Hashable, ExactIntCast, Float},
    domains::{AtomDomain, SeriesDomain, LazyFrameDomain},
    //domains::polars::lazyframe::IntoColumnIndex;
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
pub fn make_sized_partitioned_sum<TC: Hashable, T: Float>(
    input_domain: LazyFrameDomain,
    partition_column: &str,
    sum_column: &str,
    bounds: (T,T),
    null_partition: bool,
) -> Fallible<
    Transformation<
        LazyFrameDomain,
        LazyFrameDomain,
        SymmetricDistance, 
        SymmetricDistance, // @MIKE: Difficulty here to compute the bouded sum sensititivity
    >,
> {

    // Check if partition column is in domain and get partition_column ref
    let partition_id = (input_domain.series_domains.iter()).position(|s| s.field.name == partition_column).ok_or_else(|| err!(FailedFunction, "Desired partition column not in domain."))?;
    
    // Check if sum column is in domain and get sum_col_id
    let sum_id = (input_domain.series_domains.iter()).position(|s| s.field.name == sum_column).ok_or_else(|| err!(FailedFunction, "Desired column to be summed not in domain."))?;
    
    // Get margins keys
    let margins_key = input_domain.margins.keys().into_iter().find(|k| k.contains(&partition_id)).unwrap();

    // Get partition margin
    let counts = input_domain.margins.get(margins_key).unwrap();
    
    // Check if margins include counts
    if counts.counts_index.is_none() {
        return fallible!(FailedFunction, "Dataframe domain does not includes counts for the selected partition colunm.")
    }

    // Get categories index
    let categories = counts.data.select(&[col(partition_column)]).collect()?.column(partition_column)?;

    // Defined output sizes 
    let partion_size = categories.len();
    let output_partitions = partion_size + if null_partition { 1 } else { 0 };
    let d_output_partitions = IntDistance::exact_int_cast(output_partitions)?; //What is this doing?

    // Create output col name
    let sum_column_name = "Bounded sums of ".to_string() + sum_column;

    // Type of categorical column
    let cat_type = input_domain.column(partition_column).unwrap().field.dtype;

    // Create output domain
    let output_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new(partition_column, AtomDomain::<DataType>::default()), // @MIKE: possible or runtime?
        SeriesDomain::new(&sum_column_name, AtomDomain::<f64>::default()),
    ])?;
    let output_domain = output_domain.with_counts(counts.data)?;

    // Compute max partition size
    let counts_col_name = counts.get_count_column_name()?;
    // MIKE: Compute max and cast it to usize for input into openDP's type (carefully check that step!!!)
    let size: usize = counts.data.collect()?.column(&counts_col_name)?.max().unwrap().try_into().unwrap();

    // Compute ideal sensitivity
    let (lower, upper) = bounds;
    let ideal_sensitivity = upper.inf_sub(&lower)?;
    
    // Compute sensitivity correction for bit approximation
    let size_exact = T::exact_int_cast(size)?;
    let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
    let _2 = T::exact_int_cast(2)?;

    //Formula is: n^2 / 2^(k - 1) max(|L|, U)
    let error = size.inf_mul(&size)?
                    .inf_div(&_2.inf_pow(&mantissa_bits)?)?
                    .inf_mul(&lower.alerting_abs()?.total_max(upper)?)?;

    let relaxation = error.inf_add(&error)?;

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |data: &LazyFrame| {
            
            // Check if col names exists
            if !data.collect()?.get_column_names().contains(&partition_column) {
                return fallible!(FailedFunction, "Dataframe does not contains the desired partition colunm.")
            }
            if !data.collect()?.get_column_names().contains(&sum_column) {
                return fallible!(FailedFunction, "Dataframe does not contains the column to be summed.")
            }

            // Create partitioned bounded sums
            let mut cat_sums = data.clone()
                        .groupby([partition_column])
                        .agg([col(sum_column).sum()])
                        .collect()
                        .unwrap();

            cat_sums.columns([partition_column, &counts_col_name]);

            // Known / unknwon categories
            let mask_known_cat = &cat_sums
                        .column(partition_column)
                        .unwrap()
                        .is_in(counts.data.collect()?.column(partition_column)?)
                        .unwrap();

            // Remove unknown classes
            let sums = cat_sums.filter(
                &mask_known_cat)
                .unwrap();

            // Compute and concatenate unkown categories aggregation
            if null_partition {
                let unknown_sum: f64 = cat_sums
                                                .filter(&!mask_known_cat)
                                                .unwrap()
                                                .column("sum")
                                                .unwrap()
                                                .sum()
                                                .unwrap();

                let df_unkown = df!(partition_column => &["Unknown"], // check if type conversion is needed
                                                &counts_col_name => &[unknown_sum]).unwrap();

                let sums = sums.vstack(&df_unkown).unwrap();
            } 

            //let vec_sums: Vec<f64> = sums.column("counts_col_name").unwrap().f64().unwrap().into_no_null_iter().collect();

            Ok(sums.lazy())
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(), //@MIKE: Not ok as we have vector of sums whose sensitivity / distance is not Float
        StabilityMap::new(move |d_in: &IntDistance|
            T::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)) // @MIKE: how to handle sensititivity with a dataframe and a vector.
    )
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
use opendp_derive::bootstrap;

use polars::prelude::*;
use polars::datatypes::DataType::{Utf8, Float64};

use crate::{
    core::{Function, StabilityMap, Transformation},
    error::Fallible,
    metrics::{InsertDeleteDistance, L1Distance, IntDistance},
    traits::{Float},
    domains::{AtomDomain, SeriesDomain, LazyFrameDomain},
};

 #[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
           input_domain(rust_type= b"null", c_type = "AnyDomain *"),
           bounds(rust_type = "(T, T)"),
           null_partition(default = false)),
    generics(T(example = "$get_first(bounds)"))
)]
/// Make a Transformation that partitions a dataframe by a given column and compute bounded sums.
/// 
/// # Arguments
/// * `input_domain` - LazyFrameDomain with relevant categories and counts metadata.
/// * `partition_column` - Name of column to split dataframe by.
/// * `sum_column` -  Name of column to sum.
/// * `bound` -  Bounds for the DP sum.
/// * `null_partition` - Whether to include a trailing null partition for rows that were not in the `partition_keys`
/// 
/// # Generics
/// * `T` - Type of bounds for clamping.
pub fn make_sized_partitioned_sum<T: Float>(
    input_domain: LazyFrameDomain,
    partition_column: &str,
    sum_column: &str,
    bounds: (T,T),
    null_partition: bool,
) -> Fallible<
    Transformation<
        LazyFrameDomain,
        LazyFrameDomain,
        InsertDeleteDistance, 
        L1Distance<T>,
    >,
> {

    // Copy parition col name and check if partition column is in domain and get partition_column ref
    let partition_name = partition_column.to_string().clone();
    
    // Copy name of column to be summed and and check if sum column is in domain and get sum_col_id
    let sum_name = sum_column.to_string().clone();
    let _sum_id = (input_domain.series_domains.iter()).position(|s| s.field.name == sum_column).ok_or_else(|| err!(FailedFunction, "Desired column to be summed not in domain."))?;
    
    // Get margins keys
    // TO DO: Check if margins exists!!!!!
    let margins_key = input_domain.margins.keys().into_iter().find(|k| k.contains(&partition_name)).unwrap();
    // Get partition margin
    let counts = input_domain.margins.get(margins_key).unwrap().clone();
    
    // Check if margins include counts
    if counts.counts_index.is_none() {
        return fallible!(FailedFunction, "Dataframe domain does not includes counts for the selected partition colunm.")
    }

    // Create output col name
    let sum_column_name = "Bounded sums of ".to_string() + sum_column;

    // Create output domain
    let output_domain = LazyFrameDomain::new(vec![
        input_domain.column(partition_column.clone()).unwrap().clone(),
        SeriesDomain::new(&sum_column_name, AtomDomain::<f64>::default()),
    ])?;
    let output_domain = output_domain.with_counts(counts.data.clone())?;

    // Compute max partition size
    let counts_col_name = counts.get_count_column_name()?;
    let size = counts.data.clone().collect()?.column(&counts_col_name)?.max::<T>().unwrap();

    // Compute ideal sensitivity
    let (lower, upper) = bounds;
    let ideal_sensitivity = upper.inf_sub(&lower)?;
    
    // Compute sensitivity correction for bit approximation
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
            if !data.clone().collect()?.get_column_names().contains(&partition_name.as_str()) {
                return fallible!(FailedFunction, "Dataframe does not contains the desired partition colunm.")
            }
            if !data.clone().collect()?.get_column_names().contains(&sum_name.as_str()) {
                return fallible!(FailedFunction, "Dataframe does not contains the column to be summed.")
            }

            // Create partitioned bounded sums
            let cat_sums = data.clone()
                        .groupby_stable([col(partition_name.as_str()).alias(partition_name.clone().as_str())])
                        .agg([col(sum_name.as_str())
                                        .alias(sum_column_name.clone().as_str())
                                        .clip(AnyValue::from(lower.to_f64().unwrap()),AnyValue::from(upper.to_f64().unwrap()))
                                        .sum()])
                        .collect()
                        .unwrap();
            
            // Known / unknwon categories
            let mask_known_cat = &cat_sums
                        .column(partition_name.as_str())
                        .unwrap()
                        .is_in(counts.data.clone().collect()?.column(partition_name.as_str())?)
                        .unwrap();

            // Remove unknown classes
            let mut sums = cat_sums.filter(
                &mask_known_cat)
                .unwrap();

            // Compute and concatenate unkown categories aggregation
            if null_partition {    
                let mut unkowns = cat_sums.filter(
                    &!mask_known_cat)
                    .unwrap();

                let number_of_unkonwn_cats = unkowns.height();
                unkowns.with_column(Series::new(partition_name.as_str(), vec!["Unknown"; number_of_unkonwn_cats]))?;

                let unknown_sums = unkowns.lazy()
                                        .groupby([col(partition_name.as_str())])
                                        .agg([col(sum_column_name.as_str())
                                        .sum()])
                                        .collect()
                                        .unwrap();


                // Convert partition vector class to string to ass "unknown"
                sums.with_column(sums.column(partition_name.as_str())?.cast(&Utf8)?)?;

                // Append unknown class
                let mut sums_with_unknown = sums.vstack(&unknown_sums).unwrap();
                sums_with_unknown.with_column(sums_with_unknown.column(sum_column_name.as_str())?.cast(&Float64)?)?;

                Ok(sums_with_unknown.lazy())
            } else {
                sums.with_column(sums.column(sum_column_name.as_str())?.cast(&Float64)?)?;
                Ok(sums.lazy())
            }

        }),
        InsertDeleteDistance::default(),
        L1Distance::<T>::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance|
            T::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)) 
    )
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dataFrame_partition_known_categories() -> Fallible<()> {

        let data_frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("Age", AtomDomain::<i32>::default()),
            SeriesDomain::new("Country", AtomDomain::<String>::default()),
        ])?
        .with_counts(df!["Country" => ["CH", "US"], "count" => [3, 2]]?.lazy())?;

        let data = df!("Age" => [1, 4, 2, 8, 10], "Country" => ["CH", "CH", "US", "CH", "US"])?.lazy();


        let bounded_partitioned_sum = make_sized_partitioned_sum(
                                                                            data_frame_domain,
                                                                            "Country",
                                                                            "Age",
                                                                            (0.,99.),
                                                                            false).unwrap();

        let result = bounded_partitioned_sum.invoke(&data).unwrap(); 

        let df_check = df!("Country" => ["CH", "US"],
                                    "Bounded sums of Age" => [13., 12.],)?;

        assert!(result.clone().collect()?.frame_equal(&df_check));

        Ok(())
    }

    #[test] 
    fn test_dataFrame_partition_unknown_categories() -> Fallible<()> {

        let data_frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("Age", AtomDomain::<i32>::default()),
            SeriesDomain::new("Country", AtomDomain::<String>::default()),
        ])?
        .with_counts(df!["Country" => ["CH", "US"], "count" => [3, 2]]?.lazy())?;

        let data = df!("Age" => [1., 4., 2., 8., 10., 9., 1.],
                                    "Country" => ["CH", "CH", "US", "CH", "US", "UK", "IT"])?.lazy();

        let bounded_partitioned_sum_withUnkown = make_sized_partitioned_sum(
            data_frame_domain,
            "Country",
            "Age",
            (0.,99.),
            true).unwrap();

        let result = bounded_partitioned_sum_withUnkown.invoke(&data).unwrap(); 
        
        let df_check = df!("Country" => ["CH", "US", "Unknown"],
                                    "Bounded sums of Age" => [13., 12., 10.],)?;

        assert!(result.clone().collect()?.frame_equal(&df_check));

        Ok(())
    }
}
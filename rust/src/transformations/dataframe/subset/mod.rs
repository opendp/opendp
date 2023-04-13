use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::LazyFrameDomain,
    error::Fallible,
    transformations::DatasetMetric,
};
use polars::prelude::*;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *"),
        indicator_column(c_type = "const char *")
    )
)]
/// Make a Transformation that subsets a dataframe by a boolean column.
///
/// # Arguments
/// * `input_domain` - domain of the input dataframe
/// * `input_metric` - metric on the input domain
/// * `indicator_column` - name of the boolean column that indicates inclusion in the subset
///
/// # Generics
/// * `M` - Metric type
pub fn make_subset_by<M: DatasetMetric>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    indicator_column: &str,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, M, M>>
{
    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new_fallible(move |data: &LazyFrame| {
            unimplemented!();
            // // the partition to move each row into
            // let indicator = (data.get(&indicator_column))
            //     .ok_or_else(|| err!(FailedFunction, "{:?} does not exist in the input dataframe"))?
            //     .as_form::<Vec<bool>>()?;

            // // where to collect partitioned data
            // let mut subsetted = DataFrame::new();

            // // iteratively partition each column
            // keep_columns.iter().try_for_each(|column_name| {
            //     // retrieve a Column from the dataframe
            //     let column = data.get(&column_name).ok_or_else(|| {
            //         err!(FailedFunction, "{:?} does not exist in the input dataframe")
            //     })?;

            //     subsetted.insert(column_name.clone(), column.subset(&indicator));

            //     Fallible::Ok(())
            // })?;

            // Ok(subsetted)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}

#[cfg(test)]
mod test {
    use crate::{domains::SeriesDomain, metrics::SymmetricDistance};

    use super::*;

    #[test]
    fn test_subset_by() -> Fallible<()> {
        let trans = make_subset_by(
            LazyFrameDomain::new(vec![
                SeriesDomain::new::<bool>("filter"),
                SeriesDomain::new::<String>("values"),
            ])?,
            SymmetricDistance::default(),
            "filter",
        )?;

        let frame = df![
            "filter" => [true, false, false, true],
            "values" => ["1", "2", "3", "4"]
        ]?;
        let res = trans.invoke(&frame.lazy())?.collect()?;

        let expected = df![
            "values" => ["1", "4"]
        ]?;
        assert_eq!(res, expected);
        Ok(())
    }
}

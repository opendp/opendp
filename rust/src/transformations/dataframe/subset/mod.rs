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
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        DataFrameDomain::new_all(),
        Function::new_fallible(move |data: &DataFrame<TK>| {
            // the partition to move each row into
            let indicator = (data.get(&indicator_column))
                .ok_or_else(|| err!(FailedFunction, "{:?} does not exist in the input dataframe"))?
                .as_form::<Vec<bool>>()?;

            // where to collect partitioned data
            let mut subsetted = DataFrame::new();

            // iteratively partition each column
            keep_columns.iter().try_for_each(|column_name| {
                // retrieve a Column from the dataframe
                let column = data.get(&column_name).ok_or_else(|| {
                    err!(FailedFunction, "{:?} does not exist in the input dataframe")
                })?;

                subsetted.insert(column_name.clone(), column.subset(&indicator));

                Fallible::Ok(())
            })?;

            Ok(subsetted)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    ))
}

#[cfg(test)]
mod test {
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_subset_by() -> Fallible<()> {
        let trans = make_subset_by::<String>("filter".to_string(), vec!["values".to_string()])?;

        let mut df = DataFrame::new();
        df.insert("filter".to_string(), vec![true, false, false, true].into());
        df.insert("values".to_string(), vec!["1", "2", "3", "4"].into());
        let res = trans.invoke(&df)?;

        let subset = res
            .get("values")
            .unwrap_test()
            .as_form::<Vec<&str>>()?
            .clone();

        assert_eq!(subset, vec!["1", "4"]);
        Ok(())
    }
}

use opendp_derive::bootstrap;

use crate::{core::{Transformation, Function, StabilityMap}, error::Fallible, domains::{VectorDomain, AllDomain, Array2Domain}, metrics::SymmetricDistance, traits::{Hashable, Primitive}};
use ndarray::Array2;

use super::{DataFrameDomain, DataFrame};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib"))]
/// Make a Transformation that retrieves the column `key` from a dataframe as `Vec<TOA>`.
/// 
/// # Arguments
/// * `key` - categorical/hashable data type of the key/column name
/// 
/// # Generics
/// * `K` - data type of key
/// * `TOA` - Atomic Output Type to downcast vector to
pub fn make_select_column<K, TOA>(
    key: K,
) -> Fallible<
    Transformation<
        DataFrameDomain<K>,
        VectorDomain<AllDomain<TOA>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    K: Hashable,
    TOA: Primitive,
{
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &DataFrame<K>| -> Fallible<Vec<TOA>> {
            // retrieve column from dataframe and handle error
            arg.get(&key)
                .ok_or_else(|| err!(FailedFunction, "column does not exist: {:?}", key))?
                // cast down to &Vec<T>
                .as_form::<Vec<TOA>>()
                .map(|c| c.clone())
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    ))
}


fn dataframe_to_array<K: Hashable, TOA: Primitive>(
    col_names: &Vec<K>,
    dataframe: &DataFrame<K>,
) -> Fallible<Array2<TOA>> {
    let vecs = col_names
        .iter()
        .map(|k| {
            dataframe
                .get(k)
                .ok_or_else(|| err!(FailedFunction, "column does not exist: {:?}", k))?
                .as_form::<Vec<TOA>>()
        })
        .collect::<Fallible<Vec<&Vec<TOA>>>>()?;

    let nrows = vecs[0].len();
    let flat = vecs.into_iter().flatten().cloned().collect::<Vec<TOA>>();

    Array2::from_shape_vec((nrows, col_names.len()), flat)
        .map_err(|e| err!(FailedFunction, "{:?}", e))
}


pub fn make_select_array<K: Hashable, TOA: Primitive>(col_names: Vec<K>) -> Fallible<Transformation<DataFrameDomain<K>, Array2Domain<AllDomain<TOA>>, SymmetricDistance, SymmetricDistance>> {
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        Array2Domain::new_all(),
        Function::new_fallible(move |arg: &DataFrame<K>| {
            dataframe_to_array(&col_names, arg)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1)
    ))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::ExplainUnwrap, data::Column};

    #[test]
    fn test_make_select_column() {
        let transformation = make_select_column::<String, String>("1".to_owned()).unwrap_test();
        let arg: DataFrame<String> = vec![
            (
                "0".to_owned(),
                Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]),
            ),
            (
                "1".to_owned(),
                Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]),
            ),
        ]
        .into_iter()
        .collect();
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
    }


    #[test]
    fn test_make_select_array() -> Fallible<()> {

        let trans = make_select_array(vec![1, 2, 3])?;

        let mut df = DataFrame::new();
        df.insert(1, Column::new(vec![1., 2., 3.]));
        df.insert(2, Column::new(vec![4., 5., 6.]));
        df.insert(3, Column::new(vec![7., 8., 9.]));

        let arr: Array2<f64> = trans.invoke(&df)?;

        println!("{:?}", arr);
        Ok(())
    }
}

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::SymmetricDistance,
    traits::{Hashable, Primitive},
};

use super::{DataFrame, DataFrameDomain};

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
        VectorDomain<AtomDomain<TOA>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::Column, error::ExplainUnwrap};

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
}

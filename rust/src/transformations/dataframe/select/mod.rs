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
#[deprecated(note = "Use Polars instead", since = "0.12.0")]
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
    Transformation::new(
        DataFrameDomain::new(),
        VectorDomain::new(AtomDomain::default()),
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
    )
}

#[cfg(test)]
mod test;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation, MetricSpace},
    domains::{AtomDomain, LazyFrameDomain, VectorDomain},
    error::Fallible,
    traits::Primitive,
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
        key(c_type = "const char *")
    )
)]
/// Make a Transformation that retrieves the column `key` from a dataframe as `Vec<TOA>`.
///
/// # Arguments
/// * `key` - categorical/hashable data type of the key/column name
///
/// # Generics
/// * `TOA` - Atomic Output Type to downcast vector to
/// * `M` - metric type
pub fn make_select_column<TOA, M>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    key: &str,
) -> Fallible<
    Transformation<
        LazyFrameDomain,
        VectorDomain<AtomDomain<TOA>>,
        M,
        M,
    >,
>
where
    TOA: Primitive,
    M: DatasetMetric,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
    let output_domain = input_domain.column(key)
        .ok_or_else(|| err!(FailedCast, "Column not found"))?;
    Transformation::new(
        input_domain,
        VectorDomain::new(AtomDomain::default(), None),
        Function::new_fallible(move |arg: &LazyFrame| -> Fallible<Vec<TOA>> {
            unimplemented!();
            // retrieve column from dataframe and handle error
            // arg.collect()?.column(&key)?
            //     // cast down to &Vec<T>
            //     .as_form::<Vec<TOA>>()
            //     .map(|c| c.clone())
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domains::SeriesDomain, error::ExplainUnwrap, metrics::SymmetricDistance};

    #[test]
    fn test_make_select_column() -> Fallible<()> {
        let transformation = make_select_column::<String, SymmetricDistance>(
            LazyFrameDomain::new(vec![
                SeriesDomain::new::<String>("0"),
                SeriesDomain::new::<String>("1"),
            ])?,
            SymmetricDistance::default(),
            "1",
        )
        .unwrap_test();
        let arg = df![
            "0" => ["ant", "bat", "cat"],
            "1" => ["foo", "bar", "baz"]
        ]?.lazy();
        let ret = transformation.invoke(&arg)?;
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
        Ok(())
    }
}

use opendp_derive::bootstrap;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    data::Column,
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::EventLevelMetric,
    traits::{Hashable, Primitive, RoundCast},
    transformations::{make_cast_default, make_is_equal},
};

use super::{DataFrame, DataFrameDomain};

#[cfg(feature = "ffi")]
mod ffi;

#[deprecated(note = "Use Polars instead", since = "0.12.0")]
/// Internal function to map a transformation onto a column of a dataframe.
fn make_apply_transformation_dataframe<K: Hashable, VI: Primitive, VO: Primitive, M>(
    input_domain: DataFrameDomain<K>,
    input_metric: M,
    column_name: K,
    transformation: Transformation<
        VectorDomain<AtomDomain<VI>>,
        M,
        VectorDomain<AtomDomain<VO>>,
        M,
    >,
) -> Fallible<Transformation<DataFrameDomain<K>, M, DataFrameDomain<K>, M>>
where
    M: EventLevelMetric,
    (DataFrameDomain<K>, M): MetricSpace,
    (VectorDomain<AtomDomain<VI>>, M): MetricSpace,
    (VectorDomain<AtomDomain<VO>>, M): MetricSpace,
{
    let function = transformation.function.clone();

    Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        input_domain,
        input_metric,
        Function::new_fallible(move |arg: &DataFrame<K>| {
            let mut data = arg.clone();
            let column = data.remove(&column_name).ok_or_else(|| {
                err!(
                    FailedFunction,
                    "{:?} does not exist in the input dataframe",
                    column_name
                )
            })?;

            data.insert(
                column_name.clone(),
                Column::new(function.eval(column.as_form::<Vec<VI>>()?)?),
            );
            Ok(data)
        }),
        StabilityMap::new_from_constant(1),
    )
}

#[deprecated(note = "Use Polars instead", since = "0.12.0")]
#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *")
    ),
    generics(TK(suppress), M(suppress)),
    derived_types(
        TK = "$get_atom(get_type(input_domain))",
        M = "$get_type(input_metric)"
    )
)]
/// Make a Transformation that casts the elements in a column in a dataframe from type `TIA` to type `TOA`.
/// If cast fails, fill with default.
///
///
/// | `TIA`  | `TIA::default()` |
/// | ------ | ---------------- |
/// | float  | `0.`             |
/// | int    | `0`              |
/// | string | `""`             |
/// | bool   | `false`          |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `column_name` - column name to be transformed
///
/// # Generics
/// * `TK` - Type of the column name
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_df_cast_default<TK, TIA, TOA, M>(
    input_domain: DataFrameDomain<TK>,
    input_metric: M,
    column_name: TK,
) -> Fallible<Transformation<DataFrameDomain<TK>, M, DataFrameDomain<TK>, M>>
where
    TK: Hashable,
    TIA: Primitive,
    TOA: Primitive + RoundCast<TIA>,
    M: EventLevelMetric,
    (DataFrameDomain<TK>, M): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
    #[allow(deprecated)]
    make_apply_transformation_dataframe(
        input_domain,
        input_metric.clone(),
        column_name,
        make_cast_default::<M, TIA, TOA>(Default::default(), input_metric)?,
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *")
    ),
    generics(TK(suppress), M(suppress)),
    derived_types(
        TK = "$get_atom(get_type(input_domain))",
        M = "$get_type(input_metric)"
    )
)]
#[deprecated(note = "Use Polars instead", since = "0.12.0")]
/// Make a Transformation that checks if each element in a column in a dataframe is equivalent to `value`.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `column_name` - Column name to be transformed
/// * `value` - Value to check for equality
///
/// # Generics
/// * `TK` - Type of the column name
/// * `TIA` - Atomic Input Type to cast from
pub fn make_df_is_equal<TK, TIA, M>(
    input_domain: DataFrameDomain<TK>,
    input_metric: M,
    column_name: TK,
    value: TIA,
) -> Fallible<Transformation<DataFrameDomain<TK>, M, DataFrameDomain<TK>, M>>
where
    TK: Hashable,
    TIA: Primitive,
    M: EventLevelMetric,
    (DataFrameDomain<TK>, M): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
{
    let column_input_domain = VectorDomain::new(AtomDomain::default());
    #[allow(deprecated)]
    make_apply_transformation_dataframe(
        input_domain,
        input_metric.clone(),
        column_name,
        make_is_equal(column_input_domain, input_metric, value)?,
    )
}

#[cfg(test)]
mod test;

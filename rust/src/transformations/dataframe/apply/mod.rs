use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain, LazyFrameDomain},
    error::Fallible,
    traits::{Primitive, RoundCast},
    transformations::{make_cast_default, make_is_equal, DatasetMetric},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Internal function to map a transformation onto a column of a dataframe.
fn make_apply_transformation_dataframe<VI: Primitive, VO: Primitive, M>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    column_name: &str,
    transformation: Transformation<
        VectorDomain<AtomDomain<VI>>,
        VectorDomain<AtomDomain<VO>>,
        M,
        M,
    >,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, M, M>>
where
    M: DatasetMetric,
    (LazyFrameDomain, M): MetricSpace,
    (VectorDomain<AtomDomain<VI>>, M): MetricSpace,
    (VectorDomain<AtomDomain<VO>>, M): MetricSpace,
{
    let function = transformation.function.clone();

    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new_fallible(move |arg: &LazyFrame| {
            let mut data = arg.clone();
            let column = data.select(&[col(column_name)]);

            data.with_column(
                // TODO: Lazy API doesn't like arbitrary function execution
                unimplemented!()
                // Series::new(column_name.clone(), function.eval(column.as_form::<Vec<VI>>()?)?),
            );
            Ok(data)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *"),
        column_name(c_type = "const char *"),
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
/// * `column_name` - column name to be transformed
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_df_cast_default<TIA, TOA, M>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    column_name: &str,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, M, M>>
where
    TIA: Primitive,
    TOA: Primitive + RoundCast<TIA>,
    M: DatasetMetric,
    (LazyFrameDomain, M): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
    make_apply_transformation_dataframe(
        input_domain,
        input_metric.clone(),
        column_name,
        make_cast_default::<TIA, TOA, _>(Default::default(), input_metric)?,
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *"),
        column_name(c_type = "const char *"),
    ),
    generics(TK(suppress), M(suppress)),
    derived_types(
        TK = "$get_atom(get_type(input_domain))",
        M = "$get_type(input_metric)"
    )
)]
/// Make a Transformation that checks if each element in a column in a dataframe is equivalent to `value`.
///
/// # Arguments
/// * `column_name` - Column name to be transformed
/// * `value` - Value to check for equality
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
pub fn make_df_is_equal<TIA, M>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    column_name: &str,
    value: TIA,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, M, M>>
where
    TIA: Primitive,
    M: DatasetMetric,
    (LazyFrameDomain, M): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
{
    let column_input_domain = VectorDomain::new(AtomDomain::default(), None);
    make_apply_transformation_dataframe(
        input_domain,
        input_metric.clone(),
        column_name,
        make_is_equal(column_input_domain, input_metric, value)?,
    )
}

#[cfg(test)]
mod test {
    use crate::{metrics::SymmetricDistance, domains::SeriesDomain};

    use super::*;

    #[test]
    fn test_df_cast_default() -> Fallible<()> {
        let trans = make_df_cast_default::<i32, bool, _>(
            LazyFrameDomain::new(vec![
                SeriesDomain::new::<i32>("filter"), 
                SeriesDomain::new::<String>("values"),
            ])?,
            SymmetricDistance::default(),
            "filter",
        )?;

        let df = DataFrame::new(vec![
            Series::new("filter", vec![0, 1, 3, 0]),
            Series::new("values", vec!["1", "2", "3", "4"]),
        ])?;
        let res = trans.invoke(&df.lazy())?;

        let filter: Vec<Option<bool>> = res.collect()?
            .column("filter")?
            .bool()?
            .into();

        assert_eq!(filter, vec![Some(false), Some(true), Some(true), Some(false)]);

        Ok(())
    }

    #[test]
    fn test_df_is_equal() -> Fallible<()> {
        let trans = make_df_is_equal(
            LazyFrameDomain::new(vec![SeriesDomain::new::<String>("0")])?,
            SymmetricDistance::default(),
            "0",
            "true".to_string(),
        )?;

        let frame = df!(
            "0" => vec!["false", "true", "true", "false"],
            "1" => vec![12., 23., 94., 128.]
        )?;
        let res = trans.invoke(&frame.lazy())?;

        let filter: Vec<Option<bool>> = res.collect()?.column("0")?.bool()?.into();

        assert_eq!(filter, vec![Some(false), Some(true), Some(true), Some(false)]);

        Ok(())
    }
}

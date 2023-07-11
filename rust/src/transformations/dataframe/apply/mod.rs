use opendp_derive::bootstrap;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    data::Column,
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    traits::{Hashable, Primitive, RoundCast},
    transformations::{make_cast_default, make_is_equal, DatasetMetric},
};

use super::{DataFrame, OldFrameDomain};

#[cfg(feature = "ffi")]
mod ffi;

/// Internal function to map a transformation onto a column of a dataframe.
fn make_apply_transformation_dataframe<K: Hashable, VI: Primitive, VO: Primitive, M>(
    input_domain: OldFrameDomain<K>,
    input_metric: M,
    column_name: K,
    transformation: Transformation<
        VectorDomain<AtomDomain<VI>>,
        VectorDomain<AtomDomain<VO>>,
        M,
        M,
    >,
) -> Fallible<Transformation<OldFrameDomain<K>, OldFrameDomain<K>, M, M>>
where
    M: DatasetMetric,
    (OldFrameDomain<K>, M): MetricSpace,
    (VectorDomain<AtomDomain<VI>>, M): MetricSpace,
    (VectorDomain<AtomDomain<VO>>, M): MetricSpace,
{
    let function = transformation.function.clone();

    Transformation::new(
        input_domain.clone(),
        input_domain,
        Function::new_fallible(move |arg: &DataFrame<K>| {
            let mut data = arg.clone();
            let column = data.remove(&column_name).ok_or_else(|| {
                err!(FailedFunction, "{:?} does not exist in the input dataframe")
            })?;

            data.insert(
                column_name.clone(),
                Column::new(function.eval(column.as_form::<Vec<VI>>()?)?),
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
/// * `column_name` - column name to be transformed
///
/// # Generics
/// * `TK` - Type of the column name
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_df_cast_default<TK, TIA, TOA, M>(
    input_domain: OldFrameDomain<TK>,
    input_metric: M,
    column_name: TK,
) -> Fallible<Transformation<OldFrameDomain<TK>, OldFrameDomain<TK>, M, M>>
where
    TK: Hashable,
    TIA: Primitive,
    TOA: Primitive + RoundCast<TIA>,
    M: DatasetMetric,
    (OldFrameDomain<TK>, M): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
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
/// Make a Transformation that checks if each element in a column in a dataframe is equivalent to `value`.
///
/// # Arguments
/// * `column_name` - Column name to be transformed
/// * `value` - Value to check for equality
///
/// # Generics
/// * `TK` - Type of the column name
/// * `TIA` - Atomic Input Type to cast from
pub fn make_df_is_equal<TK, TIA, M>(
    input_domain: OldFrameDomain<TK>,
    input_metric: M,
    column_name: TK,
    value: TIA,
) -> Fallible<Transformation<OldFrameDomain<TK>, OldFrameDomain<TK>, M, M>>
where
    TK: Hashable,
    TIA: Primitive,
    M: DatasetMetric,
    (OldFrameDomain<TK>, M): MetricSpace,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
{
    let column_input_domain = VectorDomain::new(AtomDomain::default());
    make_apply_transformation_dataframe(
        input_domain,
        input_metric.clone(),
        column_name,
        make_is_equal(column_input_domain, input_metric, value)?,
    )
}

#[cfg(test)]
mod test {
    use crate::{error::ExplainUnwrap, metrics::SymmetricDistance};

    use super::*;

    #[test]
    fn test_df_cast_default() -> Fallible<()> {
        let trans = make_df_cast_default::<String, i32, bool, _>(
            Default::default(),
            SymmetricDistance::default(),
            "filter".to_string(),
        )?;

        let mut df = DataFrame::new();
        df.insert("filter".to_string(), vec![0, 1, 3, 0].into());
        df.insert("values".to_string(), vec!["1", "2", "3", "4"].into());
        let res = trans.invoke(&df)?;

        let filter = res
            .get("filter")
            .unwrap_test()
            .as_form::<Vec<bool>>()?
            .clone();

        assert_eq!(filter, vec![false, true, true, false]);

        Ok(())
    }

    #[test]
    fn test_df_is_equal() -> Fallible<()> {
        let trans = make_df_is_equal(
            Default::default(),
            SymmetricDistance::default(),
            0,
            "true".to_string(),
        )?;

        let mut df = DataFrame::new();
        df.insert(
            0,
            vec![
                "false".to_string(),
                "true".to_string(),
                "true".to_string(),
                "false".to_string(),
            ]
            .into(),
        );
        df.insert(1, vec![12., 23., 94., 128.].into());
        let res = trans.invoke(&df)?;

        let filter = res.get(&0).unwrap_test().as_form::<Vec<bool>>()?.clone();

        assert_eq!(filter, vec![false, true, true, false]);

        Ok(())
    }
}

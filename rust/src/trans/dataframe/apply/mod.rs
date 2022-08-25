use crate::{
    core::{Function, StabilityMap, Transformation},
    data::Column,
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    metrics::SymmetricDistance,
    traits::{Hashable, Primitive, RoundCast},
    trans::{make_cast_default, make_is_equal},
};

use super::{DataFrame, DataFrameDomain};

#[cfg(feature = "ffi")]
mod ffi;

fn make_apply_transformation_dataframe<K: Hashable, VI: Primitive, VO: Primitive>(
    column_name: K,
    transformation: Transformation<
        VectorDomain<AllDomain<VI>>,
        VectorDomain<AllDomain<VO>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
) -> Fallible<
    Transformation<DataFrameDomain<K>, DataFrameDomain<K>, SymmetricDistance, SymmetricDistance>,
> {
    let function = transformation.function.clone();

    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        DataFrameDomain::new_all(),
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
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    ))
}

pub fn make_df_cast_default<K, TIA, TOA>(
    column_name: K,
) -> Fallible<
    Transformation<DataFrameDomain<K>, DataFrameDomain<K>, SymmetricDistance, SymmetricDistance>,
>
where
    K: Hashable,
    TIA: Primitive,
    TOA: Primitive + RoundCast<TIA>,
{
    make_apply_transformation_dataframe(column_name, make_cast_default::<TIA, TOA>()?)
}

pub fn make_df_is_equal<K, TIA>(
    column_name: K,
    value: TIA,
) -> Fallible<
    Transformation<DataFrameDomain<K>, DataFrameDomain<K>, SymmetricDistance, SymmetricDistance>,
>
where
    K: Hashable,
    TIA: Primitive,
{
    make_apply_transformation_dataframe(column_name, make_is_equal(value)?)
}

#[cfg(test)]
mod test {
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_df_cast_default() -> Fallible<()> {
        let trans = make_df_cast_default::<String, i32, bool>("filter".to_string())?;

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
        let trans = make_df_is_equal(0, "true".to_string())?;

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

        let filter = res
            .get(&0)
            .unwrap_test()
            .as_form::<Vec<bool>>()?
            .clone();

        assert_eq!(filter, vec![false, true, true, false]);

        Ok(())
    }
}

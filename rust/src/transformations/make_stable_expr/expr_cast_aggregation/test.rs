use crate::core::Transformation;
use crate::domains::{Context, Margin, WildExprDomain};
use crate::metrics::{L0PInfDistance, L1Distance, SymmetricDistance};
use polars::prelude::DataType;

use super::*;

const ALL_NUMERIC_TYPES: &[DataType] = &[
    // Signed Integers
    DataType::Int8,
    DataType::Int16,
    DataType::Int32,
    DataType::Int64,
    DataType::Int128,
    // Unsigned Integers
    DataType::UInt8,
    DataType::UInt16,
    DataType::UInt32,
    DataType::UInt64,
    DataType::UInt128,
    // Floats
    DataType::Float32,
    DataType::Float64,
];

#[test]
fn test_make_cast_aggregation() -> Fallible<()> {
    let lf = df!(
        "test_col" => &[1, 2, 3],
    )?
    .lazy();

    let input_domain = WildExprDomain {
        columns: vec![],
        context: Context::Aggregation {
            margin: Margin::select(),
        },
    };

    let transformation: Transformation<_, _, _, L1Distance<f64>> = len()
        .cast(DataType::Int64)
        .make_stable(input_domain, L0PInfDistance::<1, _>(SymmetricDistance))?;

    let expr = transformation.invoke(&lf.logical_plan)?.expr;
    let result = lf.select([expr]).collect()?;

    let expected_df = df!(
        "len" => &[3i64],
    )?;
    assert_eq!(expected_df, result);
    Ok(())
}

#[test]
fn test_make_cast_cannot_downcast() -> Fallible<()> {
    // test cases are all numeric types combo to the supported type.
    let test_cases: Vec<(&DataType, &DataType)> = ALL_NUMERIC_TYPES
        .iter()
        .flat_map(move |b| CAST_TYPES_SUPPORTED.iter().map(move |a| (a, b)))
        .collect();
    // test_cases = (DF Value Type, Target Cast)

    for (col_type, cast_target) in test_cases.iter() {
        let lf = df!(
            "test_col" => &[1, 2, 3],
        )?
        .lazy()
        .with_column(col("test_col").cast((*col_type).clone()));

        let input_domain = WildExprDomain {
            columns: vec![],
            context: Context::Aggregation {
                margin: Margin::select(),
            },
        };

        let transformation: Transformation<_, _, _, L1Distance<f64>> = len()
            .cast((*cast_target).clone())
            .make_stable(input_domain, L0PInfDistance::<1, _>(SymmetricDistance))?;

        let expr = transformation.invoke(&lf.logical_plan)?.expr;
        let cast_target_max = cast_target.max()?;
        let df_max = cast_target.max()?;
        let cast_target_min = cast_target.min()?;
        let df_min = col_type.min()?;

        if cast_target_max.value() >= df_max.value() && cast_target_min.value() <= df_min.value() {
            let result = lf.select([expr]).collect()?;
            assert!(result.height() > 0);
        } else {
            // Need to ensure error occurs here.
            let result = lf.select([expr]).collect();
            assert!(result.is_err());
        }
    }
    Ok(())
}

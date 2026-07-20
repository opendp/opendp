use super::*;
use crate::core::Transformation;
use crate::domains::{Context, Margin, WildExprDomain};
use crate::metrics::{L0PInfDistance, L1Distance, SymmetricDistance};
use polars::prelude::DataType;

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
    // len returns u32, these types downcast it and aren't supported.
    let downcast_types = vec![DataType::Int8, DataType::Int16, DataType::Int32];

    for dtype in downcast_types.iter() {
        let input_domain = WildExprDomain {
            columns: vec![],
            context: Context::Aggregation {
                margin: Margin::select(),
            },
        };

        let result: Fallible<Transformation<_, _, _, L1Distance<f64>>> = len()
            .cast((*dtype).clone())
            .make_stable(input_domain, L0PInfDistance::<1, _>(SymmetricDistance));

        // Assert against a specific error message or variant
        assert!(format!("{:?}", result.unwrap_err()).contains("cannot downcast"));
    }
    Ok(())
}

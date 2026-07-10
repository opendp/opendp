use crate::core::Transformation;
use crate::domains::{Context, Margin, WildExprDomain};
use crate::metrics::{L0PInfDistance, L1Distance, SymmetricDistance};

use super::*;

#[test]
fn test_make_cast_aggregation() -> Fallible<()> {
    // Need to create a cast expression after another dp measurement (like len)
    // Need to check that the output dtype after the expression runs is the cast measurement expected type

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

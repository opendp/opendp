use super::*;
use crate::core::Transformation;
use crate::domains::{AtomDomain, Context, Margin, SeriesDomain, WildExprDomain};
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
    // len returns an unbounded u32 domain, so these target types are too narrow.
    for dtype in [DataType::Int8, DataType::Int16, DataType::Int32] {
        let input_domain = WildExprDomain {
            columns: vec![],
            context: Context::Aggregation {
                margin: Margin::select(),
            },
        };

        let result: Fallible<Transformation<_, _, _, L1Distance<f64>>> = len()
            .cast(dtype)
            .make_stable(input_domain, L0PInfDistance::<1, _>(SymmetricDistance));

        assert!(format!("{:?}", result.unwrap_err()).contains("cannot downcast"));
    }
    Ok(())
}

#[test]
fn test_make_cast_uses_tightened_domain_bounds() -> Fallible<()> {
    let lf = df!("value" => &[u64::MAX / 2, u64::MAX / 2])?.lazy();
    let input_domain = WildExprDomain {
        columns: vec![SeriesDomain::new(
            "value",
            AtomDomain::<u64>::new_closed((0, u64::MAX / 2))?,
        )],
        context: Context::Aggregation {
            margin: Margin::select().with_max_length(2),
        },
    };

    let transformation: Transformation<_, _, _, L1Distance<f64>> = col("value")
        .sum()
        .clip(lit(0u64), lit(i64::MAX as u64))
        .cast(DataType::Int64)
        .make_stable(input_domain, L0PInfDistance::<1, _>(SymmetricDistance))?;

    assert_eq!(
        transformation
            .output_domain
            .column
            .atom_domain::<i64>()?
            .get_closed_bounds()?,
        (0, i64::MAX)
    );

    let expr = transformation.invoke(&lf.logical_plan)?.expr;
    assert_eq!(
        lf.select([expr]).collect()?.column("value")?.i64()?.get(0),
        Some(i64::MAX)
    );
    Ok(())
}

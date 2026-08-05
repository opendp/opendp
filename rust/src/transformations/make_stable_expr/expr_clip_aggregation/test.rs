use super::*;
use crate::core::Transformation;
use crate::domains::{AtomDomain, Context, Margin, WildExprDomain};
use crate::metrics::{L0PInfDistance, L1Distance, SymmetricDistance};
use polars::prelude::*;

#[test]
fn test_make_clip_aggregation() -> Fallible<()> {
    let lf = df!("value" => &[u64::MAX / 2, u64::MAX / 2])?.lazy();
    let input_domain = WildExprDomain {
        columns: vec![crate::domains::SeriesDomain::new(
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
        .make_stable(input_domain, L0PInfDistance::<1, _>(SymmetricDistance))?;

    assert_eq!(
        transformation
            .output_domain
            .column
            .atom_domain::<u64>()?
            .get_closed_bounds()?,
        (0, i64::MAX as u64)
    );

    let expr = transformation.invoke(&lf.logical_plan)?.expr;
    assert_eq!(
        lf.select([expr]).collect()?.column("value")?.u64()?.get(0),
        Some(i64::MAX as u64)
    );
    Ok(())
}

use polars::prelude::{col, lit};

use crate::{
    domains::{ArrayDomain, AtomDomain, Context, SeriesDomain, WildExprDomain},
    metrics::{FrameDistance, SymmetricDistance},
    transformations::make_stable_expr,
};

use super::Fallible;

#[test]
fn test_arr_namespace() -> Fallible<()> {
    let input_domain = WildExprDomain {
        columns: vec![SeriesDomain::new(
            "a",
            ArrayDomain::new(AtomDomain::<i64>::default(), 2),
        )],
        context: Context::RowByRow,
    };

    let res = make_stable_expr(
        input_domain,
        FrameDistance(SymmetricDistance),
        col("a").arr().get(lit(0), true),
    );
    let msg = res.unwrap_err().message.unwrap();
    assert!(msg.starts_with("Expr is not recognized at this time: Get(true)"));
    Ok(())
}

use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_expr::namespace_dt::test::{
    example_dt_lf, parse_example_dt_lf,
};
use crate::transformations::make_stable_lazyframe;

use super::*;

#[test]
fn test_make_expr_components() -> Fallible<()> {
    let (lf_domain, lf) = example_dt_lf()?;
    let lf_plan = parse_example_dt_lf(lf.clone()).with_columns([
        col("time").dt().second(),
        col("datetime").dt().ordinal_day(),
        col("date").dt().quarter(),
    ]);
    let t_components = make_stable_lazyframe(lf_domain, SymmetricDistance, lf_plan)?;

    let actual = t_components.invoke(&lf)?.collect()?;
    let expected = r#"shape: (4, 3)
┌──────┬──────────┬──────┐
│ time ┆ datetime ┆ date │
│ ---  ┆ ---      ┆ ---  │
│ i8   ┆ i16      ┆ i8   │
╞══════╪══════════╪══════╡
│ 12   ┆ 224      ┆ 3    │
│ 23   ┆ 257      ┆ 3    │
│ 30   ┆ 287      ┆ 4    │
│ 22   ┆ null     ┆ null │
└──────┴──────────┴──────┘"#;
    assert_eq!(format!("{:?}", actual), expected);
    Ok(())
}

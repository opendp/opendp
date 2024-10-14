use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_expr::namespace_dt::test::{
    example_dt_lf, parse_example_dt_lf,
};
use crate::transformations::make_stable_lazyframe;

use super::*;

#[test]
fn test_make_expr_strptime() -> Fallible<()> {
    let (lf_domain, lf) = example_dt_lf()?;

    let lf_plan = parse_example_dt_lf(lf.clone());
    let t_casted = make_stable_lazyframe(lf_domain, SymmetricDistance, lf_plan)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = r#"shape: (4, 3)
┌──────────┬─────────────────────┬────────────┐
│ time     ┆ datetime            ┆ date       │
│ ---      ┆ ---                 ┆ ---        │
│ time     ┆ datetime[ns]        ┆ date       │
╞══════════╪═════════════════════╪════════════╡
│ 14:00:12 ┆ 2023-08-12 14:00:34 ┆ 2023-08-12 │
│ 15:00:23 ┆ 2024-09-13 15:00:45 ┆ 2024-09-13 │
│ 16:00:30 ┆ 2025-10-14 16:00:56 ┆ 2025-10-14 │
│ 16:00:22 ┆ null                ┆ null       │
└──────────┴─────────────────────┴────────────┘"#;
    assert_eq!(format!("{:?}", actual), expected);
    Ok(())
}

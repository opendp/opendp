use crate::domains::{AtomDomain, LazyFrameDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

use super::*;

#[test]
fn test_make_expr_strptime() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("time", AtomDomain::<String>::default()),
        SeriesDomain::new("datetime", AtomDomain::<String>::default()),
        SeriesDomain::new("date", AtomDomain::<String>::default()),
    ])?;
    let lf = df!(
        "time" => &[
            "14:00:00".to_string(),
            "15:00:00".to_string(),
            "16:00:00".to_string(),
            "16:00:00".to_string()
        ],
        "datetime" => &[
            "2023-08-12 14:00:00".to_string(),
            "2024-09-13 15:00:00".to_string(),
            "2025-10-14 16:00:00".to_string(),
            "2025-13-14 16:00:00".to_string()
        ],
        "date" => &[
            "2023-08-12".to_string(),
            "2024-09-13".to_string(),
            "2025-10-14".to_string(),
            "2025-13-14".to_string()
        ]
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("time").str().strptime(
            DataType::Time,
            StrptimeOptions {
                format: Some("%H:%M:%S".to_string()),
                strict: false,
                exact: true,
                cache: true,
            },
            lit("null"),
        ),
        col("datetime").str().strptime(
            DataType::Datetime(TimeUnit::Nanoseconds, None),
            StrptimeOptions {
                format: Some("%Y-%m-%d %H:%M:%S".to_string()),
                strict: false,
                exact: false,
                cache: true,
            },
            lit("null"),
        ),
        col("date")
            .str()
            .strptime(
                DataType::Date,
                StrptimeOptions {
                    format: Some("%Y-%m-%d".to_string()),
                    strict: false,
                    exact: false,
                    cache: true,
                },
                lit("null"),
            )
            .alias("date"),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, SymmetricDistance, casted)?;

    let actual = t_casted.invoke(&lf)?.collect()?;
    let expected = r#"shape: (4, 3)
┌──────────┬─────────────────────┬────────────┐
│ time     ┆ datetime            ┆ date       │
│ ---      ┆ ---                 ┆ ---        │
│ time     ┆ datetime[ns]        ┆ date       │
╞══════════╪═════════════════════╪════════════╡
│ 14:00:00 ┆ 2023-08-12 14:00:00 ┆ 2023-08-12 │
│ 15:00:00 ┆ 2024-09-13 15:00:00 ┆ 2024-09-13 │
│ 16:00:00 ┆ 2025-10-14 16:00:00 ┆ 2025-10-14 │
│ 16:00:00 ┆ null                ┆ null       │
└──────────┴─────────────────────┴────────────┘"#;
    assert_eq!(format!("{:?}", actual), expected);
    Ok(())
}

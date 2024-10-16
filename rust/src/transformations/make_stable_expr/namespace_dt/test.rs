use polars::{
    df,
    prelude::{col, lit, DataType, IntoLazy, LazyFrame, StrptimeOptions, TimeUnit},
};

use crate::domains::{AtomDomain, LazyFrameDomain, SeriesDomain};

use super::Fallible;

pub fn example_dt_lf() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("time", AtomDomain::<String>::default()),
        SeriesDomain::new("datetime", AtomDomain::<String>::default()),
        SeriesDomain::new("date", AtomDomain::<String>::default()),
    ])?;

    let lf = df!(
        "time" => &[
            "14:00:12".to_string(),
            "15:00:23".to_string(),
            "16:00:30".to_string(),
            "16:00:22".to_string()
        ],
        "datetime" => &[
            "2023-08-12 14:00:34".to_string(),
            "2024-09-13 15:00:45".to_string(),
            "2025-10-14 16:00:56".to_string(),
            "2025-13-14 16:00:39".to_string()
        ],
        "date" => &[
            "2023-08-12".to_string(),
            "2024-09-13".to_string(),
            "2025-10-14".to_string(),
            "2025-13-14".to_string()
        ]
    )?
    .lazy();

    Ok((lf_domain, lf))
}

pub fn parse_example_dt_lf(lf: LazyFrame) -> LazyFrame {
    lf.with_columns([
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
    ])
}

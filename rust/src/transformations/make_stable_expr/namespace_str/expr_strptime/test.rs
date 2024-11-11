use std::str::FromStr;

use chrono::NaiveTime;

use crate::domains::{AtomDomain, LazyFrameDomain, SeriesDomain};
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
        "time" => &["00:00:00", "30:00:00", "00:00:80", "-10:00:00"],
        "datetime" => &["2000-01-01 00:00:00", "2000-13-01 00:00:00", "2000-01-40 00:00:00", "2000-13-01 00:00:80"],
        "date" => &["+10000-01-01", "2000-13-01", "2000-01-40", "2000-13-40"]
    )?
    .lazy();

    let casted = lf.clone().with_columns([
        col("time").str().strptime(
            DataType::Time,
            StrptimeOptions {
                format: Some("%H:%M:%S".into()),
                strict: false,
                exact: true,
                cache: true,
            },
            lit("null"),
        ),
        col("datetime").str().strptime(
            DataType::Datetime(TimeUnit::Milliseconds, None),
            StrptimeOptions {
                format: Some("%Y-%m-%d %H:%M:%S".into()),
                strict: false,
                exact: false,
                cache: true,
            },
            lit("null"),
        ),
        col("date").str().strptime(
            DataType::Date,
            StrptimeOptions {
                format: Some("%Y-%m-%d".into()),
                strict: false,
                exact: false,
                cache: true,
            },
            lit("null"),
        ),
    ]);
    let t_casted = make_stable_lazyframe(lf_domain, SymmetricDistance, casted)?;

    let observed = t_casted.invoke(&lf)?.collect()?;
    let expected = r#"shape: (4, 3)
┌──────────┬─────────────────────┬──────────────┐
│ time     ┆ datetime            ┆ date         │
│ ---      ┆ ---                 ┆ ---          │
│ time     ┆ datetime[ms]        ┆ date         │
╞══════════╪═════════════════════╪══════════════╡
│ 00:00:00 ┆ 2000-01-01 00:00:00 ┆ +10000-01-01 │
│ null     ┆ null                ┆ null         │
│ null     ┆ null                ┆ null         │
│ null     ┆ null                ┆ null         │
└──────────┴─────────────────────┴──────────────┘"#;
    assert_eq!(format!("{:?}", observed), expected);
    Ok(())
}

#[test]
fn test_strptime_datetime_edge() -> Fallible<()> {
    // this is the boundary between an acceptable and unacceptable datetime in milliseconds
    let lf = df!("datetime" => &[
        "+262142-01-01T00:00:00Z",
        "+262143-01-01T00:00:00Z",
    ])?
    .lazy()
    .with_column(
        col("datetime")
            .str()
            .strptime(
                DataType::Datetime(TimeUnit::Milliseconds, None),
                StrptimeOptions {
                    format: Some("%Y-%m-%dT%H:%M:%SZ".into()),
                    strict: false,
                    exact: true,
                    cache: true,
                },
                lit("null"),
            )
            .is_null(),
    );

    assert_eq!(lf.collect()?, df!["datetime" => [false, true]]?);

    Ok(())
}

#[test]
fn test_strptime_time_ignore_bad_date() -> Fallible<()> {
    // check that invalid dates that could overflow/panic are ignored
    //                     nano overflow        ,  milli overflow
    let lf = df!("t" => &["2300-01-01T00:00:00Z", "+262142-01-01T01:02:03Z"])?
        .lazy()
        .with_column(col("t").str().strptime(
            DataType::Time,
            StrptimeOptions {
                format: Some("%Y-%m-%dT%H:%M:%SZ".into()),
                strict: false,
                exact: true,
                cache: true,
            },
            lit("null"),
        ));

    let expected_data = vec!["00:00:00", "01:02:03"]
        .into_iter()
        .map(|s| NaiveTime::from_str(s).unwrap())
        .collect::<Vec<_>>();
    let expected = df!["t" => expected_data]?;
    assert_eq!(lf.collect()?, expected);

    Ok(())
}

#[test]
fn test_strptime_nano_panic() -> Fallible<()> {
    let closure = || {
        df!("d" => &["2300-01-01T00:00:00Z".to_string()])?
            .lazy()
            .with_column(col("d").str().strptime(
                DataType::Datetime(TimeUnit::Nanoseconds, None),
                StrptimeOptions {
                    format: Some("%Y-%m-%dT%H:%M:%SZ".into()),
                    strict: false,
                    exact: true,
                    cache: true,
                },
                lit("null"),
            ))
            .collect()
    };

    if std::panic::catch_unwind(closure).is_ok() {
        // This panic will be raised if Polars nanosecond parsing stops panicking.
        // See https://github.com/pola-rs/polars/issues/19928
        // If Polars stops panicking (this panic is raised), consider allowing nanosecond datetimes.
        panic!("expected polars to panic")
    }
    Ok(())
}

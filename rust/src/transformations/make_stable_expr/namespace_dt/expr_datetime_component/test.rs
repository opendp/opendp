use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domains::{AtomDomain, DatetimeDomain, LazyFrameDomain, SeriesDomain};
use crate::metrics::{FrameDistance, SymmetricDistance};
use crate::transformations::make_stable_lazyframe;

use super::*;
use TemporalFunction::*;

#[test]
fn test_make_expr_components() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("time", AtomDomain::<NaiveTime>::default()),
        SeriesDomain::new(
            "datetime",
            DatetimeDomain {
                time_unit: TimeUnit::Nanoseconds,
                time_zone: None,
            },
        ),
        SeriesDomain::new("date", AtomDomain::<NaiveDate>::default()),
    ])?;

    let mut data = df!(
        "time" => &[NaiveTime::from_str("00:01:02").unwrap()],
        "datetime" => &[NaiveDateTime::from_str("2000-01-02T03:04:05").unwrap()],
        "date" => &[NaiveDate::from_str("2000-01-02").unwrap()]
    )?;
    let datetime = data
        .column("datetime")?
        .cast(&DataType::Datetime(TimeUnit::Nanoseconds, None))?;
    data.with_column(datetime)?;
    let data = data.lazy();

    let plan = data.clone().with_columns([
        col("time").dt().hour().alias("time-hour"),
        col("datetime").dt().ordinal_day().alias("datetime-day"),
        col("date").dt().quarter().alias("date-quarter"),
    ]);
    let t_components = make_stable_lazyframe(lf_domain, FrameDistance(SymmetricDistance), plan)?;

    let actual = t_components.invoke(&data)?.collect()?;
    let expected = r#"shape: (1, 6)
┌──────────┬─────────────────────┬────────────┬───────────┬──────────────┬──────────────┐
│ time     ┆ datetime            ┆ date       ┆ time-hour ┆ datetime-day ┆ date-quarter │
│ ---      ┆ ---                 ┆ ---        ┆ ---       ┆ ---          ┆ ---          │
│ time     ┆ datetime[ns]        ┆ date       ┆ i8        ┆ i16          ┆ i8           │
╞══════════╪═════════════════════╪════════════╪═══════════╪══════════════╪══════════════╡
│ 00:01:02 ┆ 2000-01-02 03:04:05 ┆ 2000-01-02 ┆ 0         ┆ 2            ┆ 1            │
└──────────┴─────────────────────┴────────────┴───────────┴──────────────┴──────────────┘"#;
    assert_eq!(format!("{:?}", actual), expected);
    Ok(())
}

fn assert_temporal_op_schema<const L: usize>(df: DataFrame, ops: [TemporalFunction; L]) {
    let exprs = ops
        .iter()
        .map(|op| {
            Expr::Function {
                input: vec![col("x")],
                function: FunctionExpr::TemporalExpr(op.clone()),
            }
            .alias(op.to_string().as_str())
        })
        .collect::<Vec<_>>();

    let observed = df.lazy().select(exprs).collect_schema().unwrap();
    let expected = Arc::new(Schema::from_iter(ops.iter().map(|op| {
        Field::new(op.to_string().into(), match_datetime_component(op).unwrap())
    })));

    assert_eq!(observed, expected);
}

#[test]
fn test_time_op_schema() {
    let time_ops = [Hour, Minute, Second, Millisecond, Microsecond, Nanosecond];
    assert_temporal_op_schema(
        df!("x" => &[NaiveTime::from_str("00:01:02").unwrap()]).unwrap(),
        time_ops,
    );
}

#[test]
fn test_datetime_op_schema() {
    let datetime_ops = [
        Millennium,
        Century,
        Year,
        IsoYear,
        Quarter,
        Month,
        Week,
        WeekDay,
        Day,
        OrdinalDay,
        Hour,
        Minute,
        Second,
        Millisecond,
        Microsecond,
        Nanosecond,
    ];

    assert_temporal_op_schema(
        df!("x" => &[NaiveDateTime::from_str("2000-01-02T03:04:05").unwrap()]).unwrap(),
        datetime_ops,
    );
}

#[test]
fn test_date_op_schema() {
    let date_ops = [
        Millennium, Century, Year, IsoYear, Quarter, Month, Week, WeekDay, Day, OrdinalDay,
    ];
    assert_temporal_op_schema(
        df!("x" => &[NaiveDate::from_str("2000-01-02").unwrap()]).unwrap(),
        date_ops,
    );
}

#[test]
fn test_out_of_range_datetime_component() -> Fallible<()> {
    for time_unit in [
        TimeUnit::Milliseconds,
        TimeUnit::Microseconds,
        TimeUnit::Nanoseconds,
    ] {
        let dtype = DataType::Datetime(time_unit, None);
        // A physical timestamp that exceeds Chrono's range for ms/us is still
        // a member of DatetimeDomain. Do not evaluate unsafe extraction.
        let series = Series::new("t".into(), &[i64::MAX]).cast(&dtype)?;
        let data = DataFrame::new(vec![series.into_column()])?.lazy();
        let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
            "t",
            DatetimeDomain {
                time_unit,
                time_zone: None,
            },
        )])?;
        let query = data.clone().select([col("t").dt().year()]);
        let transformation = make_stable_lazyframe(domain, FrameDistance(SymmetricDistance), query);

        if time_unit == TimeUnit::Nanoseconds {
            let transformation = transformation?;
            // The full i64 nanosecond range remains representable in Chrono.
            let actual = transformation.invoke(&data)?.collect()?;
            assert_eq!(actual.column("t")?.i32()?.get(0), Some(2262));
        } else {
            assert!(transformation.is_err());
        }
    }
    Ok(())
}

#[test]
fn test_leap_second() -> Fallible<()> {
    let t1 = NaiveDateTime::from_str("2016-12-31T23:59:60").unwrap();
    let t2 = NaiveDateTime::from_str("2017-01-01T00:00:00").unwrap();
    // chrono distinguishes between leap seconds
    assert_ne!(t1, t2);

    // polars does not distinguish
    assert_eq!(df!("" => [t1])?, df!("" => [t2])?);

    Ok(())
}

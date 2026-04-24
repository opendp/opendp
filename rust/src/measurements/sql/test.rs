use polars::{df, frame::DataFrame, prelude::IntoLazy};

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    error::Fallible,
    measurements::make_private_lazyframe,
    measures::{Approximate, MaxDivergence},
    metrics::SymmetricDistance,
};

use super::*;

fn run_query(query: &str) -> Fallible<DataFrame> {
    let a_domain = SeriesDomain::new("a", AtomDomain::<i32>::default());
    let b_domain = SeriesDomain::new("b", AtomDomain::<String>::default());
    let input_domain = LazyFrameDomain::new(vec![a_domain, b_domain])?
        .with_margin(Margin::select().with_max_length(100))?
        .with_margin(Margin::by(["b"]).with_invariant_keys())?;

    let lf = DataFrame::from_rows_and_schema(&[], &input_domain.schema())?.lazy();
    let plan = sql_to_plan(query.to_string(), HashMap::from([("data".to_string(), lf)]))?;

    // println!("{:?}", plan.describe_plan()?);

    let m_sql = make_private_lazyframe(
        input_domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        plan,
        Some(1.0),
        Some(100),
    )?;

    let data = df!["a" => [15; 1000], "b" => [["X"; 500], ["Y"; 500]].concat()]?.lazy();
    m_sql.invoke(&data)?.collect().map_err(Into::into)
}

#[test]
fn test_sql_queries_compile_and_execute() -> Fallible<()> {
    [
        "SELECT DP_LEN(a) FROM data",
        "SELECT noise(count(*)) FROM data",
        "SELECT dp_sum(a, 12, 70) FROM data",
        "SELECT dp_sum(a, 12, 70) as sum, dp_len(a) FROM data",
        "SELECT dp_sum(a, 12, 70) FROM data GROUP BY b",
    ]
    .into_iter()
    .try_for_each(|query| run_query(query).map(|_| ()))
}

#[test]
fn test_sql_group_by_dp_sum_returns_grouped_rows() -> Fallible<()> {
    let actual = run_query("SELECT b, dp_sum(a, 12, 70) AS total FROM data GROUP BY b")?;
    let names = actual
        .get_column_names()
        .iter()
        .map(|name| name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(actual.height(), 2);
    assert_eq!(names, vec!["b", "total"]);

    Ok(())
}

#[test]
fn test_sql_group_by_multiple_dp_aggregates() -> Fallible<()> {
    let actual = run_query(
        "SELECT b, dp_sum(a, 12, 70) AS total, dp_len(a) AS count FROM data GROUP BY b",
    )?;
    let names = actual
        .get_column_names()
        .iter()
        .map(|name| name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(actual.height(), 2);
    assert_eq!(names, vec!["b", "total", "count"]);

    Ok(())
}

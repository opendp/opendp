use polars::{df, frame::DataFrame, prelude::IntoLazy};

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    error::Fallible,
    measurements::make_private_lazyframe,
    measures::{Approximate, MaxDivergence},
    metrics::SymmetricDistance,
};

use super::*;

fn check_query(query: &str) -> Fallible<()> {
    println!("running query {:?}", query);
    let a_domain = SeriesDomain::new("a", AtomDomain::<i32>::default());
    let b_domain = SeriesDomain::new("b", AtomDomain::<String>::default());
    let input_domain = LazyFrameDomain::new(vec![a_domain, b_domain])?
        .with_margin(Margin::select().with_max_length(100))?
        .with_margin(Margin::by(["b"]).with_invariant_keys())?;

    let lf = DataFrame::from_rows_and_schema(&[], &input_domain.schema())?.lazy();
    let plan = sql_to_plan(query.to_string(), HashMap::from([("data".to_string(), lf)]))?;

    println!("{:?}", plan.describe_plan()?);

    let m_sql = make_private_lazyframe(
        input_domain,
        SymmetricDistance,
        Approximate(MaxDivergence),
        plan,
        Some(1.0),
        Some(100),
    )?;

    let data = df!["a" => [15; 1000], "b" => [["X"; 500], ["Y"; 500]].concat()]?.lazy();
    let actual = m_sql.invoke(&data)?.collect()?;
    println!("{:?}", actual);

    Ok(())
}

#[test]
fn test_sql() -> Fallible<()> {
    [
        "SELECT DP_LEN(a) FROM data",
        "SELECT noise(count(*)) FROM data",
        "SELECT dp_sum(a, 12, 70) FROM data",
        // "SELECT noise(sum(clip(a, 12, 70))) FROM data",
        "SELECT dp_sum(a, 12, 70) as sum, dp_len(a) FROM data",

        // likely fails because dp_sum is a Function Expr, which is not considered an aggregation when it should be
        // likely fixed in https://github.com/pola-rs/polars/pull/23191
        "SELECT dp_sum(a, 12, 70) FROM data GROUP BY b",
        
    ]
    .into_iter()
    .try_for_each(check_query)
}

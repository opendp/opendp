use polars::df;

use crate::{
    domains::{AtomDomain, SeriesDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::SymmetricDistance,
};

use super::*;

#[test]
fn test_sql() -> Fallible<()> {
    let series_domain = SeriesDomain::new("a", AtomDomain::<bool>::default());
    let input_domain = LazyFrameDomain::new(vec![series_domain])?;
    let query = "SELECT noise(count(*)), a FROM data GROUP BY a";

    let m_sql = make_private_sql(
        input_domain,
        SymmetricDistance,
        MaxDivergence,
        query,
        Some(1.0),
        Some(100),
    )?;

    let data = df!["a" => [true; 1000]]?.lazy();
    let actual = m_sql.invoke(&data)?.collect()?;
    println!("{:?}", actual);

    Ok(())
}

use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::error::*;
use polars::prelude::*;

pub fn get_test_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("chunk_2_bool", AtomDomain::<bool>::default()),
        SeriesDomain::new("cycle_5_alpha", AtomDomain::<String>::default()),
        SeriesDomain::new("const_1f64", AtomDomain::<f64>::default()),
        SeriesDomain::new("chunk_(..10u32)", AtomDomain::<u32>::default()),
        SeriesDomain::new("cycle_(..100i32)", AtomDomain::<i32>::default()),
    ])?
    .with_margin::<&str>(
        &[],
        Margin::default()
            .with_public_lengths()
            .with_max_partition_length(1000),
    )?
    .with_margin(
        &["chunk_2_bool"],
        Margin::default()
            .with_public_lengths()
            .with_max_partition_length(500)
            .with_max_num_partitions(2)
            .with_max_partition_contributions(1),
    )?
    .with_margin(
        &["chunk_2_bool", "cycle_5_alpha"],
        Margin::default()
            .with_public_keys()
            .with_max_partition_length(200),
    )?
    .with_margin(
        &["chunk_(..10u32)"],
        Margin::default()
            .with_public_keys()
            .with_max_partition_length(100),
    )?;

    let lf = df!(
        "chunk_2_bool" => [[false; 500], [true; 500]].concat(),
        "cycle_5_alpha" => ["A", "B", "C", "D", "E"].repeat(200),
        "const_1f64" => [1.0; 1000],
        "chunk_(..10u32)" => (0..10u32).flat_map(|i| [i; 100].into_iter()).collect::<Vec<_>>(),
        "cycle_(..100i32)" => (0..100i32).cycle().take(1000).collect::<Vec<_>>()
    )?
    .lazy();

    Ok((lf_domain, lf))
}

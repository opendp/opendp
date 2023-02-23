use criterion::{criterion_group, BenchmarkId, Criterion};
use opendp::{
    measurements::{b2dp::{
        exponential_mechanism, Eta,
    }, ExponentialOptions},
    traits::samplers::GeneratorOpenDP,
};

fn utility_fn(x: &u32) -> f64 {
    *x as f64
}

fn run_mechanism(n: i64, optimize: bool) -> u32 {
    let eta = Eta::new(1, 1, 1).unwrap();
    let mut rng = GeneratorOpenDP::default();
    let mut outcomes: Vec<u32> = Vec::new();
    outcomes.push(0);
    let k: u32 = 1000; // outcome space size
    for i in 1..k {
        outcomes.push(i + (n as u32));
    }
    let options = ExponentialOptions {
        min_retries: 1,
        optimized_sample: optimize,
    };
    let result = exponential_mechanism(
        eta,
        &outcomes,
        utility_fn,
        0,
        n as u32 + k,
        k,
        &mut rng,
        options,
    )
    .unwrap();
    *result
}

fn bench_utility(c: &mut Criterion) {
    let mut group = c.benchmark_group("Utility Range");
    group.sample_size(10);
    for i in [100, 1000, 10000, 20000, 30000, 40000, 50000].iter() {
        group.bench_with_input(BenchmarkId::new("Default", i), i, |b, i| {
            b.iter(|| run_mechanism(*i))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_utility);
//criterion_main!(benches);

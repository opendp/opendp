use criterion::{criterion_group, BenchmarkId, Criterion};
use opendp::{
    measurements::b2dp::{exponential_mechanism, mechanisms::exponential::ExponentialOptions, Eta},
    traits::samplers::GeneratorOpenDP,
};

fn utility_fn(x: &u32) -> f64 {
    *x as f64
}

fn run_mechanism(n: i64) -> u32 {
    let eta = Eta::new(1, 1, 1).unwrap();
    let mut rng = GeneratorOpenDP::default();
    let mut outcomes: Vec<u32> = Vec::new();
    for i in 1..n {
        outcomes.push(i as u32);
    }
    let result = exponential_mechanism(
        eta, &outcomes, 0, n as u32, n as u32, &mut rng, None,
    )
    .unwrap();
    *result
}

fn bench_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Outcome Space Size");
    group.sample_size(10);
    for i in [100, 1000, 5000, 10000, 15000, 20000, 25000, 50000].iter() {
        group.bench_with_input(BenchmarkId::new("Default", i), i, |b, i| {
            b.iter(|| run_mechanism(*i))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_sizes);
//criterion_main!(benches);

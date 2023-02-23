use criterion::{criterion_group, BenchmarkId, Criterion};
use opendp::{
    measurements::b2dp::{exponential_mechanism, mechanisms::exponential::ExponentialOptions, Eta},
    traits::samplers::GeneratorOpenDP,
};

fn utility_fn(x: &u32) -> f64 {
    *x as f64
}

fn run_mechanism(num_retries: u32, weight_low: bool) -> u32 {
    let eta = Eta::new(1, 1, 1).unwrap();
    let mut rng = GeneratorOpenDP::default();
    let mut outcomes: Vec<u32> = Vec::new();
    let n = 256;
    let optimize = false;
    if weight_low {
        outcomes.push(0);
    } else {
        outcomes.push(1);
    }
    for _i in 1..n {
        outcomes.push(1);
    }
    let options = ExponentialOptions {
        min_retries: num_retries,
        optimized_sample: optimize,
    };
    let result = exponential_mechanism(
        eta, &outcomes, utility_fn, 0, n as u32, n as u32, &mut rng, options,
    )
    .unwrap();
    *result
}

fn bench_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Timing Channel Demo");

    for i in 1..21 {
        group.bench_with_input(BenchmarkId::new("HigherWeight", i), &i, |b, i| {
            b.iter(|| run_mechanism(*i, false))
        });
        group.bench_with_input(BenchmarkId::new("LowerWeight", i), &i, |b, i| {
            b.iter(|| run_mechanism(*i, true))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_sizes);
//criterion_main!(benches);

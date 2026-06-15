#[cfg(feature = "contrib")]
use std::hint::black_box;
#[cfg(feature = "contrib")]
use std::time::{Duration, Instant};

#[cfg(feature = "contrib")]
use opendp::{
    domains::{AtomDomain, VectorDomain},
    measurements::make_gaussian,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::samplers::sample_rounded_gaussian,
};

#[cfg(feature = "contrib")]
fn time_it(mut f: impl FnMut()) -> Duration {
    let start = Instant::now();
    f();
    start.elapsed()
}

#[cfg(feature = "contrib")]
fn ns_per_sample(duration: Duration, n: usize) -> f64 {
    duration.as_secs_f64() * 1e9 / n as f64
}

#[cfg(feature = "contrib")]
fn bench_continuous_scalar(mu: f64, scale: f64, n: usize) -> f64 {
    let duration = time_it(|| {
        for _ in 0..n {
            black_box(sample_rounded_gaussian(mu, scale).unwrap());
        }
    });
    ns_per_sample(duration, n)
}

#[cfg(feature = "contrib")]
fn bench_make_gaussian_scalar(mu: f64, scale: f64, n: usize) -> f64 {
    let meas = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        scale,
        None,
    )
    .unwrap();

    let duration = time_it(|| {
        for _ in 0..n {
            black_box(meas.invoke(&mu).unwrap());
        }
    });
    ns_per_sample(duration, n)
}

#[cfg(feature = "contrib")]
fn bench_make_gaussian_vector(mu: f64, scale: f64, n: usize) -> f64 {
    let meas = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        VectorDomain::new(AtomDomain::<f64>::new_non_nan()),
        L2Distance::<f64>::default(),
        scale,
        None,
    )
    .unwrap();
    let arg = vec![mu; n];

    let duration = time_it(|| {
        black_box(meas.invoke(&arg).unwrap());
    });
    ns_per_sample(duration, n)
}

#[cfg(feature = "contrib")]
fn main() {
    let n_continuous = 100;
    let n_scalar = 10_000;
    let n_vector = 10_000;

    println!("mu,scale,continuous_scalar_ns,make_gaussian_scalar_ns,make_gaussian_vector_ns");

    for mu in [0.0, 1.0, 1_000_000.0, 1.0e100] {
        for scale in [0.5, 1.0, 2.0, 8.0] {
            let continuous_scalar = bench_continuous_scalar(mu, scale, n_continuous);
            let make_gaussian_scalar = bench_make_gaussian_scalar(mu, scale, n_scalar);
            let make_gaussian_vector = bench_make_gaussian_vector(mu, scale, n_vector);

            println!(
                "{mu},{scale},{continuous_scalar:.1},{make_gaussian_scalar:.1},{make_gaussian_vector:.1}"
            );
        }
    }
}

#[cfg(not(feature = "contrib"))]
fn main() {
    eprintln!("run with `--features contrib` to benchmark against make_gaussian");
}

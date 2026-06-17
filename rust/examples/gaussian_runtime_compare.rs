#[cfg(feature = "contrib")]
use std::hint::black_box;
#[cfg(feature = "contrib")]
use std::time::{Duration, Instant};

#[cfg(feature = "contrib")]
use opendp::traits::samplers::{
    sample_from_uniform_bytes, sample_rounded_gaussian, sample_rounded_gaussian_f64_to_f32_native,
};
#[cfg(feature = "contrib")]
use opendp::{
    domains::AtomDomain, measurements::make_gaussian, measures::ZeroConcentratedDivergence,
    metrics::AbsoluteDistance,
};

#[cfg(feature = "contrib")]
use statrs::function::erf;

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
#[cfg(feature = "contrib")]
fn bench_continuous_f64_to_f32_native(mu: f64, scale: f64, n: usize) -> Option<f64> {
    sample_rounded_gaussian_f64_to_f32_native(mu, scale).ok()?;

    let duration = time_it(|| {
        for _ in 0..n {
            black_box(sample_rounded_gaussian_f64_to_f32_native(mu, scale).unwrap());
        }
    });
    Some(ns_per_sample(duration, n))
}

#[cfg(feature = "contrib")]
fn naive_inverse_cdf_gaussian(mu: f64, scale: f64) -> f64 {
    let bits = sample_from_uniform_bytes::<u64, 8>().unwrap() >> 11;
    let u = (bits as f64 + 0.5) * (1.0 / ((1u64 << 53) as f64));
    let z = -std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * u);
    mu + scale * z
}

#[cfg(feature = "contrib")]
fn naive_inverse_cdf_gaussian_f32(mu: f64, scale: f64) -> f32 {
    naive_inverse_cdf_gaussian(mu, scale) as f32
}

#[cfg(feature = "contrib")]
fn bench_naive_inverse_cdf(mu: f64, scale: f64, n: usize) -> f64 {
    let duration = time_it(|| {
        for _ in 0..n {
            black_box(naive_inverse_cdf_gaussian(mu, scale));
        }
    });
    ns_per_sample(duration, n)
}

#[cfg(feature = "contrib")]
fn bench_naive_inverse_cdf_f32(mu: f64, scale: f64, n: usize) -> f64 {
    let duration = time_it(|| {
        for _ in 0..n {
            black_box(naive_inverse_cdf_gaussian_f32(mu, scale));
        }
    });
    ns_per_sample(duration, n)
}

#[cfg(feature = "contrib")]
fn bench_make_gaussian_f64(mu: f64, scale: f64, n: usize) -> f64 {
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
fn bench_make_gaussian_f32(mu: f64, scale: f64, n: usize) -> Option<f64> {
    let mu = mu as f32;
    if !mu.is_finite() {
        return None;
    }

    let meas = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        AtomDomain::<f32>::new_non_nan(),
        AbsoluteDistance::<f32>::default(),
        scale,
        None,
    )
    .ok()?;

    let duration = time_it(|| {
        for _ in 0..n {
            black_box(meas.invoke(&mu).unwrap());
        }
    });
    Some(ns_per_sample(duration, n))
}

#[cfg(feature = "contrib")]
fn main() {
    let n_exact = 10_000;
    let n_naive = 1_000_000;
    let n_make_gaussian = 100_000;

    println!(
        "mu,scale,exact_unbounded_f64_ns,exact_native_f64_to_f32_ns,make_gaussian_f64_ns,make_gaussian_f32_ns,naive_inverse_cdf_f64_ns,naive_inverse_cdf_f32_ns"
    );

    for mu in [0.0, 1.0, 1_000_000.0, 1.0e100] {
        for scale in [0.5, 1.0, 2.0, 8.0] {
            let continuous_scalar = bench_continuous_scalar(mu, scale, n_exact);
            let continuous_f64_to_f32 =
                bench_continuous_f64_to_f32_native(mu, scale, n_exact).unwrap_or(f64::NAN);
            let make_gaussian_f64 = bench_make_gaussian_f64(mu, scale, n_make_gaussian);
            let make_gaussian_f32 =
                bench_make_gaussian_f32(mu, scale, n_make_gaussian).unwrap_or(f64::NAN);
            let naive_inverse_cdf = bench_naive_inverse_cdf(mu, scale, n_naive);
            let naive_inverse_cdf_f32 = bench_naive_inverse_cdf_f32(mu, scale, n_naive);

            println!(
                "{mu},{scale},{continuous_scalar:.1},{continuous_f64_to_f32:.1},{make_gaussian_f64:.1},{make_gaussian_f32:.1},{naive_inverse_cdf:.1},{naive_inverse_cdf_f32:.1}"
            );
        }
    }
}

#[cfg(not(feature = "contrib"))]
fn main() {
    eprintln!("run with `--features contrib` to benchmark against make_gaussian");
}

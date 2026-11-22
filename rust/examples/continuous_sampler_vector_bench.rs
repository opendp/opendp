//! Compare per-element cost across vector sizes. Builds on the pushed baseline too.
//! cargo run --release --no-default-features --features contrib,use-openssl --example continuous_sampler_vector_bench -- 16384 5
#[cfg(feature = "contrib")]
fn main() -> opendp::error::Fallible<()> {
    use opendp::traits::{
        CastInternalRational,
        samplers::{IidContinuousGaussian, IidContinuousLaplace, RoundedContinuousVectorSampler},
    };
    use std::{hint::black_box, time::Instant};

    fn measure(
        n: usize,
        repetitions: usize,
        vector_len: usize,
        mut sample: impl FnMut() -> opendp::error::Fallible<Vec<f64>>,
    ) -> opendp::error::Fallible<(f64, f64, f64)> {
        for _ in 0..3 {
            black_box(sample()?);
        }
        let batches = n.div_ceil(vector_len);
        let mut times = Vec::new();
        for _ in 0..repetitions {
            let start = Instant::now();
            for _ in 0..batches {
                black_box(sample()?);
            }
            times.push(start.elapsed().as_secs_f64() * 1e9 / (batches * vector_len) as f64);
        }
        times.sort_by(f64::total_cmp);
        Ok((times[times.len() / 2], times[0], times[times.len() - 1]))
    }

    let args: Vec<_> = std::env::args().collect();
    let n = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(16384);
    let repetitions = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(5);
    assert!(n > 0 && repetitions > 0);
    println!(
        "# samples_per_batch={n}, repetitions={repetitions}, warmup_calls=3, openssl={}",
        cfg!(feature = "use-openssl")
    );
    println!("mu,scale,vector_len,sampler,median_ns,min_ns,max_ns");
    for (mu, scale) in [(0.0f64, 1.0f64), (1e6, 1.0), (0.0, 1e-6)] {
        let laplace = IidContinuousLaplace::<f64>::new(scale.into_rational()?)?;
        let gaussian = IidContinuousGaussian::<f64>::new(scale.into_rational()?)?;
        for vector_len in [1, 16, 256, 4096] {
            let input = vec![mu; vector_len];
            let (median, min, max) = measure(n, repetitions, vector_len, || {
                laplace.sample_around(black_box(&input))
            })?;
            println!("{mu},{scale},{vector_len},continuous_laplace,{median:.1},{min:.1},{max:.1}");
            let (median, min, max) = measure(n, repetitions, vector_len, || {
                gaussian.sample_around(black_box(&input))
            })?;
            println!("{mu},{scale},{vector_len},continuous_gaussian,{median:.1},{min:.1},{max:.1}");
        }
    }
    Ok(())
}

#[cfg(not(feature = "contrib"))]
fn main() {
    eprintln!("run with --features contrib,use-openssl");
}

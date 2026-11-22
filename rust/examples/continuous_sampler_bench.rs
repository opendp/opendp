//! Release-mode comparison of rounded continuous and fine-grid discrete samplers.
//! Run with `cargo run --release --no-default-features --features contrib,use-openssl --example continuous_sampler_bench -- 10000 5`.
//! Append `locations` after the batch count for a fixed-scale location sweep.
//! Omit `contrib` for a discrete-only baseline (this example also builds on pre-continuous revisions).
//! Discrete samples use scale / 2^-1074, then exact rational addition and rounding.
//! The distributions are not identical: a rounded continuous law vs. a rounded lattice law.
use std::{hint::black_box, time::Instant};

use dashu::{integer::UBig, rational::RBig};
use opendp::{
    error::Fallible,
    traits::{
        CastInternalRational,
        samplers::{sample_discrete_gaussian, sample_discrete_laplace},
    },
};

#[cfg(feature = "contrib")]
use opendp::traits::samplers::{
    IidContinuousGaussian, IidContinuousLaplace, RoundedContinuousVectorSampler,
};

fn measure<T>(
    n: usize,
    repetitions: usize,
    mut sample: impl FnMut() -> Fallible<T>,
) -> Fallible<(f64, f64, f64)> {
    for _ in 0..100 {
        black_box(sample()?);
    }
    let mut times = Vec::new();
    for _ in 0..repetitions {
        let start = Instant::now();
        for _ in 0..n {
            black_box(sample()?);
        }
        times.push(start.elapsed().as_secs_f64() * 1e9 / n as f64);
    }
    times.sort_by(f64::total_cmp);
    Ok((times[times.len() / 2], times[0], times[times.len() - 1]))
}

fn main() -> Fallible<()> {
    let args: Vec<_> = std::env::args().collect();
    let n: usize = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(10_000);
    let repetitions: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(5);
    assert!(n > 0 && repetitions > 0);
    let grid = RBig::from_parts(1.into(), UBig::ONE << 1074);
    println!(
        "# n={n}, repetitions={repetitions}, openssl={}",
        cfg!(feature = "use-openssl")
    );
    println!("mu,scale,sampler,median_ns,min_ns,max_ns");
    let cases: &[(f64, f64)] = if args.get(3).is_some_and(|s| s == "locations") {
        &[
            (0.0, 1.0),
            (1.0, 1.0),
            (1e6, 1.0),
            (1e12, 1.0),
            (1e16, 1.0),
            (1e100, 1.0),
            (1e300, 1.0),
        ]
    } else {
        &[(0.0, 1.0), (1.0, 1.0), (1e6, 0.5), (0.0, 1e-6), (0.0, 1e6)]
    };
    for &(mu, scale) in cases {
        let scale_r = scale.into_rational()?;
        let mu_r = mu.into_rational()?;
        let discrete_scale = &scale_r / &grid;
        let report = |name: &str, f: &mut dyn FnMut() -> Fallible<Vec<f64>>| -> Fallible<()> {
            let (median, min, max) = measure(n, repetitions, f)?;
            println!("{mu},{scale},{name},{median:.1},{min:.1},{max:.1}");
            Ok(())
        };
        #[cfg(feature = "contrib")]
        {
            let laplace = IidContinuousLaplace::<f64>::new(scale_r.clone())?;
            let gaussian = IidContinuousGaussian::<f64>::new(scale_r)?;
            report("continuous_laplace", &mut || {
                laplace.sample_around(black_box(&[mu]))
            })?;
            report("continuous_gaussian", &mut || {
                gaussian.sample_around(black_box(&[mu]))
            })?;
        }
        report("discrete_laplace", &mut || {
            let noise = sample_discrete_laplace(black_box(discrete_scale.clone()))?;
            Ok(vec![f64::from_rational(&mu_r + RBig::from(noise) * &grid)])
        })?;
        report("discrete_gaussian", &mut || {
            let noise = sample_discrete_gaussian(black_box(discrete_scale.clone()))?;
            Ok(vec![f64::from_rational(&mu_r + RBig::from(noise) * &grid)])
        })?;
    }
    Ok(())
}

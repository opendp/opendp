//! Exact discrete sampler benchmark, also builds on the unmodified baseline.
//! cargo run --release --no-default-features --features use-openssl --example discrete_sampler_bench -- 5000 5
use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
    rbig,
};
use opendp::{
    error::Fallible,
    traits::{
        CastInternalRational,
        samplers::{sample_discrete_gaussian, sample_discrete_laplace},
    },
};
use std::{hint::black_box, time::Instant};

fn measure(
    n: usize,
    reps: usize,
    mut sample: impl FnMut() -> Fallible<()>,
) -> Fallible<(f64, f64, f64)> {
    for _ in 0..100 {
        sample()?;
    }
    let mut times = Vec::new();
    for _ in 0..reps {
        let start = Instant::now();
        for _ in 0..n {
            sample()?;
        }
        times.push(start.elapsed().as_secs_f64() * 1e9 / n as f64);
    }
    times.sort_by(f64::total_cmp);
    Ok((times[reps / 2], times[0], times[reps - 1]))
}

fn main() -> Fallible<()> {
    let args: Vec<_> = std::env::args().collect();
    let n = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(5000);
    let reps = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(5);
    assert!(n > 0 && reps > 0);
    let grid = RBig::from_parts(1.into(), UBig::ONE << 1074);
    println!(
        "# n={n}, repetitions={reps}, warmup_calls=100, openssl={}",
        cfg!(feature = "use-openssl")
    );
    println!("scale,output,sampler,median_ns,min_ns,max_ns");
    for (label, scale, float_output) in [
        ("zero", RBig::ZERO, false),
        ("0.1", rbig!(1 / 10), false),
        ("1", RBig::ONE, false),
        ("3/2", rbig!(3 / 2), false),
        ("10/3", rbig!(10 / 3), false),
        ("10", RBig::from(10), false),
        ("1e6", RBig::from(1_000_000), false),
        ("2^53", RBig::from(UBig::ONE << 53), false),
        ("2^1074", RBig::from(UBig::ONE << 1074), false),
        (
            "(2^1074+1)/3",
            RBig::from_parts(((UBig::ONE << 1074) + 1u8).into(), 3u8.into()),
            false,
        ),
        ("2^1074", RBig::from(UBig::ONE << 1074), true),
    ] {
        let samplers: [(&str, fn(RBig) -> Fallible<IBig>); 2] = [
            ("laplace", sample_discrete_laplace),
            ("gaussian", sample_discrete_gaussian),
        ];
        for (name, sample) in samplers {
            let (median, min, max) = measure(n, reps, || {
                let value = sample(black_box(scale.clone()))?;
                if float_output {
                    black_box(f64::from_rational(RBig::from(value) * &grid));
                } else {
                    black_box(value);
                }
                Ok(())
            })?;
            let output = if float_output { "f64_grid" } else { "integer" };
            println!("{label},{output},{name},{median:.1},{min:.1},{max:.1}");
        }
    }
    Ok(())
}

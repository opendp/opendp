//! Measure the per-call cost of the configured randomness backend.
//! cargo run --release --no-default-features --features use-openssl --example sampler_entropy_bench
use std::{hint::black_box, time::Instant};

use opendp::{error::Fallible, traits::samplers::fill_bytes};

fn main() -> Fallible<()> {
    let n = 100_000;
    println!("# openssl={}", cfg!(feature = "use-openssl"));
    println!("bytes_per_call,median_ns_per_call,median_ns_per_byte");
    for size in [1, 8, 64, 256, 4096] {
        let mut buffer = vec![0u8; size];
        for _ in 0..100 {
            fill_bytes(black_box(&mut buffer))?;
        }
        let mut times = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            for _ in 0..n {
                fill_bytes(black_box(&mut buffer))?;
                black_box(&buffer);
            }
            times.push(start.elapsed().as_secs_f64() * 1e9 / n as f64);
        }
        times.sort_by(f64::total_cmp);
        let median = times[times.len() / 2];
        println!("{size},{median:.1},{:.2}", median / size as f64);
    }
    Ok(())
}

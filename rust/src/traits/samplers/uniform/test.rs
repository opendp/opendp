use std::mem::size_of;

use dashu::integer::UBig;

use crate::traits::samplers::test::check_chi_square;

use super::*;

fn counts_u64<I: IntoIterator<Item = u64>>(samples: I, k: usize) -> Vec<u64> {
    let mut c = vec![0u64; k];
    for s in samples {
        c[s as usize] += 1;
    }
    c
}

fn expected_uniform_counts(n: usize, k: usize) -> Vec<f64> {
    vec![n as f64 / k as f64; k]
}

fn run_chisq_on_u64_samples<I: IntoIterator<Item = u64>>(
    samples: I,
    n: usize,
    k: usize,
) -> Fallible<()> {
    let observed = counts_u64(samples, k);
    let expected = expected_uniform_counts(n, k);
    check_chi_square(&observed, &expected)
}

fn run_chisq_uint_below<T, const N: usize>(
    upper: T,
    n: usize,
    k: usize,
    label: &str,
) -> Fallible<()>
where
    T: Integer + num::Unsigned + FromBytes<N> + Copy + TryInto<u64>,
    <T as TryInto<u64>>::Error: std::fmt::Debug,
{
    // Generate samples as u64
    let mut samples = Vec::<u64>::with_capacity(n);
    for _ in 0..n {
        let s = sample_uniform_uint_below::<T, N>(upper)?;
        let v: u64 = s.try_into().expect("sample fits u64 in tests");
        debug_assert!((v as usize) < k, "{label}: sample out of range");
        samples.push(v);
    }
    run_chisq_on_u64_samples(samples, n, k)
}

fn run_chisq_ubig_below(upper: UBig, n: usize, k: usize, label: &str) -> Fallible<()> {
    let mut samples = Vec::<u64>::with_capacity(n);
    for _ in 0..n {
        let s = sample_uniform_ubig_below(upper.clone())?;
        let v: u64 = s.to_string().parse().expect("sample fits u64 in tests");
        debug_assert!((v as usize) < k, "{label}: sample out of range");
        samples.push(v);
    }
    run_chisq_on_u64_samples(samples, n, k)
}

#[test]
fn test_sample_from_uniform_bytes() -> Fallible<()> {
    macro_rules! sample {
        ($($ty:ty)+) => {$(
            sample_from_uniform_bytes::<$ty, {size_of::<$ty>()}>()?;
        )+}
    }

    sample!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
    Ok(())
}

#[test]
fn test_uniform_uint_below_chisq_across_dtypes_and_k() -> Fallible<()> {
    const CASES: &[(usize, usize)] = &[(64, 8_000), (256, 50_000), (257, 60_000)];

    for &(k, n) in CASES {
        assert!(k >= 2);

        // u8 can only represent uppers up to 255 (u8::MAX). So skip k=256,257 for u8.
        if k <= u8::MAX as usize {
            run_chisq_uint_below::<u8, { size_of::<u8>() }>(k as u8, n, k, "u8")?;
        }

        if k <= u16::MAX as usize {
            run_chisq_uint_below::<u16, { size_of::<u16>() }>(k as u16, n, k, "u16")?;
        }

        if k <= u32::MAX as usize {
            run_chisq_uint_below::<u32, { size_of::<u32>() }>(k as u32, n, k, "u32")?;
        }

        // u64 always fits for these k
        run_chisq_uint_below::<u64, { size_of::<u64>() }>(k as u64, n, k, "u64")?;
    }

    Ok(())
}

#[test]
fn test_uniform_ubig_below_chisq_across_k() -> Fallible<()> {
    const CASES: &[(usize, usize)] = &[
        (64, 8_000),
        (256, 50_000),
        (257, 60_000),
        (1_000, 120_000), // exp=120
    ];

    for &(k, n) in CASES {
        run_chisq_ubig_below(UBig::from(k as u64), n, k, "ubig")?;
    }

    Ok(())
}

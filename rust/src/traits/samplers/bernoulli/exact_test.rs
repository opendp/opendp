use super::*;
use crate::traits::samplers::entropy::test::{TestBuffered, TestStream};

fn word_stream(words: &[u64]) -> TestStream {
    TestStream::from_bytes(words.iter().flat_map(|w| w.to_ne_bytes()).collect())
}

#[test]
fn ratio_endpoints_and_invalid_inputs_do_not_request_entropy() -> Fallible<()> {
    let mut source = TestStream::from_bytes(Vec::new());
    assert!(!sample_bernoulli_ratio_with(
        &UBig::ZERO,
        &UBig::from(17u8),
        &mut source
    )?);
    assert!(sample_bernoulli_ratio_with(
        &UBig::from(17u8),
        &UBig::from(17u8),
        &mut source
    )?);
    assert!(sample_bernoulli_ratio_with(&UBig::ONE, &UBig::ZERO, &mut source).is_err());
    assert!(sample_bernoulli_ratio_with(&UBig::from(2u8), &UBig::ONE, &mut source).is_err());
    assert!(sample_bernoulli_rational_with(-RBig::ONE, &mut source).is_err());
    assert_eq!(source.position, 0);
    Ok(())
}

#[test]
fn lazy_coin_decides_exact_prefix_boundaries() -> Fallible<()> {
    let numer = (UBig::ONE << 63) + 1u8;
    let denom = UBig::ONE << 64;
    let mut below = word_stream(&[1u64 << 63]);
    assert!(sample_bernoulli_ratio_with(&numer, &denom, &mut below)?);
    let mut above = word_stream(&[(1u64 << 63) + 1]);
    assert!(!sample_bernoulli_ratio_with(&numer, &denom, &mut above)?);
    assert_eq!((below.position, above.position), (8, 8));
    Ok(())
}

#[test]
fn lazy_coin_preserves_prefix_beyond_native_precision() -> Fallible<()> {
    for bits in [65usize, 129, 1075] {
        let numer = (UBig::ONE << (bits - 1)) + 1u8;
        let denom = UBig::ONE << bits;
        let count = bits.div_ceil(64);
        for expected in [true, false] {
            let mut words = vec![0u64; count];
            words[0] = 1u64 << 63;
            if !expected {
                words[count - 1] = u64::MAX;
            }
            let mut source = word_stream(&words);
            assert_eq!(
                sample_bernoulli_ratio_with(&numer, &denom, &mut source)?,
                expected
            );
            assert_eq!(source.position, count * 8);
        }
        // Ending the stream while the prefix still overlaps is an error,
        // not a fresh draw, false result, or a finite-precision approximation.
        let mut short = vec![0u64; count - 1];
        short[0] = 1u64 << 63;
        let mut source = word_stream(&short);
        assert!(sample_bernoulli_ratio_with(&numer, &denom, &mut source).is_err());
    }
    Ok(())
}

#[test]
fn unreduced_wide_ratio_has_correct_endpoint_decisions() -> Fallible<()> {
    let numer = UBig::ONE << 120;
    let denom = UBig::ONE << 121;
    assert!(sample_bernoulli_ratio_with(
        &numer,
        &denom,
        &mut word_stream(&[0])
    )?);
    assert!(!sample_bernoulli_ratio_with(
        &numer,
        &denom,
        &mut word_stream(&[u64::MAX])
    )?);
    Ok(())
}

#[test]
fn buffered_and_direct_rational_coins_have_identical_traces() -> Fallible<()> {
    let mut direct = TestStream::new(19);
    let mut buffered = TestBuffered::<7>::new(19);
    let numer = (UBig::ONE << 1073) + 17u8;
    let denom = (UBig::ONE << 1074) + 91u8;
    for _ in 0..1000 {
        let a = sample_bernoulli_ratio_with(&numer, &denom, &mut direct)?;
        let b = sample_bernoulli_ratio_with(&numer, &denom, &mut buffered)?;
        assert_eq!(a, b);
        assert_eq!(direct.position, buffered.consumed());
    }
    Ok(())
}

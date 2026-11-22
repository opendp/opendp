use super::*;
use rand::{RngCore, SeedableRng};

// Test-only fixed byte stream: its byte order does not depend on request sizes.
// Seeding a generator separately for each fill would not have this property.
pub(crate) struct TestStream {
    bytes: Vec<u8>,
    pub(crate) position: usize,
}

impl TestStream {
    pub(crate) fn new(seed: u64) -> Self {
        let mut bytes = vec![0u8; 1 << 20];
        rand::rngs::StdRng::seed_from_u64(seed).fill_bytes(&mut bytes);
        Self { bytes, position: 0 }
    }
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes, position: 0 }
    }
}

impl EntropySource for TestStream {
    fn fill(&mut self, output: &mut [u8]) -> Fallible<()> {
        if output.len() > self.bytes.len() - self.position {
            return fallible!(FailedFunction, "test stream exhausted");
        }
        output.copy_from_slice(&self.bytes[self.position..self.position + output.len()]);
        self.position += output.len();
        Ok(())
    }
}

pub(crate) struct TestBuffered<const N: usize> {
    buffer: BufferedEntropy<N>,
    source: TestStream,
}

impl<const N: usize> TestBuffered<N> {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            buffer: BufferedEntropy::new(),
            source: TestStream::new(seed),
        }
    }
    pub(crate) fn consumed(&self) -> usize {
        self.source.position - (N - self.buffer.cursor)
    }
}
impl<const N: usize> EntropySource for TestBuffered<N> {
    fn fill(&mut self, output: &mut [u8]) -> Fallible<()> {
        self.buffer.fill_with(output, |out| self.source.fill(out))
    }
}

#[test]
fn buffering_preserves_byte_order_and_uses_each_byte_once() -> Fallible<()> {
    fn check<const N: usize>() -> Fallible<()> {
        let mut source = TestStream::new(42);
        let mut expected = TestStream::new(42);
        let mut buffer = BufferedEntropy::<N>::new();
        for size in [0, 1, 8, 3, 255, 256, 257, 513, 1, 64] {
            let mut actual = vec![0u8; size];
            let mut direct = vec![0u8; size];
            buffer.fill_with(&mut actual, |out| source.fill(out))?;
            expected.fill(&mut direct)?;
            assert_eq!(actual, direct);
        }
        assert_eq!(source.position, expected.position.div_ceil(N) * N);
        Ok(())
    }
    check::<1>()?;
    check::<7>()?;
    check::<8>()?;
    check::<64>()?;
    check::<256>()?;
    Ok(())
}

#[test]
fn empty_reads_are_lazy_and_do_not_request_entropy() -> Fallible<()> {
    let mut buffer = BufferedEntropy::<8>::new();
    buffer.fill_with(&mut [], |_| panic!("empty read requested entropy"))?;
    assert_eq!(buffer.cursor, 8);
    Ok(())
}

#[test]
fn failed_partial_refill_is_not_published_or_reused() -> Fallible<()> {
    let mut buffer = BufferedEntropy::<8>::new();
    let mut calls = 0;
    let mut source = |out: &mut [u8]| -> Fallible<()> {
        calls += 1;
        if calls == 2 {
            out[..2].fill(0xee);
            return fallible!(FailedFunction, "partial refill failure");
        }
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = (calls * 10 + i) as u8;
        }
        Ok(())
    };
    let mut first = [0u8; 3];
    buffer.fill_with(&mut first, &mut source)?;
    assert_eq!(first, [10, 11, 12]);
    assert!(buffer.fill_with(&mut [0u8; 7], &mut source).is_err());
    assert_eq!(buffer.cursor, 8);
    let mut retry = [0u8; 8];
    buffer.fill_with(&mut retry, &mut source)?;
    assert_eq!(retry, [30, 31, 32, 33, 34, 35, 36, 37]);
    assert_eq!(calls, 3);
    Ok(())
}

#[test]
fn word_and_coin_draws_do_not_overlap() -> Fallible<()> {
    let mut source = TestStream::from_bytes((1..=10).collect());
    assert!(source.coin()?);
    assert_eq!(source.word()?, u64::from_ne_bytes([2, 3, 4, 5, 6, 7, 8, 9]));
    assert!(!source.coin()?);
    assert_eq!(source.position, 10);
    Ok(())
}

#[test]
#[should_panic(expected = "entropy buffer must not be empty")]
fn zero_capacity_is_rejected() {
    let _ = BufferedEntropy::<0>::new();
}

#[test]
fn source_aware_bounded_u8_draws_are_exhaustively_unbiased() -> Fallible<()> {
    assert_eq!(
        sample_uniform_uint_below_with::<u8, 1>(1, &mut |_| panic!("bound one requested entropy"))?,
        0
    );
    for upper in 2u8..=u8::MAX {
        let threshold = u8::MAX - u8::MAX % upper;
        let mut accepted_counts = vec![0usize; upper as usize];
        for raw in 0u8..=u8::MAX {
            let mut calls = 0;
            let out = sample_uniform_uint_below_with::<u8, 1>(upper, &mut |bytes| {
                assert_eq!(bytes.len(), 1);
                bytes[0] = if calls == 0 { raw } else { 0 };
                calls += 1;
                Ok(())
            })?;
            if raw < threshold {
                assert_eq!(out, raw % upper);
                assert_eq!(calls, 1);
                accepted_counts[out as usize] += 1;
            } else {
                assert_eq!(out, 0);
                assert_eq!(calls, 2);
            }
        }
        assert!(
            accepted_counts
                .iter()
                .all(|&n| n == (threshold / upper) as usize)
        );
    }
    Ok(())
}

#[test]
fn bounded_draws_reject_out_of_range_bytes_and_propagate_failure() -> Fallible<()> {
    let mut bytes = u64::MAX.to_ne_bytes().to_vec();
    bytes.extend(7u64.to_ne_bytes());
    let mut source = TestStream::from_bytes(bytes);
    assert_eq!(source.uniform_u64_below(10)?, 7);
    assert_eq!(source.position, 16);
    assert!(
        TestStream::from_bytes(u64::MAX.to_ne_bytes().to_vec())
            .uniform_u64_below(10)
            .is_err()
    );

    // Exercise the same source-aware core on an upper bound beyond native size.
    let upper = (UBig::ONE << 80) + 3u8;
    let mut bytes = vec![0xff; 11]; // always beyond the rejection threshold
    bytes.extend([0u8; 10]);
    bytes.push(7);
    let mut source = TestStream::from_bytes(bytes);
    assert_eq!(source.uniform_ubig_below(upper.clone())?, UBig::from(7u8));
    assert_eq!(source.position, 22);
    assert!(
        TestStream::from_bytes(vec![0xff; 11])
            .uniform_ubig_below(upper)
            .is_err()
    );
    Ok(())
}

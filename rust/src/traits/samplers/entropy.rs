//! Invocation-local entropy for the exact continuous samplers.
//! No global/TLS cache, cloneable random state, or replacement randomness source.
use dashu::integer::UBig;

use super::{
    fill_bytes,
    uniform::{sample_uniform_ubig_below_with, sample_uniform_uint_below_with},
};
use crate::error::Fallible;

pub(crate) trait EntropySource {
    // On error, callers must discard output, which may be partially filled.
    fn fill(&mut self, output: &mut [u8]) -> Fallible<()>;

    #[inline]
    fn word(&mut self) -> Fallible<u64> {
        let mut bytes = [0u8; 8];
        self.fill(&mut bytes)?;
        Ok(u64::from_ne_bytes(bytes))
    }

    #[inline]
    fn coin(&mut self) -> Fallible<bool> {
        let mut byte = [0u8; 1];
        self.fill(&mut byte)?;
        Ok(byte[0] & 1 != 0)
    }

    // The bounded helpers require a positive upper bound, as their system-backed
    // counterparts do. Continuous sampling calls these with denominators >= 2.
    fn uniform_u64_below(&mut self, upper: u64) -> Fallible<u64> {
        sample_uniform_uint_below_with::<u64, 8>(upper, &mut |out| self.fill(out))
    }

    fn uniform_ubig_below(&mut self, upper: UBig) -> Fallible<UBig> {
        sample_uniform_ubig_below_with(upper, &mut |out| self.fill(out))
    }
}

pub(crate) struct SystemEntropy;

#[cfg(not(feature = "use-openssl"))]
impl SystemEntropy {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl EntropySource for SystemEntropy {
    #[inline]
    fn fill(&mut self, output: &mut [u8]) -> Fallible<()> {
        fill_bytes(output)
    }
}

#[cfg(any(feature = "use-openssl", test))]
pub(crate) struct BufferedEntropy<const N: usize> {
    buffer: [u8; N],
    cursor: usize,
}

// Shared by rejection tests, final refinement, and elements of one vector call.
// The buffer is created lazily with respect to entropy, and dropped at call exit.
#[cfg(feature = "use-openssl")]
pub(crate) type SamplerEntropy = BufferedEntropy<256>;

// The rand fallback already amortizes its generator internally. An additional
// per-call buffer regresses singleton latency, so leave that backend direct.
#[cfg(not(feature = "use-openssl"))]
pub(crate) type SamplerEntropy = SystemEntropy;

#[cfg(any(feature = "use-openssl", test))]
impl<const N: usize> BufferedEntropy<N> {
    pub(crate) fn new() -> Self {
        assert!(N > 0, "entropy buffer must not be empty");
        Self {
            buffer: [0u8; N],
            cursor: N,
        }
    }

    #[inline]
    fn fill_with(
        &mut self,
        mut output: &mut [u8],
        mut source: impl FnMut(&mut [u8]) -> Fallible<()>,
    ) -> Fallible<()> {
        while !output.is_empty() {
            if self.cursor == N {
                // Leave cursor exhausted until the entire refill succeeds. A
                // failed/partial refill must never expose stale or partial bytes.
                source(&mut self.buffer)?;
                self.cursor = 0;
            }
            let count = output.len().min(N - self.cursor);
            output[..count].copy_from_slice(&self.buffer[self.cursor..self.cursor + count]);
            self.cursor += count;
            output = &mut output[count..];
        }
        Ok(())
    }
}

#[cfg(any(feature = "use-openssl", test))]
impl<const N: usize> EntropySource for BufferedEntropy<N> {
    #[inline]
    fn fill(&mut self, output: &mut [u8]) -> Fallible<()> {
        self.fill_with(output, fill_bytes)
    }
}

#[cfg(test)]
pub(crate) mod test;

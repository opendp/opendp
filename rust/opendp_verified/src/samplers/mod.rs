mod bernoulli;
pub use bernoulli::*;

mod uniform;
pub use uniform::*;

use crate::error::Fallible;

#[cfg(feature = "use-openssl")]
/// Fill a byte buffer with fresh uniformly random bytes.
///
/// # Proof Definition
/// Return either `Err(e)` if entropy generation fails,
/// or `Ok(())`, in which case every byte in `buffer` is overwritten with
/// uniformly random bits.
pub fn fill_bytes(buffer: &mut [u8]) -> Fallible<()> {
    use openssl::rand::rand_bytes;

    if let Err(openssl_error) = rand_bytes(buffer) {
        fallible!(EntropyExhausted, "OpenSSL error: {:?}", openssl_error)
    } else {
        Ok(())
    }
}

#[cfg(not(feature = "use-openssl"))]
/// Fill a byte buffer with fresh uniformly random bytes.
///
/// # Proof Definition
/// Return either `Err(e)` if entropy generation fails,
/// or `Ok(())`, in which case every byte in `buffer` is overwritten with
/// uniformly random bits.
pub fn fill_bytes(buffer: &mut [u8]) -> Fallible<()> {
    use rand::Rng;

    if let Err(rand_error) = rand::thread_rng().try_fill(buffer) {
        fallible!(EntropyExhausted, "Rand error: {:?}", rand_error)
    } else {
        Ok(())
    }
}

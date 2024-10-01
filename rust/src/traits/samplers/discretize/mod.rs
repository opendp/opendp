use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;

use crate::error::Fallible;
use crate::traits::samplers::sample_discrete_laplace;

use super::sample_discrete_gaussian;

#[cfg(test)]
mod test;

#[allow(non_snake_case)]
/// Sample from the discrete laplace distribution on $\mathbb{Z} \cdot 2^k$.
///
/// Implemented for rational numbers.
///
/// k can be chosen to be very negative,
/// to get an arbitrarily fine approximation to continuous laplacian noise.
///
/// # Proof Definition
/// For any setting of the input arguments, return either
/// `Err(e)` if there is insufficient system entropy, or
/// `Ok(sample)`, where `sample` is distributed according to a modified discrete_laplace(`shift`, `scale`).
///
/// The modifications to the discrete laplace are as follows:
/// - the `shift` is rounded to the nearest multiple of $2^k$
/// - the noise granularity is in increments of $2^k$.
pub fn sample_discrete_laplace_Z2k(shift: RBig, scale: RBig, k: i32) -> Fallible<RBig> {
    // integerize
    let mut i = find_nearest_multiple_of_2k(shift, k);

    // sample from the discrete laplace on ℤ*2^k
    i += sample_discrete_laplace(shr(scale, k))?;

    // postprocess! int -> rational
    Ok(x_mul_2k(i, k))
}

#[allow(non_snake_case)]
/// Sample from the discrete gaussian distribution on $\mathbb{Z} \cdot 2^k$.
///
/// Implemented for rational numbers.
///
/// k can be chosen to be very negative,
/// to get an arbitrarily fine approximation to continuous gaussian noise.
///
/// # Proof Definition
/// For any setting of the input arguments, return either
/// `Err(e)` if there is insufficient system entropy, or
/// `Ok(sample)`, where `sample` is distributed according to a modified discrete_gaussian(`shift`, `scale`).
///
/// The modifications to the discrete gaussian are as follows:
/// - the `shift` is rounded to the nearest multiple of $2^k$
/// - the noise granularity is in increments of $2^k$.
pub fn sample_discrete_gaussian_Z2k(shift: RBig, scale: RBig, k: i32) -> Fallible<RBig> {
    // integerize
    let mut i = find_nearest_multiple_of_2k(shift, k);

    // sample from the discrete gaussian on ℤ*2^k
    i += sample_discrete_gaussian(shr(scale, k))?;

    // postprocess! int -> rational
    Ok(x_mul_2k(i, k))
}

fn shr(lhs: RBig, rhs: i32) -> RBig {
    let (mut num, mut den) = lhs.into_parts();
    if rhs < 0 {
        num <<= -rhs as usize;
    } else {
        den <<= rhs as usize;
    }

    RBig::from_parts(num, den)
}

/// Find index of nearest multiple of $2^k$ from x.
///
/// # Proof Definition
/// For any setting of input arguments, return the integer $argmin_i |i 2^k - x|$.
fn find_nearest_multiple_of_2k(x: RBig, k: i32) -> IBig {
    // exactly compute shift/2^k and break into fractional parts
    let (sx, sy) = shr(x, k).into_parts();

    // argmin_i |i * 2^k - sx/sy|, the index of nearest multiple of 2^k
    let offset = &sy / UBig::from(2u8) * sx.sign();
    (sx + offset) / sy
}

/// Exactly multiply x by 2^k.
///
/// This is a postprocessing operation.
fn x_mul_2k(x: IBig, k: i32) -> RBig {
    if k > 0 {
        RBig::from(x << k as usize)
    } else {
        RBig::from_parts(x, UBig::ONE << -k as usize)
    }
}

use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
    rbig,
};

use crate::{
    error::Fallible,
    samplers::{sample_bernoulli_rational, sample_geometric_exp_fast},
};

/// Sample exactly from the discrete Laplace distribution with scale `numer/denom`.
///
/// This is the Canonne–Kamath–Steinke construction: draw a geometric magnitude
/// with failure parameter `1 - exp(-denom/numer)`, draw a fair sign bit, and
/// reject the (negative, zero) outcome so that zero is not double-counted.
///
/// # Proof Definition
/// For any strictly positive `numer` and `denom`,
/// `sample_discrete_laplace` either returns `Err(e)` due to a lack of system
/// entropy, or `Ok(out)`, where `out` is distributed as
/// `DiscreteLaplace(numer/denom)` on `ℤ`, with mass function
/// `P[out = z] = (exp(denom/numer) - 1) / (exp(denom/numer) + 1)
///               · exp(-|z|·denom/numer)`.
pub fn sample_discrete_laplace(numer: UBig, denom: UBig) -> Fallible<IBig> {
    let x = RBig::from_parts(denom.as_ibig().clone(), numer);
    loop {
        let magnitude = sample_geometric_exp_fast(x.clone())?;
        let negate = sample_bernoulli_rational(rbig!(1 / 2))?;
        if !(negate && magnitude.is_zero()) {
            return Ok(if negate {
                -magnitude.as_ibig().clone()
            } else {
                magnitude.as_ibig().clone()
            });
        }
    }
}

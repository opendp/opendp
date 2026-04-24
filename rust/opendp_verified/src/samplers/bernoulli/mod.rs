use dashu::{
    integer::{Sign, UBig},
    rational::RBig,
    rbig,
};

use crate::{
    error::Fallible, samplers::sample_uniform_ubig_below, utilities::div_rbig_by_ubig_exact,
};

/// Sample a single bit with rational probability of success.
///
/// # Arguments
/// * `prob` - The desired probability of success.
///
/// # Proof Definition
/// For any setting of `prob` within `[0, 1]`,
/// return either `Err(e)` if there is insufficient entropy,
/// or `Ok(out)`, where `out` is `true` with probability `prob`, otherwise `false`.
pub fn sample_bernoulli_rational(prob: RBig) -> Fallible<bool> {
    let (numer, denom) = prob.into_parts();
    let (Sign::Positive, numer) = numer.into_parts() else {
        return fallible!(FailedFunction, "numerator must not be negative");
    };
    if numer > denom {
        return fallible!(FailedFunction, "prob must not be greater than one");
    }

    sample_uniform_ubig_below(denom).map(|sample| numer > sample)
}

/// Sample a single bit with probability `exp(-x)` for `x` in `[0, 1]`.
///
/// # Proof Definition
/// For any setting of `x` within `[0, 1]`,
/// return either `Err(e)` if there is insufficient entropy,
/// or `Ok(out)`, where `out` is `true` with probability `exp(-x)`.
fn sample_bernoulli_exp1(x: RBig) -> Fallible<bool> {
    let (numer_signed, denom) = x.into_parts();
    let (Sign::Positive, numer) = numer_signed.into_parts() else {
        return fallible!(FailedFunction, "x must be in [0, 1]");
    };

    let mut k = UBig::ONE;
    loop {
        let x_div_k = div_rbig_by_ubig_exact(&numer, &denom, &k);

        if sample_bernoulli_rational(x_div_k)? {
            k += UBig::ONE;
        } else {
            return Ok(k % 2u8 == 1);
        }
    }
}

/// Sample a single bit with probability `exp(-x)` for non-negative `x`.
///
/// # Proof Definition
/// For any non-negative setting of `x`,
/// return either `Err(e)` if there is insufficient entropy,
/// or `Ok(out)`, where `out` is `true` with probability `exp(-x)`.
pub fn sample_bernoulli_exp(mut x: RBig) -> Fallible<bool> {
    while x > RBig::ONE {
        if sample_bernoulli_exp1(rbig!(1))? {
            x -= RBig::ONE;
        } else {
            return Ok(false);
        }
    }
    sample_bernoulli_exp1(x)
}

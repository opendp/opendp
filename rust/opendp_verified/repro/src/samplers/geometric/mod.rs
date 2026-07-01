use dashu::{integer::UBig, rational::RBig};

use crate::{
    error::Fallible,
    samplers::{sample_bernoulli_exp, sample_uniform_ubig_below},
};

/// Sample exactly from the geometric distribution with failure parameter
/// `1 - exp(-x)`.
///
/// This is the simple rejection-free reference implementation: repeatedly
/// sample `Bernoulli(exp(-x))`, incrementing the counter on `true` and
/// stopping on the first `false`.
///
/// # Proof Definition
/// For any strictly positive rational `x`,
/// `sample_geometric_exp_slow` either returns `Err(e)` due to a lack of
/// system entropy, or `Ok(out)`, where `out` is distributed as the
/// geometric law on `Nat` with mass function
/// `P[out = n] = exp(-x)^n * (1 - exp(-x))`.
///
/// The `x = 0` case is intentionally excluded from the proof definition,
/// because the loop does not terminate there: `sample_bernoulli_exp(0)`
/// returns `true` almost surely.
pub fn sample_geometric_exp_slow(x: RBig) -> Fallible<UBig> {
    let mut k = UBig::ZERO;
    loop {
        if sample_bernoulli_exp(x.clone())? {
            k += UBig::ONE;
        } else {
            return Ok(k);
        }
    }
}

fn sample_geometric_exp_fast_loop(
    denom: UBig,
    numer: dashu::integer::IBig,
    u: UBig,
) -> Fallible<UBig> {
    if sample_bernoulli_exp(RBig::from_parts(u.as_ibig().clone(), denom.clone()))? {
        let v2 = sample_geometric_exp_slow(RBig::ONE)?;
        Ok((v2 * denom + u) / numer.into_parts().1)
    } else {
        let next_u = sample_uniform_ubig_below(denom.clone())?;
        sample_geometric_exp_fast_loop(denom, numer, next_u)
    }
}

/// Sample exactly from the geometric distribution with failure parameter
/// `1 - exp(-x)`.
///
/// This is the fast reference implementation: draw one uniform residue, keep
/// sampling `Bernoulli(exp(-x))` until the first failure, then combine the
/// geometric count with the final residue to recover the output.
///
/// # Proof Definition
/// For any non-negative rational `x`,
/// `sample_geometric_exp_fast` either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as the geometric law on `Nat` with
/// mass function `P[out = n] = exp(-x)^n * (1 - exp(-x))`.
pub fn sample_geometric_exp_fast(x: RBig) -> Fallible<UBig> {
    if x.is_zero() {
        return Ok(UBig::ZERO);
    }

    let (numer, denom) = x.into_parts();
    let u = sample_uniform_ubig_below(denom.clone())?;
    sample_geometric_exp_fast_loop(denom, numer, u)
}

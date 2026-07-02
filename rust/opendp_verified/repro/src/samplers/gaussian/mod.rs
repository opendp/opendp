use dashu::{base::UnsignedAbs, integer::IBig, integer::UBig, rational::RBig};

use crate::{
    error::Fallible,
    samplers::{sample_bernoulli_exp, sample_discrete_laplace},
};

/// Sample exactly from the discrete Gaussian distribution with standard
/// deviation `numer/denom` (variance `(numer/denom)²`).
///
/// This is the Canonne–Kamath–Steinke rejection construction: propose a
/// discrete Laplace candidate `Y` with scale `t = ⌊numer/denom⌋ + 1`, and
/// accept it with probability
/// `exp(-(|Y|·t·den − num)² / (2·num·t²·den))`
/// where `num = numer²` and `den = denom²`.
///
/// # Proof Definition
/// For any strictly positive `numer` and `denom`,
/// `sample_discrete_gaussian` either returns `Err(e)` due to a lack of system
/// entropy, or `Ok(out)`, where `out` is distributed as the discrete Gaussian
/// on `ℤ` with mass function
/// `P[out = z] ∝ exp(-z²·denom²/(2·numer²))`.
pub fn sample_discrete_gaussian(numer: UBig, denom: UBig) -> Fallible<IBig> {
    let t = numer.clone() / denom.clone() + UBig::ONE;
    let num = numer.clone() * numer;
    let den = denom.clone() * denom;
    loop {
        let y = sample_discrete_laplace(t.clone(), UBig::ONE)?;
        let lhs = y.clone().unsigned_abs() * t.clone() * den.clone();
        let diff = lhs.as_ibig().clone() - num.as_ibig().clone();
        let n_abs = diff.unsigned_abs();
        let n = n_abs.clone() * n_abs;
        let d = (UBig::ONE + UBig::ONE) * num.clone() * t.clone() * t.clone() * den.clone();
        if sample_bernoulli_exp(RBig::from_parts(n.as_ibig().clone(), d))? {
            return Ok(y);
        }
    }
}

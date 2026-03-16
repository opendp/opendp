use super::bernoulli::sample_bernoulli_rational;
use crate::{error::Fallible, traits::samplers::sample_geometric_exp_fast};

use dashu::{integer::UBig, rational::RBig};

#[cfg(test)]
mod test;

/// Sample exactly from the ordinary negative binomial distribution on $\mathbb{N}_0$
/// with positive integer shape `shape` and success probability $1 - e^{-x}$,
/// parameterized as the number of failures before the `shape`-th success.
///
/// Equivalently, if $G_1, \ldots, G_{\texttt{shape}}$ are iid geometric random variables on
/// $\mathbb{N}_0$ with
/// $\Pr[G_i = k] = (1 - e^{-x}) e^{-xk}$,
/// then the output is distributed as
/// $G_1 + \cdots + G_{\texttt{shape}}$.
///
/// # Proof Definition
/// For any positive finite integer `shape` and positive finite rational `x`,
/// either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as a negative binomial random variable
/// on $\mathbb{N}_0$ with shape `shape` and success probability $1 - e^{-x}$,
/// parameterized as the number of failures before the `shape`-th success.
pub fn sample_negative_binomial_integer(mut shape: UBig, x: RBig) -> Fallible<UBig> {
    debug_assert!(!shape.is_zero(), "shape must be positive");
    debug_assert!(x > RBig::ZERO, "x must be positive");

    let mut total = UBig::ZERO;

    while !shape.is_zero() {
        total += sample_geometric_exp_fast(x.clone())?;
        shape -= UBig::ONE;
    }

    Ok(total)
}

/// Sample exactly from a Bernoulli distribution with success probability
/// $\prod_{i=0}^{k-1} \frac{\eta + i}{m + i}$.
///
/// # Proof Definition
/// For any finite `k` in $\mathbb{N}_0$, positive finite rational `eta_numer`,
/// positive finite rational `eta_denom`, and positive finite integer `base_denom`
/// such that
/// $m = \frac{\texttt{base\_denom}}{\texttt{eta\_denom}} \in \mathbb{N}_{>0}$ and
/// $\eta = \frac{\texttt{eta\_numer}}{\texttt{eta\_denom}} \in \mathbb{Q}_{>0}$,
/// either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as a Bernoulli random variable with success
/// probability $\prod_{i=0}^{k-1} \frac{\eta + i}{m + i}$.
fn sample_integer_envelope_accept(
    k: &UBig,
    eta_numer: &UBig,
    eta_denom: &UBig,
    base_denom: &UBig,
) -> Fallible<bool> {
    debug_assert!(!eta_numer.is_zero(), "eta must be positive");
    debug_assert!(!eta_denom.is_zero(), "eta denominator must be positive");
    debug_assert!(!base_denom.is_zero(), "base denominator must be positive");

    let mut i = UBig::ZERO;
    let mut numer = eta_numer.clone();
    let mut denom = base_denom.clone();

    while &i < k {
        let p = RBig::from_parts(numer.clone().into(), denom.clone());
        if !sample_bernoulli_rational(p)? {
            return Ok(false);
        }
        numer += eta_denom;
        denom += eta_denom;
        i += UBig::ONE;
    }

    Ok(true)
}

/// Sample exactly from the ordinary negative binomial distribution on $\mathbb{N}_0$
/// with positive rational shape `eta` and success probability $1 - e^{-x}$,
/// parameterized as the number of failures before the `eta`-th success.
///
/// When `eta` is not an integer, this is implemented by rejection sampling from the
/// corresponding distribution with shape $\lceil \eta \rceil$.
///
/// # Proof Definition
/// For any positive finite rationals `eta` and `x`,
/// either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as a negative binomial random variable
/// on $\mathbb{N}_0$ with shape `eta` and success probability $1 - e^{-x}$,
/// parameterized as the number of failures before the `eta`-th success.
pub fn sample_negative_binomial_rational(eta: RBig, x: RBig) -> Fallible<UBig> {
    debug_assert!(eta > RBig::ZERO, "eta must be positive");
    debug_assert!(x > RBig::ZERO, "x must be positive");

    let m = eta.ceil().into_parts().1;

    let (eta_numer_ibig, eta_denom) = eta.into_parts();
    let eta_numer = eta_numer_ibig.into_parts().1;

    let base_denom = eta_denom.clone() * m.clone();

    // Fast path for positive integer eta: no rejection needed.
    if eta_denom == UBig::ONE {
        return sample_negative_binomial_integer(m, x);
    }

    loop {
        let k = sample_negative_binomial_integer(m.clone(), x.clone())?;
        if sample_integer_envelope_accept(&k, &eta_numer, &eta_denom, &base_denom)? {
            return Ok(k);
        }
    }
}

/// Sample exactly from the truncated negative binomial distribution on $\mathbb{N}_{>0}$
/// with positive rational shape `eta` and success probability $1 - e^{-x}$,
/// obtained by conditioning the corresponding ordinary negative binomial law on `out > 0`.
///
/// Equivalently, with $\gamma = 1 - e^{-x}$, this is the distribution $D_{\eta,\gamma}$
/// from Definition 1 of Papernot--Steinke for `eta > 0`.
///
/// # Proof Definition
/// For any positive finite rationals `eta` and `x`,
/// either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as a negative binomial random variable
/// with shape `eta` and success probability $1 - e^{-x}$,
/// parameterized as the number of failures before the `eta`-th success,
/// conditioned on `out > 0`.
pub(crate) fn sample_truncated_negative_binomial_rational(eta: RBig, x: RBig) -> Fallible<UBig> {
    debug_assert!(eta > RBig::ZERO, "eta must be positive");
    debug_assert!(x > RBig::ZERO, "x must be positive");

    loop {
        let k = sample_negative_binomial_rational(eta.clone(), x.clone())?;
        if !k.is_zero() {
            return Ok(k);
        }
    }
}

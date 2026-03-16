use super::sample_bernoulli_rational;
use crate::{error::Fallible, traits::samplers::sample_geometric_exp_fast};

use dashu::{integer::UBig, rational::RBig};
use opendp_derive::proven;

#[cfg(test)]
mod test;

#[proven]
/// Sample exactly from the logarithmic distribution on $\mathbb{N}_{>0}$
/// with parameter $e^{-x}$.
///
/// Equivalently, for each $k \in \mathbb{N}_{>0}$, the output satisfies
/// $\Pr[\mathrm{out} = k] = \frac{e^{-xk}}{k \log(1 / (1 - e^{-x}))}$.
///
/// With $\gamma = 1 - e^{-x}$, this is the distribution $D_{0,\gamma}$
/// from Definition 1 of Papernot--Steinke.
///
/// # Proof Definition
/// For any positive finite rational `x`,
/// either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed on $\mathbb{N}_{>0}$ with
/// $\Pr[\mathrm{out} = k] = \frac{e^{-xk}}{k \log(1 / (1 - e^{-x}))}$.
pub(crate) fn sample_logarithmic_exp(x: RBig) -> Fallible<UBig> {
    debug_assert!(x > RBig::ZERO, "x must be positive");
    if x <= RBig::ZERO {
        return fallible!(FailedFunction, "x must be positive");
    }

    loop {
        let k = sample_geometric_exp_fast(x.clone())?;
        if k.is_zero() {
            continue;
        }
        if sample_bernoulli_rational(RBig::from_parts(1.into(), k.clone()))? {
            return Ok(k);
        }
    }
}

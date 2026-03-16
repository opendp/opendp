use super::sample_bernoulli_rational;
use crate::error::Fallible;

use dashu::{integer::UBig, rational::RBig};
use opendp_derive::proven;

#[cfg(test)]
mod test;

#[proven]
/// Sample exactly from the Poisson distribution on $\mathbb{N}_0$
/// with mean `lambda`, under the precondition $0 \le \texttt{lambda} < 1$.
///
/// # Proof Definition
/// For any finite rational `lambda` satisfying $0 \le \texttt{lambda} < 1$,
/// either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Poisson(\texttt{lambda})$.
pub(crate) fn sample_poisson_0_1(lambda: RBig) -> Fallible<UBig> {
    debug_assert!(lambda >= RBig::ZERO, "lambda must be non-negative");
    debug_assert!(lambda < RBig::ONE, "lambda must be less than one");

    if lambda.is_zero() {
        return Ok(UBig::ZERO);
    }

    loop {
        // Proposal: geometric on {0,1,2,...} with P[K = k] = (1-lambda) lambda^k
        let mut k = UBig::ZERO;
        while sample_bernoulli_rational(lambda.clone())? {
            k += UBig::ONE;
        }

        // Accept with probability 1 / k!
        let mut i = UBig::from(2u8);
        let mut accept = true;
        while i <= k {
            if !sample_bernoulli_rational(RBig::from_parts(1.into(), i.clone()))? {
                accept = false;
                break;
            }
            i += UBig::ONE;
        }

        if accept {
            return Ok(k);
        }
    }
}

#[proven]
/// Sample exactly from the Poisson distribution on $\mathbb{N}_0$ with mean `lambda`.
///
/// This is implemented by decomposing `lambda` into a sum of equal pieces in $[0,1)$,
/// sampling independent Poisson random variables for those pieces,
/// and summing the results.
///
/// # Proof Definition
/// For any finite rational `lambda` satisfying $\texttt{lambda} \ge 0$,
/// either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Poisson(\texttt{lambda})$.
pub(crate) fn sample_poisson(lambda: RBig) -> Fallible<UBig> {
    debug_assert!(lambda >= RBig::ZERO, "lambda must be non-negative");

    if lambda.is_zero() {
        return Ok(UBig::ZERO);
    }

    let pieces = (lambda.clone().floor() + 1u8).into_parts().1;
    let piece = lambda / RBig::from(pieces.clone());

    debug_assert!(piece >= RBig::ZERO);
    debug_assert!(piece < RBig::ONE);

    let mut remaining = pieces;
    let mut total = UBig::ZERO;

    while !remaining.is_zero() {
        total += sample_poisson_0_1(piece.clone())?;
        remaining -= UBig::ONE;
    }

    Ok(total)
}

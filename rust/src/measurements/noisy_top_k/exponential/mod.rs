use std::collections::BTreeSet;

use crate::{
    error::Fallible,
    traits::{
        CastInternalRational, FiniteBounds, ProductOrd,
        samplers::{sample_bernoulli_exp, sample_uniform_uint_below},
    },
};
use dashu::base::Sign;
use dashu::rational::RBig;
use opendp_derive::proven;

#[cfg(test)]
mod test;

#[proven]
/// # Proof Definition
/// Each value in $x$ must be finite.
///
/// Returns the index of the max element $z_i$,
/// where each $z_i \sim \mathrm{Exp}(\mathrm{shift}=y_i, \mathrm{scale}=\texttt{scale})$,
/// and each $y_i = -x_i$ if \texttt{negate}, else $y_i = x_i$,
/// $k$ times with removal.
pub(crate) fn exponential_top_k<TIA: Clone + CastInternalRational + ProductOrd + FiniteBounds>(
    x: &[TIA],
    scale: f64,
    k: usize,
    negate: bool,
) -> Fallible<Vec<usize>> {
    let sign = Sign::from(negate);
    let scale = scale.into_rational()?;

    let y = (x.into_iter().cloned())
        .map(|x_i| {
            x_i.total_clamp(TIA::MIN_FINITE, TIA::MAX_FINITE)?
                .into_rational()
                .map(|x_i| x_i * sign)
        })
        .collect::<Fallible<_>>()?;

    peel_permute_and_flip(y, scale, k)
}

#[proven]
/// # Proof Definition
/// Returns the index of the max element $z_i$,
/// where each $z_i \sim \mathrm{Exp}(\mathrm{shift}=x_i, \mathrm{scale}=\texttt{scale})$,
/// $k$ times with removal.
fn peel_permute_and_flip(mut x: Vec<RBig>, scale: RBig, k: usize) -> Fallible<Vec<usize>> {
    let mut natural_order = Vec::new();
    let mut sorted_order = BTreeSet::new();

    for _ in 0..k.min(x.len()) {
        let mut index = permute_and_flip(&x, &scale)?;
        x.remove(index);

        // map index on modified x back to original x (postprocessing)
        for &del in &sorted_order {
            if del <= index { index += 1 } else { break }
        }

        sorted_order.insert(index);
        natural_order.push(index);
    }
    Ok(natural_order)
}

#[proven]
/// # Proof Definition
/// Returns the index of the max element $z_i$,
/// where each $z_i \sim \mathrm{Exp}(\mathrm{shift}=x_i, \mathrm{scale}=\texttt{scale})$.
fn permute_and_flip(x: &[RBig], scale: &RBig) -> Fallible<usize> {
    let x_is_empty = || err!(FailedFunction, "x is empty");

    if scale.is_zero() {
        return (0..x.len()).max_by_key(|&i| &x[i]).ok_or_else(x_is_empty);
    }

    let x_max = x.iter().max().ok_or_else(x_is_empty)?;

    let mut permutation: Vec<usize> = (0..x.len()).collect();

    for left in 0..x.len() {
        let right = left + sample_uniform_uint_below(x.len() - left)?;
        permutation.swap(left, right); // fisher-yates shuffle up to left

        let candidate = permutation[left];
        if sample_bernoulli_exp((x_max - &x[candidate]) / scale)? {
            return Ok(candidate);
        }
    }
    unreachable!("at least one x[candidate] is equal to x_max")
}

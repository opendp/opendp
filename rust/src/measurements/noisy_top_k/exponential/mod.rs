use std::collections::BTreeSet;

use crate::{
    error::Fallible,
    traits::{
        CastInternalRational,
        samplers::{sample_bernoulli_exp, sample_uniform_uint_below},
    },
};
use dashu::base::Sign;
use dashu::rational::RBig;

#[cfg(test)]
mod test;

pub(crate) fn noisy_top_k_exponential<TIA: Clone + CastInternalRational>(
    x: &[TIA],
    scale: RBig,
    k: usize,
    negate: bool,
) -> Fallible<Vec<usize>> {
    let sign = Sign::from(negate);

    let x = (x.into_iter().cloned())
        .map(|x_i| x_i.into_rational().map(|x_i| x_i * sign))
        .collect::<Fallible<_>>()?;

    peel_permute_and_flip(x, scale, k)
}

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

use dashu::rational::RBig;

use crate::{
    error::Fallible,
    samplers::{sample_bernoulli_exp, sample_uniform_usize_below},
};

/// Create the initial candidate list `[0, 1, ..., len - 1]`.
fn make_candidates(len: usize) -> Vec<usize> {
    let mut candidates = Vec::with_capacity(len);
    let mut i = 0;
    while i < len {
        candidates.push(i);
        i += 1;
    }
    candidates
}

/// Map an index in a partially peeled score vector back into the original index space.
fn lift_deleted_index(sorted_order: &[usize], mut index: usize) -> usize {
    let mut cursor = 0;
    while cursor < sorted_order.len() {
        if sorted_order[cursor] <= index {
            index += 1;
            cursor += 1;
        } else {
            break;
        }
    }
    index
}

/// Insert an index into a sorted list of deleted/original positions.
fn insert_sorted(sorted_order: &mut Vec<usize>, index: usize) {
    let mut cursor = 0;
    while cursor < sorted_order.len() {
        if sorted_order[cursor] < index {
            cursor += 1;
        } else {
            break;
        }
    }

    sorted_order.push(index);

    let mut tail = sorted_order.len();
    while tail > cursor + 1 {
        sorted_order.swap(tail - 2, tail - 1);
        tail -= 1;
    }
}

/// # Proof Definition
/// If replacement is set, returns a sample from $\mathcal{M}_{EM}$ (as defined in MS2023 Definition 4),
/// otherwise returns a sample from $\mathcal{M}_{PF}$ (as defined in MS2023 Lemma 1),
/// where $\texttt{scale} = \frac{2 \cdot \Delta}{\epsilon}$.
pub fn permute_and_flip(x: &[RBig], scale: &RBig, replacement: bool) -> Fallible<usize> {
    if replacement {
        permute_and_flip_with_replacement(x, scale)
    } else {
        permute_and_flip_without_replacement(x, scale)
    }
}

/// Return the last index attaining the maximum score.
fn max_index(x: &[RBig]) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut i = 0;
    while i < x.len() {
        match best {
            Some(best_i) if x[best_i] > x[i] => {}
            _ => best = Some(i),
        }
        i += 1;
    }
    best
}

/// Permute-and-flip instantiated with the exponential-mechanism view.
pub fn permute_and_flip_with_replacement(x: &[RBig], scale: &RBig) -> Fallible<usize> {
    let x_max_index = match max_index(&x) {
        Some(index) => index,
        None => return fallible!(FailedFunction, "x is empty"),
    };

    if scale.is_zero() {
        return Ok(x_max_index);
    }

    let x_max = x[x_max_index].clone();

    loop {
        let candidate = {
            let limit = x.len();
            sample_uniform_usize_below(limit)
        }?;
        let x_candidate = x[candidate].clone();
        let gap = (x_max.clone() - x_candidate) / scale.clone();
        if sample_bernoulli_exp(gap)? {
            return Ok(candidate);
        }
    }
}

/// Permute-and-flip instantiated with the no-replacement view.
pub fn permute_and_flip_without_replacement(x: &[RBig], scale: &RBig) -> Fallible<usize> {
    let x_max_index = match max_index(&x) {
        Some(index) => index,
        None => return fallible!(FailedFunction, "x is empty"),
    };

    if scale.is_zero() {
        return Ok(x_max_index);
    }

    let x_max = x[x_max_index].clone();
    let mut remaining = make_candidates(x.len());

    while !remaining.is_empty() {
        let right = {
            let limit = remaining.len();
            sample_uniform_usize_below(limit)
        }?;
        let candidate = remaining.swap_remove(right);
        let x_candidate = x[candidate].clone();
        let gap = (x_max.clone() - x_candidate) / scale.clone();
        if sample_bernoulli_exp(gap)? {
            return Ok(candidate);
        }
    }

    unreachable!("at least one x[candidate] is equal to x_max")
}

/// Repeatedly apply permute-and-flip and translate peeled indices back to the original order.
pub fn peel_permute_and_flip(
    mut x: Vec<RBig>,
    scale: RBig,
    k: usize,
    replacement: bool,
) -> Fallible<Vec<usize>> {
    let mut natural_order = Vec::new();
    let mut sorted_order = Vec::new();

    for _ in 0..k.min(x.len()) {
        let mut index = permute_and_flip(&x, &scale, replacement)?;
        x.remove(index);

        index = lift_deleted_index(&sorted_order, index);

        insert_sorted(&mut sorted_order, index);
        natural_order.push(index);
    }

    Ok(natural_order)
}

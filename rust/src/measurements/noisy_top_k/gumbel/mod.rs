use dashu::float::FBig;
use opendp_derive::proven;

use crate::{
    error::Fallible,
    traits::{
        Number,
        samplers::{GumbelRV, PartialSample, Shuffle},
    },
};
use num::Zero;

use std::cmp::Ordering::{self, *};

#[cfg(test)]
mod test;

#[proven]
/// # Proof Definition
/// `scale` must be non-negative.
///
/// Returns the index of the top element $z_i$,
/// where each $z_i \sim \mathrm{Gumbel}(\mathrm{shift}=y_i, \mathrm{scale}=\texttt{scale})$,
/// and each $y_i = -x_i$ if \texttt{negate}, else $y_i = x_i$,
/// $k$ times with removal.
///
/// Each $x_i$ must be non-null.
///
/// The function will only return an error if entropy is exhausted.
/// If an error is returned, the error is data-dependent.
pub fn gumbel_top_k<T: Number>(x: &[T], scale: f64, k: usize, negate: bool) -> Fallible<Vec<usize>>
where
    FBig: TryFrom<T> + TryFrom<f64>,
{
    let scale = FBig::try_from(scale)?;
    if scale.is_zero() {
        // If scale is zero, we can just return the top k indices.
        // This is a special case that avoids the need to sample.
        let cmp = match negate {
            false => |l: &mut (usize, &T), r: &mut (usize, &T)| Ok(l.1 > r.1),
            true => |l: &mut (usize, &T), r: &mut (usize, &T)| Ok(l.1 < r.1),
        };

        let x_top = top_k(x.iter().enumerate(), k, cmp)?;
        return Ok(x_top.into_iter().map(|(i, _)| i).collect());
    }

    // When all scores are same, return a random index.
    // This is a workaround for slow performance of the samplers
    // when all scores are the same.
    if x.windows(2).all(|w| w[0] == w[1]) {
        let mut y = (0..x.len()).collect::<Vec<_>>();
        y.shuffle()?;
        return Ok(y[..k].to_vec());
    }

    let iter = (x.iter().enumerate())
        // Cast to FBig and discard failed casts.
        // Cast only fails on NaN scores, which are not in the input domain but could still be passed by the user.
        // If the user still passes NaN in the input data, discarding results in graceful failure.
        .filter_map(|(i, x_i)| {
            let x_i = x_i.total_clamp(T::MIN_FINITE, T::MAX_FINITE).ok()?;
            Some((i, FBig::try_from(x_i).ok()?))
        })
        .map(|(i, mut x_i)| {
            // Normalize sign.
            if negate {
                x_i = -x_i
            }
            // Initialize partial sample.
            let rv = GumbelRV {
                shift: x_i,
                scale: scale.clone(),
            };
            (i, PartialSample::new(rv))
        });

    // Reduce to the k pairs with largest samples.
    let k_pairs = top_k(iter, k, |l, r| l.1.greater_than(&mut r.1))?;

    // Discard samples, keep indices.
    Ok(k_pairs.into_iter().map(|(i, _)| i).collect())
}

#[proven]
/// Returns the top k elements from the iterator, using a heap to track the top k elements.
/// Optimized for the case where k is small compared to the number of elements in the iterator.
///
/// # Proof Definition
/// `iter` must be finite and `greater_than` must form a total order.
///
/// Returns the top k elements from the iterator in sorted order.
///
/// Only returns an error if `greater_than` returns an error.
pub(crate) fn top_k<T>(
    mut iter: impl Iterator<Item = T>,
    k: usize,
    greater_than: impl Fn(&mut T, &mut T) -> Fallible<bool>,
) -> Fallible<Vec<T>> {
    let heap = Vec::with_capacity(k);

    if k == 0 {
        return Ok(heap);
    }

    iter.try_fold(heap, |mut heap, mut value| {
        if let Some(last) = heap.get_mut(k - 1) {
            if greater_than(last, &mut value)? {
                return Ok(heap);
            }
            heap.pop();
        }

        let index = partition_point_mut(&mut heap, |x| greater_than(x, &mut value))?;
        heap.insert(index, value);

        Ok(heap)
    })
}

#[proven]
/// # Proof Definition
/// `x` must be partitioned by `pred`.
/// `pred` may mutate its argument, but not change its true value used for comparisons.
///
/// Returns the index of the partition point according to the given predicate (the index of the first element of the second partition),
/// or an error if the predicate fails.
pub fn partition_point_mut<T>(
    x: &mut Vec<T>,
    mut pred: impl FnMut(&mut T) -> Fallible<bool>,
) -> Fallible<usize> {
    binary_search_by_mut(x, |x_i| Ok(if pred(x_i)? { Less } else { Greater }))
}

#[proven]
/// # Proof Definition
/// `f` may mutate its argument, but not change its true value used for comparisons.
///
/// Returns the index $i$ of the first element in $x$ that is less than $f(x_i)$,
/// or an error if the comparator fails.
pub fn binary_search_by_mut<T>(
    x: &mut Vec<T>,
    mut f: impl FnMut(&mut T) -> Fallible<Ordering>,
) -> Fallible<usize> {
    let mut size = x.len();
    if size == 0 {
        return Ok(0);
    }
    let mut base = 0usize;

    while size > 1 {
        let half = size / 2;
        let mid = base + half;

        let cmp = f(&mut x[mid])?;
        base = if let Greater = cmp { base } else { mid };

        size -= half;
    }

    let cmp = f(&mut x[base])?;
    Ok(base + (cmp == Less) as usize)
}

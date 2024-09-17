use std::ops::Sub;

use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::Function,
    error::Fallible,
    traits::{Float, Number, RoundCast},
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib"), generics(TA(default = "float")))]
/// Postprocess a noisy array of float summary counts into a cumulative distribution.
///
/// # Generics
/// * `TA` - Atomic Type. One of `f32` or `f64`
pub fn make_cdf<TA: Float>() -> Fallible<Function<Vec<TA>, Vec<TA>>> {
    Ok(Function::new_fallible(|arg: &Vec<TA>| {
        let cumsum = arg
            .iter()
            .scan(TA::zero(), |acc, v| {
                *acc += *v;
                Some(*acc)
            })
            .collect::<Vec<TA>>();
        let sum = cumsum[cumsum.len() - 1];
        Ok(cumsum.into_iter().map(|v| v / sum).collect())
    }))
}

#[doc(hidden)]
pub enum Interpolation {
    Nearest,
    Linear,
}

#[bootstrap(
    features("contrib"),
    arguments(interpolation(c_type = "char *", rust_type = "String", default = "linear")),
    generics(F(default = "float"))
)]
/// Postprocess a noisy array of summary counts into quantiles.
///
/// # Arguments
/// * `bin_edges` - The edges that the input data was binned into before counting.
/// * `alphas` - Return all specified `alpha`-quantiles.
/// * `interpolation` - Must be one of `linear` or `nearest`
///
/// # Generics
/// * `TA` - Atomic Type of the bin edges and data.
/// * `F` - Float type of the alpha argument. One of `f32` or `f64`
pub fn make_quantiles_from_counts<TA, F>(
    bin_edges: Vec<TA>,
    alphas: Vec<F>,
    interpolation: Interpolation,
) -> Fallible<Function<Vec<TA>, Vec<TA>>>
where
    TA: Number + RoundCast<F>,
    F: Float + RoundCast<TA>,
{
    if bin_edges.len().is_zero() {
        return fallible!(MakeTransformation, "bin_edges.len() must be positive");
    }
    if bin_edges.windows(2).any(|w| w[0] >= w[1]) {
        return fallible!(MakeTransformation, "bin_edges must be increasing");
    }
    if alphas.windows(2).any(|w| w[0] >= w[1]) {
        return fallible!(MakeTransformation, "alphas must be increasing");
    }
    if let Some(lower) = alphas.first() {
        if lower.is_sign_negative() {
            return fallible!(
                MakeTransformation,
                "alphas must be greater than or equal to zero"
            );
        }
    }
    if let Some(upper) = alphas.last() {
        if upper > &F::one() {
            return fallible!(
                MakeTransformation,
                "alphas must be less than or equal to one"
            );
        }
    }

    Ok(Function::new_fallible(move |arg: &Vec<TA>| {
        // one fewer args than bin edges, or one greater args than bin edges are allowed
        if abs_diff(bin_edges.len(), arg.len()) != 1 {
            return fallible!(
                FailedFunction,
                "there must be one more bin edge than there are counts"
            );
        }
        if arg.is_empty() {
            return Ok(vec![bin_edges[0].clone(); alphas.len()]);
        }
        // if args includes extremal bins for (-inf, edge_0] and [edge_n, inf), discard them
        let arg = if bin_edges.len() + 1 == arg.len() {
            &arg[1..arg.len() - 1]
        } else {
            &arg[..]
        };
        // compute the cumulative sum of the input counts
        let cumsum = (arg.iter())
            .scan(TA::zero(), |acc, v| {
                *acc += v.clone();
                Some(acc.clone())
            })
            .map(F::round_cast)
            .collect::<Fallible<Vec<F>>>()?;

        // reuse the last element of the cumsum
        let sum = cumsum[cumsum.len() - 1];

        let cdf: Vec<F> = cumsum.into_iter().map(|v| v / sum).collect();

        // each index is the number of bins whose combined mass is less than the alpha_edge mass
        let mut indices = vec![0; alphas.len()];
        count_lt_recursive(indices.as_mut_slice(), alphas.as_slice(), cdf.as_slice(), 0);

        indices
            .into_iter()
            .zip(&alphas)
            .map(|(idx, &alpha)| {
                // Want to find the cumulative values to the left and right of edge
                // When no elements less than edge, consider cumulative value to be zero
                let left_cdf = if idx == 0 { F::zero() } else { cdf[idx - 1] };
                let right_cdf = cdf[idx];

                // println!("x's {:?}, {:?}", edge, (left.clone(), right.clone()));
                // println!("y's {:?}", (&bin_edges[idx], &bin_edges[idx + 1]));
                match interpolation {
                    Interpolation::Nearest => {
                        // if edge nearer to right than to left, then increment index
                        Ok(bin_edges[idx + (alpha - left_cdf > right_cdf - alpha) as usize])
                    }
                    Interpolation::Linear => {
                        let left_edge = F::round_cast(bin_edges[idx])?;
                        let right_edge = F::round_cast(bin_edges[idx + 1])?;

                        // find the interpolant between the bin edges.
                        // denominator is never zero because bin edges is strictly increasing
                        let t = (alpha - left_cdf) / (right_cdf - left_cdf);
                        let v = (F::one() - t) * left_edge + t * right_edge;
                        TA::round_cast(v)
                    }
                }
            })
            .collect()
    }))
}

fn abs_diff<T: PartialOrd + Sub<Output = T>>(a: T, b: T) -> T {
    if a < b {
        b - a
    } else {
        a - b
    }
}

/// Compute number of elements less than each edge
/// Formula is #(`x` <= e) for each e in `edges`.
///
/// # Arguments
/// * `counts` - location to write the result
/// * `edges` - edges to collect counts for. Must be sorted
/// * `x` - dataset to count against
/// * `x_start_idx` - value to add to the count. Useful for recursion on subslices
fn count_lt_recursive<TI: PartialOrd>(
    counts: &mut [usize],
    edges: &[TI],
    x: &[TI],
    x_start_idx: usize,
) {
    if edges.is_empty() {
        return;
    }
    if edges.len() == 1 {
        counts[0] = x_start_idx + count_lt(x, &edges[0]);
        return;
    }
    // use binary search to find |{i; x[i] < middle edge}|
    let mid_edge_idx = (edges.len() + 1) / 2;
    let mid_edge = &edges[mid_edge_idx];
    let mid_x_idx = count_lt(x, mid_edge);
    counts[mid_edge_idx] = x_start_idx + mid_x_idx;

    count_lt_recursive(
        &mut counts[..mid_edge_idx],
        &edges[..mid_edge_idx],
        &x[..mid_x_idx],
        x_start_idx,
    );

    count_lt_recursive(
        &mut counts[mid_edge_idx + 1..],
        &edges[mid_edge_idx + 1..],
        &x[mid_x_idx..],
        x_start_idx + mid_x_idx,
    );
}

/// Find the number of elements in `x` lt `target`.
/// Formula is #(`x` < `target`)
///
/// # Arguments
/// * `x` - dataset to count against
/// * `target` - value to compare against
fn count_lt<TI: PartialOrd>(x: &[TI], target: &TI) -> usize {
    if x.is_empty() {
        return 0;
    }
    let (mut lower, mut upper) = (0, x.len());

    while upper - lower > 1 {
        let middle = lower + (upper - lower) / 2;

        if &x[middle] < target {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    if &x[lower] < target {
        upper
    } else {
        lower
    }
}

#[cfg(test)]
mod test;

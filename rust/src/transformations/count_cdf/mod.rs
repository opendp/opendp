use std::{
    iter::Sum,
    ops::{AddAssign, Div, Mul, Sub},
};

use num::{Float, Zero};

use crate::{
    core::{Transformation, Function},
    metrics::AgnosticMetric,
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    traits::{CheckNull, RoundCast},
};

use super::postprocess::make_postprocess;

/// Constructs a [`Transformation`] that maps a float vector of counts into a cumulative distribution
pub fn make_cdf<T>() -> Fallible<
    Transformation<
        VectorDomain<AllDomain<T>>,
        VectorDomain<AllDomain<T>>,
        AgnosticMetric,
        AgnosticMetric,
    >,
>
where
    T: CheckNull + Zero + AddAssign + Clone + Float,
{
    make_postprocess(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(|arg: &Vec<T>| {
            let cumsum = arg
                .iter()
                .scan(T::zero(), |acc, v| {
                    *acc += v.clone();
                    Some(*acc)
                })
                .collect::<Vec<T>>();
            let sum = cumsum[cumsum.len() - 1].clone();
            Ok(cumsum.into_iter().map(|v| v / sum).collect())
        }),
    )
}

pub enum Interpolate {
    Nearest,
    Linear,
}

/// Constructs a [`Transformation`] that retrieves nearest bin edge to quantile
pub fn make_quantiles_from_counts<T, F>(
    bin_edges: Vec<T>,
    alphas: Vec<F>,
    interpolate: Interpolate,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<T>>,
        VectorDomain<AllDomain<T>>,
        AgnosticMetric,
        AgnosticMetric,
    >,
>
where
    T: 'static
        + CheckNull
        + for<'a> Sum<&'a T>
        + RoundCast<F>
        + PartialOrd
        + Clone
        + Zero
        + AddAssign
        + Sub<Output = T>
        + Div<Output = T>
        + Mul<Output = T>,
    F: 'static + Float + RoundCast<T>,
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
    if let Some(upper) = alphas.first() {
        if upper > &F::one() {
            return fallible!(
                MakeTransformation,
                "alphas must be less than or equal to one"
            );
        }
    }
    make_postprocess(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<T>| {
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
            let cumsum = arg
                .iter()
                .scan(T::zero(), |acc, v| {
                    *acc += v.clone();
                    Some(acc.clone())
                })
                .collect::<Vec<T>>();

            // reuse the last element of the cumsum
            let sum = F::round_cast(cumsum[cumsum.len() - 1].clone())?;

            // scale alphas up into T space and convert to T
            let alpha_edges = alphas
                .iter()
                .cloned()
                .map(|a| T::round_cast(a * sum))
                .collect::<Fallible<Vec<T>>>()?;

            // each index is the number of bins whose combined mass is less than the alpha_edge mass
            let mut indices = vec![0; alphas.len()];
            count_lt_recursive(
                indices.as_mut_slice(),
                alpha_edges.as_slice(),
                cumsum.as_slice(),
                0,
            );

            Ok(indices
                .into_iter()
                .zip(alpha_edges)
                .map(|(idx, edge)| {
                    // Want to find the cumulative values to the left and right of edge
                    // When no elements less than edge, consider cumulative value to be zero
                    let left = if idx == 0 {
                        T::zero()
                    } else {
                        cumsum[idx - 1].clone()
                    };
                    let right = cumsum[idx].clone();

                    // println!("x's {:?}, {:?}", edge, (left.clone(), right.clone()));
                    // println!("y's {:?}", (&bin_edges[idx], &bin_edges[idx + 1]));
                    match interpolate {
                        Interpolate::Nearest => {
                            // if edge nearer to right than to left, then increment index
                            bin_edges[idx + (edge.clone() - left > right - edge) as usize].clone()
                        }
                        Interpolate::Linear => {
                            // find the interpolant between the bin edges.
                            // denominator is never zero because bin edges is strictly increasing
                            let slope = (bin_edges[idx + 1].clone() - bin_edges[idx].clone())
                                / (right - left.clone());
                            bin_edges[idx].clone() + (edge.clone() - left) * slope
                        }
                    }
                })
                .collect())
        }),
    )
}

fn abs_diff<T: PartialOrd + Sub<Output=T>>(a: T, b: T) -> T {
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
mod test_cdf {
    use super::*;
    #[test]
    fn test_cdf() -> Fallible<()> {
        let cdf_trans = make_cdf::<f64>()?;
        let cdf = cdf_trans.invoke(&vec![2.23, 3.4, 5., 2.7])?;
        assert_eq!(
            cdf,
            vec![
                0.16729182295573897,
                0.42235558889722435,
                0.7974493623405852,
                1.0
            ]
        );
        Ok(())
    }

    #[test]
    fn test_quantile() -> Fallible<()> {
        let edges = vec![0, 25, 50, 75, 100];
        let alphas = vec![0., 0.1, 0.24, 0.51, 0.74, 0.75, 0.76, 0.99, 1.];
        let quantile_trans =
            make_quantiles_from_counts(edges.clone(), alphas.clone(), Interpolate::Nearest)?;
        let quantiles = quantile_trans.invoke(&vec![100, 100, 100, 100])?;
        println!("{:?}", quantiles);
        assert_eq!(quantiles, vec![0, 0, 25, 50, 75, 75, 75, 100, 100]);

        let quantile_trans = make_quantiles_from_counts(edges, alphas, Interpolate::Linear)?;
        let quantiles = quantile_trans.invoke(&vec![100, 100, 100, 100])?;
        println!("{:?}", quantiles);
        // assert_eq!(quantiles, vec![0, 0, 0, 50, 75, 75, 75, 100, 100]);
        Ok(())
    }

    #[test]
    fn test_quantile_with_edge_buckets() -> Fallible<()> {
        let edges = vec![0, 25, 50, 75, 100];
        let alphas = vec![0., 0.1, 0.24, 0.51, 0.74, 0.75, 0.76, 0.99, 1.];
        let quantile_trans = make_quantiles_from_counts(edges, alphas, Interpolate::Nearest)?;
        let quantiles = quantile_trans.invoke(&vec![210, 100, 100, 100, 100, 234])?;
        println!("{:?}", quantiles);
        assert_eq!(quantiles, vec![0, 0, 25, 50, 75, 75, 75, 100, 100]);
        Ok(())
    }

    #[test]
    fn test_quantile_float() -> Fallible<()> {
        let edges = vec![0., 10., 20., 30.];
        let alphas = vec![0.2, 0.4, 0.7];
        let quantile_trans =
            make_quantiles_from_counts(edges.clone(), alphas.clone(), Interpolate::Nearest)?;
        let quantiles = quantile_trans.invoke(&vec![2.23, 3.4, 5.])?;
        assert_eq!(quantiles, vec![10., 20., 20.]);

        let quantile_trans = make_quantiles_from_counts(edges, alphas, Interpolate::Linear)?;
        let quantiles = quantile_trans.invoke(&vec![2.23, 3.4, 5.])?;
        assert_eq!(
            quantiles,
            vec![9.533632286995514, 15.947058823529412, 23.622]
        );
        println!("{:?}", quantiles);
        Ok(())
    }

    #[test]
    fn test_quantile_int() -> Fallible<()> {
        let edges = vec![0, 10, 50, 100];
        let alphas = vec![0.2, 0.4, 0.7];
        let quantile_trans = make_quantiles_from_counts(edges, alphas, Interpolate::Nearest)?;
        let quantiles = quantile_trans.invoke(&vec![2, 3, 5])?;
        assert_eq!(quantiles, vec![10, 50, 50]);
        Ok(())
    }
}

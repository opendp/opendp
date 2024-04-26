use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::LpDistance,
    traits::{InfCast, Integer, Number},
};

#[cfg(feature = "ffi")]
mod ffi;

mod consistency_postprocessor;
pub use consistency_postprocessor::*;
use opendp_derive::bootstrap;

#[bootstrap(features("contrib"), generics(TA(suppress), M(suppress)))]
/// Expand a vector of counts into a b-ary tree of counts,
/// where each branch is the sum of its `b` immediate children.
///
/// # Arguments
/// * `leaf_count` - The number of leaf nodes in the b-ary tree.
/// * `branching_factor` - The number of children on each branch of the resulting tree. Larger branching factors result in shallower trees.
///
/// # Generics
/// * `M` - Metric. Must be `L1Distance<Q>` or `L2Distance<Q>`
/// * `TA` - Atomic Type of the input data.
pub fn make_b_ary_tree<M, TA>(
    input_domain: VectorDomain<AtomDomain<TA>>,
    input_metric: M,
    leaf_count: u32,
    branching_factor: u32,
) -> Fallible<Transformation<VectorDomain<AtomDomain<TA>>, VectorDomain<AtomDomain<TA>>, M, M>>
where
    TA: Integer,
    M: BAryTreeMetric,
    M::Distance: Number,
    (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
{
    if leaf_count == 0 {
        return fallible!(MakeTransformation, "leaf_count must be at least 1");
    }
    if branching_factor < 2 {
        return fallible!(MakeTransformation, "branching_factor must be at least two");
    }
    let leaf_count = leaf_count as usize;
    let branching_factor = branching_factor as usize;

    let num_layers = num_layers_from_num_leaves(leaf_count, branching_factor);
    // specifically, the number of leaves in a full tree
    let num_leaves = branching_factor.pow(num_layers as u32 - 1);
    // leaf_count is the number of leaves in the final layer of a complete tree

    Transformation::new(
        input_domain.clone(),
        input_domain.without_size(),
        Function::new(move |arg: &Vec<TA>| {
            // if arg.len() != num_bins, then user has a bug that will affect utility, but cannot alert
            //    if less data is passed than num_bins, then pad with extra zeros
            //    if more data is passed then num_bins, then ignore
            let pad_length = num_leaves - leaf_count.min(arg.len());

            let mut layers = vec![arg
                .iter()
                .take(leaf_count)
                .cloned()
                .chain((0..pad_length).map(|_| TA::zero()))
                .collect::<Vec<TA>>()];

            (0..num_layers - 1).for_each(|i| {
                layers.push(
                    layers[i]
                        .chunks(branching_factor)
                        .map(|chunk| chunk.iter().sum::<TA>())
                        .collect(),
                );
            });
            let tree_length = num_nodes_from_num_layers(num_layers, branching_factor) - pad_length;
            layers
                .into_iter()
                .rev()
                .flatten()
                .take(tree_length)
                .collect()
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(M::Distance::inf_cast(num_layers)?),
    )
}

// find i such that b^i >= x
// returns 0 when x == 0
fn log_b_ceil(x: usize, b: usize) -> usize {
    let mut checker = 1;
    let mut pow = 0;
    loop {
        if checker >= x {
            return pow;
        }
        checker *= b;
        pow += 1;
    }
}

fn num_layers_from_num_leaves(num_leaves: usize, b: usize) -> usize {
    log_b_ceil(num_leaves, b) + 1
}

fn num_layers_from_num_nodes(num_nodes: usize, b: usize) -> usize {
    log_b_ceil((b - 1) * num_nodes + 1, b)
}

fn num_nodes_from_num_layers(num_layers: usize, b: usize) -> usize {
    (b.pow(num_layers as u32) - 1) / (b - 1)
}

pub trait BAryTreeMetric: Metric {}
impl<const P: usize, T> BAryTreeMetric for LpDistance<P, T> {}

#[bootstrap(features("contrib"))]
/// Returns an approximation to the ideal `branching_factor` for a dataset of a given size,
/// that minimizes error in cdf and quantile estimates based on b-ary trees.
///
/// # Citations
/// * [QYL13 Understanding Hierarchical Methods for Differentially Private Histograms](http://www.vldb.org/pvldb/vol6/p1954-qardaji.pdf)
///
/// # Arguments
/// * `size_guess` - A guess at the size of your dataset.
pub fn choose_branching_factor(size_guess: u32) -> u32 {
    // Formula (3) estimates variance
    fn v_star_avg(n: f64, b: f64) -> f64 {
        let h = n.log(b);
        return (b - 1.) * h.powi(3) - 2. * (b + 1.) * h.powi(2) / 3.;
    }

    // find the b with minimum average variance
    (2..size_guess + 1)
        .map(|b| (b, v_star_avg(size_guess as f64, b as f64)))
        .min_by(|(_, a_s), (_, b_s)| a_s.total_cmp(b_s))
        .map(|p| p.0)
        .unwrap_or(size_guess)
}

#[cfg(all(test, feature = "partials"))]
mod test;

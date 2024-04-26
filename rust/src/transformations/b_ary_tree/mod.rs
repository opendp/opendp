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
pub mod test {
    use crate::{measurements::then_laplace, metrics::L1Distance};

    use super::*;

    fn log_b_ceil_float(x: usize, b: usize) -> usize {
        // naive implementation susceptible to float approximation errors
        // for example, log_5(125) == 3, but evaluates to 3.0000000000000004 -> 4
        (x as f64).log(b as f64).ceil() as usize
    }

    #[test]
    fn test_log_b_ceil() {
        // tests pass for small values,
        //    but large values fail because log_b_ceil_float is unstable
        (1..100).for_each(move |x| {
            (2..100).for_each(|b| {
                assert_eq!(
                    log_b_ceil(x, b),
                    log_b_ceil_float(x, b),
                    "log_{:?}({:?})",
                    b,
                    x
                )
            })
        });
    }

    #[test]
    fn test_num_layers_from_num_leaves() {
        assert_eq!(num_layers_from_num_leaves(10, 2), 5);
        assert_eq!(num_layers_from_num_leaves(16, 2), 5);
        assert_eq!(num_layers_from_num_leaves(8, 2), 4);
        assert_eq!(num_layers_from_num_leaves(6, 2), 4);
        assert_eq!(num_layers_from_num_leaves(3, 2), 3);
    }

    #[test]
    fn test_layers_from_num_nodes() {
        assert_eq!(num_layers_from_num_nodes(1, 2), 1);
        assert_eq!(num_layers_from_num_nodes(2, 2), 2);
        assert_eq!(num_layers_from_num_nodes(3, 2), 2);
        assert_eq!(num_layers_from_num_nodes(7, 2), 3);
        assert_eq!(num_layers_from_num_nodes(8, 2), 4);

        assert_eq!(num_layers_from_num_nodes(2, 3), 2);
        assert_eq!(num_layers_from_num_nodes(4, 3), 2);
        assert_eq!(num_layers_from_num_nodes(5, 4), 2);
        assert_eq!(num_layers_from_num_nodes(13, 3), 3);
        assert_eq!(num_layers_from_num_nodes(14, 3), 4);
    }

    #[test]
    fn test_num_nodes_from_num_layers() {
        assert_eq!(num_nodes_from_num_layers(1, 2), 1);
        assert_eq!(num_nodes_from_num_layers(2, 2), 3);
        assert_eq!(num_nodes_from_num_layers(3, 2), 7);
        assert_eq!(num_nodes_from_num_layers(4, 2), 15);

        assert_eq!(num_nodes_from_num_layers(1, 3), 1);
        assert_eq!(num_nodes_from_num_layers(2, 3), 4);
        assert_eq!(num_nodes_from_num_layers(3, 3), 13);
        assert_eq!(num_nodes_from_num_layers(4, 3), 40);
    }

    #[test]
    fn test_make_b_ary_tree() -> Fallible<()> {
        let trans = make_b_ary_tree::<L1Distance<i32>, i32>(
            Default::default(),
            L1Distance::default(),
            10,
            2,
        )?;
        let actual = trans.invoke(&vec![1; 10])?;
        let expect = vec![
            vec![10],
            vec![8, 2],
            vec![4, 4, 2, 0],
            vec![2, 2, 2, 2, 2, 0, 0, 0],
            vec![1; 10],
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<i32>>();
        assert_eq!(expect, actual);

        Ok(())
    }

    #[test]
    fn test_noise_b_ary_tree() -> Fallible<()> {
        let meas = (make_b_ary_tree::<_, i32>(Default::default(), L1Distance::default(), 10, 2)?
            >> then_laplace(1., None))?;
        println!("noised {:?}", meas.invoke(&vec![1; 10])?);

        Ok(())
    }

    #[test]
    fn test_identity() -> Fallible<()> {
        let b = 2;
        let trans = make_b_ary_tree::<_, i32>(Default::default(), L1Distance::default(), 10, b)?;
        let meas = (trans.clone() >> then_laplace(0., None))?;
        let post = make_consistent_b_ary_tree::<i32, f64>(b)?;

        let noisy_tree = meas.invoke(&vec![1; 10])?;
        // casting should not lose data, as noise was integral
        let consi_leaves = post
            .eval(&noisy_tree)?
            .into_iter()
            .map(|v| v as i32)
            .collect();
        let consi_tree =
            make_b_ary_tree::<_, i32>(Default::default(), L1Distance::<f64>::default(), 10, b)?
                .invoke(&consi_leaves)?;

        println!("noisy      leaves {:?}", noisy_tree[15..].to_vec());
        println!("consistent leaves {:?}", consi_leaves);

        println!("noisy      tree {:?}", noisy_tree);
        println!("consistent tree {:?}", consi_tree);

        assert_eq!(noisy_tree, consi_tree);
        Ok(())
    }
}

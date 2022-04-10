use std::{
    iter::Sum,
    ops::{AddAssign, MulAssign},
};

use num::{Float, Zero};

use crate::{
    core::{Function, Metric, StabilityMap, Transformation},
    metrics::{AgnosticMetric, LpDistance},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    traits::{CheckNull, DistanceConstant, InfCast, RoundCast},
};

use super::postprocess::make_postprocess;

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

// TODO: parameterize by type of container size
pub fn make_b_ary_tree<T, M>(
    num_bins: usize,
    b: usize,
) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, M, M>>
where
    T: CheckNull + Zero + Clone + for<'a> Sum<&'a T> + std::fmt::Debug,
    M: BAryTreeMetric,
    M::Distance: DistanceConstant<M::Distance> + InfCast<usize>,
{
    if num_bins == 0 {
        return fallible!(MakeTransformation, "num_leaves must be at least 1");
    }
    if b < 2 {
        return fallible!(MakeTransformation, "branching factor must be at least two");
    }

    let num_layers = num_layers_from_num_leaves(num_bins, b);
    let num_leaves = b.pow(num_layers as u32 - 1);

    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<T>| {
            // if arg.len() != num_bins, then user has a bug that will affect utility, but cannot alert
            //    if less data is passed than num_bins, then pad with extra zeros
            //    if more data is passed then num_bins, then ignore
            let pad_length = num_leaves - num_bins.min(arg.len());

            let mut layers = vec![arg
                .iter()
                .take(num_bins)
                .cloned()
                .chain((0..pad_length).map(|_| T::zero()))
                .collect::<Vec<T>>()];

            (0..num_layers - 1).for_each(|i| {
                layers.push(
                    layers[i]
                        .chunks(b)
                        .map(|chunk| chunk.iter().sum::<T>())
                        .collect(),
                );
            });
            let tree_length = num_nodes_from_num_layers(num_layers, b) - pad_length;
            layers
                .into_iter()
                .rev()
                .flatten()
                .take(tree_length)
                .collect()
        }),
        M::default(),
        M::default(),
        StabilityMap::new_from_constant(M::Distance::inf_cast(num_layers)?),
    ))
}

pub fn make_b_ary_tree_consistent<TI, TO>(
    b: usize,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TI>>,
        VectorDomain<AllDomain<TO>>,
        AgnosticMetric,
        AgnosticMetric,
    >,
>
where
    TI: CheckNull + Clone,
    TO: CheckNull + Float + RoundCast<TI> + for<'a> Sum<&'a TO> + MulAssign + AddAssign,
{
    make_postprocess(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| {
            let layers = num_layers_from_num_nodes(arg.len(), b);

            let mut vars = vec![TO::one(); num_nodes_from_num_layers(layers, b)];
            let zero_leaves = vars.len() - arg.len();
            let mut tree: Vec<TO> = arg
                .iter()
                .cloned()
                .map(|v| TO::round_cast(v))
                .chain((0..zero_leaves).map(|_| Ok(TO::zero())))
                .collect::<Fallible<_>>()?;

            // zero out all zero variance zero nodes on the tree
            (0..layers).for_each(|l| {
                // number of zeros in layer l
                let l_zeros = zero_leaves / b.pow((layers - l - 1) as u32);
                let l_end = num_nodes_from_num_layers(l + 1, b);
                vars[l_end - l_zeros..l_end].fill(TO::zero());
                tree[l_end - l_zeros..l_end].fill(TO::zero());
            });

            // bottom-up scan to compute z
            (0..layers - 1).rev().for_each(|l| {
                let l_start = num_nodes_from_num_layers(l, b);
                (0..b.pow(l as u32)).for_each(|offset| {
                    let i = l_start + offset;
                    if vars[i].is_zero() {
                        return;
                    }

                    let child_slice = i * b + 1..i * b + 1 + b;

                    let child_var: TO = vars[child_slice.clone()].iter().sum();
                    let child_val: TO = tree[child_slice].iter().sum();

                    // weight to give to self (part 1)
                    let mut alpha = vars[i].recip();

                    // update total variance of node to reflect postprocessing
                    vars[i] = (vars[i].recip() + child_var.recip()).recip();

                    // weight to give to self (part 2)
                    // weight of self is a proportion of total inverse variance (total var / prior var)
                    alpha *= vars[i];

                    // postprocess by weighted inverse variance
                    tree[i] = alpha * tree[i] + (TO::one() - alpha) * child_val;
                });
            });

            // top down scan to compute h
            let mut h_b = tree.clone();
            (0..layers - 1).for_each(|l| {
                let l_start = num_nodes_from_num_layers(l, b);

                (0..b.pow(l as u32)).for_each(|offset| {
                    let i = l_start + offset;
                    let child_slice = i * b + 1..i * b + 1 + b;
                    let child_vars = vars[child_slice.clone()].to_vec();

                    // children need to be adjusted by this amount to be consistent with parent
                    let correction = h_b[i] - tree[child_slice.clone()].iter().sum();
                    if correction.is_zero() {
                        return;
                    }

                    // apportion the correction among children relative to their variance
                    let sum_var = child_vars.iter().sum();
                    h_b[child_slice]
                        .iter_mut()
                        .zip(child_vars)
                        .for_each(|(v, child_var)| *v += correction * child_var / sum_var);
                });
            });

            // entire tree is consistent, so only the nonzero leaves in bottom layer are needed
            let leaf_start = num_nodes_from_num_layers(layers - 1, b);
            let leaf_end = num_nodes_from_num_layers(layers, b) - zero_leaves;
            Ok(h_b[leaf_start..leaf_end].to_vec())
        }))
}

#[cfg(test)]
pub mod test_b_trees {
    use crate::{metrics::L1Distance, meas::make_base_geometric};

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
        let trans = make_b_ary_tree::<i32, L1Distance<i32>>(10, 2)?;
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
        let meas =
            (make_b_ary_tree::<i32, L1Distance<i32>>(10, 2)? >> make_base_geometric(1., None)?)?;
        println!("noised {:?}", meas.invoke(&vec![1; 10])?);

        Ok(())
    }

    #[test]
    fn test_identity() -> Fallible<()> {
        let b = 2;
        let trans = make_b_ary_tree::<i32, L1Distance<i32>>(10, b)?;
        let meas = (trans.clone() >> make_base_geometric(0., None)?)?;
        let post = make_b_ary_tree_consistent::<i32, f64>(b)?;

        let noisy_tree = meas.invoke(&vec![1; 10])?;
        let consi_leaves = post.invoke(&noisy_tree)?;
        let consi_tree = make_b_ary_tree::<f64, L1Distance<f64>>(10, b)?.invoke(&consi_leaves)?;

        println!("noisy      leaves {:?}", noisy_tree[15..].to_vec());
        println!("consistent leaves {:?}", consi_leaves);

        println!("noisy      tree {:?}", noisy_tree);
        println!("consistent tree {:?}", consi_tree);

        let noisy_tree_f = noisy_tree.into_iter().map(f64::from).collect::<Vec<f64>>();
        assert_eq!(noisy_tree_f, consi_tree);
        Ok(())
    }
}

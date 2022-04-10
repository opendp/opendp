use std::{iter::Sum, ops::{MulAssign, AddAssign}};

use num::{Zero, Float};

use crate::{
    core::{Function, Metric, StabilityMap, Transformation},
    metrics::{AgnosticMetric, LpDistance},
    domains::{AllDomain, VectorDomain},
    error::{ExplainUnwrap, Fallible},
    traits::{CheckNull, DistanceConstant, InfCast, RoundCast},
};

// find i such that b^i >= x
fn log_b_ceil(x: usize, b: usize) -> usize {
    if x == 1 {
        return 0;
    }

    let mut checker = b;
    (0usize..)
        .find(|_| {
            checker *= b;
            checker >= x
        })
        .unwrap_assert("loop runs until value is found")
}

fn height_from_leaf_count(num_leaves: usize, b: usize) -> usize {
    log_b_ceil(num_leaves, b) + 1
}

fn layers_from_num_nodes(num_nodes: usize, b: usize) -> usize {
    log_b_ceil((b - 1) * num_nodes + 1, b)
}

// height = num_layers - 1
fn num_nodes_from_height(height: usize, b: usize) -> Fallible<usize> {
    // num_nodes = b^height / (b - 1)
    b.checked_pow(height as u32)
        .map(|n| n / (b - 1))
        .ok_or_else(|| err!(FailedFunction, "tree is too large to be indexed (highly unlikely)"))
}

pub trait BAryTreeMetric: Metric {}
impl<const P: usize, T> BAryTreeMetric for LpDistance<P, T> {}

// TODO: parameterize by type of container size
pub fn make_b_ary_tree<T, M>(
    num_bins: usize,
    b: usize,
) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, M, M>>
where
    T: CheckNull + Zero + Clone + for<'a> Sum<&'a T>,
    M: BAryTreeMetric,
    M::Distance: DistanceConstant<M::Distance> + InfCast<usize>,
{
    if num_bins == 0 {
        return fallible!(MakeTransformation, "num_leaves must be at least 1");
    }
    if b < 2 {
        return fallible!(MakeTransformation, "branching factor must be at least two");
    }

    let new_height = height_from_leaf_count(num_bins, b);
    let num_leaves = b.pow(new_height as u32);

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

            (0..new_height + 1).for_each(|i| {
                layers.push(
                    layers[i]
                        .chunks(b)
                        .map(|chunk| chunk.iter().sum::<T>())
                        .collect(),
                )
            });

            layers.into_iter().rev().flatten().collect()
        }),
        M::default(),
        M::default(),
        StabilityMap::new_from_constant(M::Distance::inf_cast(new_height + 1)?),
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
    TO: CheckNull + Float + RoundCast<TI> + for <'a> Sum<&'a TO> + MulAssign + AddAssign,
{
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TI>| {
            let layers = layers_from_num_nodes(arg.len(), b);

            let mut vars = vec![TO::one(); num_nodes_from_height(layers - 1, b)?];
            let zero_leaves = vars.len() - arg.len();
            let mut tree: Vec<TO> = arg
                .iter()
                .cloned()
                .map(|v| TO::round_cast(v))
                .chain((0..zero_leaves).map(|_| Ok(TO::zero())))
                .collect::<Fallible<_>>()?;

            // zero out all zero variance zero nodes on the tree
            (0..layers).try_for_each(|l| {
                // number of zeros in layer l
                let l_zeros = zero_leaves / b.pow((layers - l - 1) as u32);
                let l_end = num_nodes_from_height(l, b)?;
                vars[l_end - l_zeros..l_end].fill(TO::zero());
                tree[l_end - l_zeros..l_end].fill(TO::zero());
                Fallible::Ok(())
            })?;

            // bottom-up scan to compute z
            (0..layers - 1)
            .rev()
            .try_for_each(|l| {
                let l_start = num_nodes_from_height(l - 1, b)?;
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
                Fallible::Ok(())
            })?;

            // top down scan to compute h
            let mut h_b = tree.clone();
            (0..layers - 1).try_for_each(|l| {
                let l_start = num_nodes_from_height(l - 1, b)?;

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
                Fallible::Ok(())
            })?;

            // entire tree is consistent, so only the nonzero leaves in bottom layer are needed
            let leaf_start = num_nodes_from_height(layers - 2, b)?;
            let leaf_end = num_nodes_from_height(layers - 1, b)? - zero_leaves;
            Ok(h_b[leaf_start..leaf_end].to_vec())
        }),
        AgnosticMetric::default(),
        AgnosticMetric::default(),
        StabilityRelation::new_all(
            |_d_in: &(), _d_out: &()| Ok(true),
            None::<fn(&_) -> _>,
            None::<fn(&_) -> _>,
        ),
    ))
}

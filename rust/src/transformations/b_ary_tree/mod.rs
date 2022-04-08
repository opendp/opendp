use std::iter::Sum;

use num::Zero;

use crate::{
    core::{Function, Metric, StabilityMap, Transformation},
    metrics::{AgnosticMetric, LpDistance},
    domains::{AllDomain, VectorDomain},
    error::{ExplainUnwrap, Fallible},
    traits::{CheckNull, DistanceConstant, InfCast},
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

// height = num_layers - 1
// fn num_nodes_from_height(height: usize, b: usize) -> Option<usize> {
//     // num_nodes = b^height / (b - 1)
//     b.checked_pow(height as u32).map(|n| n / (b - 1))
// }

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

pub fn make_b_ary_tree_consistent<T: CheckNull>(
    _b: usize,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<T>>,
        VectorDomain<AllDomain<T>>,
        AgnosticMetric,
        AgnosticMetric,
    >,
> {
    unimplemented!()
}

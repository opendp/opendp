use opendp_derive::bootstrap;

use crate::{
    core::Function,
    error::Fallible,
    traits::{CheckAtom, Float, RoundCast},
};

use super::{num_layers_from_num_nodes, num_nodes_from_num_layers};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    generics(TIA(default = "int"), TOA(default = "float"))
)]
/// Postprocessor that makes a noisy b-ary tree internally consistent, and returns the leaf layer.
///
/// The input argument of the function is a balanced `b`-ary tree implicitly stored in breadth-first order
/// Tree is assumed to be complete, as in, all leaves on the last layer are on the left.
/// Non-existent leaves are assumed to be zero.
///
/// The output remains consistent even when leaf nodes are missing.
/// This is due to an adjustment to the original algorithm to apportion corrections to children relative to their variance.
///
/// # Citations
/// * [HRMS09 Boosting the Accuracy of Differentially Private Histograms Through Consistency, section 4.1](https://arxiv.org/pdf/0904.0942.pdf)
///
/// # Arguments
/// * `branching_factor` - the maximum number of children
///
/// # Generics
/// * `TIA` - Atomic type of the input data. Should be an integer type.
/// * `TOA` - Atomic type of the output data. Should be a float type.
pub fn make_consistent_b_ary_tree<TIA, TOA>(
    branching_factor: u32,
) -> Fallible<Function<Vec<TIA>, Vec<TOA>>>
where
    TIA: CheckAtom + Clone,
    TOA: Float + RoundCast<TIA>,
{
    let branching_factor = branching_factor as usize;
    Ok(Function::new_fallible(move |arg: &Vec<TIA>| {
        let layers = num_layers_from_num_nodes(arg.len(), branching_factor);

        let mut vars = vec![TOA::one(); num_nodes_from_num_layers(layers, branching_factor)];
        let zero_leaves = vars.len() - arg.len();
        let mut tree: Vec<TOA> = arg
            .iter()
            .cloned()
            .map(|v| TOA::round_cast(v))
            .chain((0..zero_leaves).map(|_| Ok(TOA::zero())))
            .collect::<Fallible<_>>()?;

        // zero out all zero variance zero nodes on the tree
        (0..layers).for_each(|l| {
            // number of zeros in layer l
            let l_zeros = zero_leaves / branching_factor.pow((layers - l - 1) as u32);
            let l_end = num_nodes_from_num_layers(l + 1, branching_factor);
            vars[l_end - l_zeros..l_end].fill(TOA::zero());
            tree[l_end - l_zeros..l_end].fill(TOA::zero());
        });

        // bottom-up scan to compute z
        (0..layers - 1).rev().for_each(|l| {
            let l_start = num_nodes_from_num_layers(l, branching_factor);
            (0..branching_factor.pow(l as u32)).for_each(|offset| {
                let i = l_start + offset;
                if vars[i].is_zero() {
                    return;
                }

                let child_slice =
                    i * branching_factor + 1..i * branching_factor + 1 + branching_factor;

                let child_var: TOA = vars[child_slice.clone()].iter().sum();
                let child_val: TOA = tree[child_slice].iter().sum();

                // weight to give to self (part 1)
                let mut alpha = vars[i].recip();

                // update total variance of node to reflect postprocessing
                vars[i] = (vars[i].recip() + child_var.recip()).recip();

                // weight to give to self (part 2)
                // weight of self is a proportion of total inverse variance (total var / prior var)
                alpha *= vars[i];

                // postprocess by weighted inverse variance
                tree[i] = alpha * tree[i] + (TOA::one() - alpha) * child_val;
            });
        });

        // top down scan to compute h
        let mut h_b = tree.clone();
        (0..layers - 1).for_each(|l| {
            let l_start = num_nodes_from_num_layers(l, branching_factor);

            (0..branching_factor.pow(l as u32)).for_each(|offset| {
                let i = l_start + offset;
                let child_slice =
                    i * branching_factor + 1..i * branching_factor + 1 + branching_factor;
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
        let leaf_start = num_nodes_from_num_layers(layers - 1, branching_factor);
        let leaf_end = num_nodes_from_num_layers(layers, branching_factor) - zero_leaves;
        Ok(h_b[leaf_start..leaf_end].to_vec())
    }))
}

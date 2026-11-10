#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::{InsertDeleteDistance, IntDistance, SymmetricDistance};
use crate::traits::CheckAtom;
use crate::traits::samplers::Shuffle;
use std::cmp::Ordering;

#[doc(hidden)]
pub trait IsMetricOrdered: Metric {
    const ORDERED: bool;
}
impl IsMetricOrdered for SymmetricDistance {
    const ORDERED: bool = false;
}
impl IsMetricOrdered for InsertDeleteDistance {
    const ORDERED: bool = true;
}

#[bootstrap(
    features("contrib"),
    arguments(constant(rust_type = "$get_atom(get_type(input_domain))")),
    generics(TA(suppress), MI(suppress), MO(default = "SymmetricDistance"))
)]
#[deprecated(
    since = "0.17.0",
    note = "Use Polars instead for new analyses; no direct replacement exists."
)]
/// Make a Transformation that either truncates or imputes records
/// with `constant` to match a provided `size`.
///
/// # Arguments
/// * `input_domain` - Domain of input data.
/// * `input_metric` - Metric of input data.
/// * `size` - Number of records in output data.
/// * `constant` - Value to impute with.
///
/// # Generics
/// * `TA` - Atomic Type.
/// * `MI` - Input Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
/// * `MO` - Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
///
/// # Returns
/// A vector of the same type `TA`, but with the provided `size`.
pub fn make_resize<TA, MI, MO>(
    input_domain: VectorDomain<AtomDomain<TA>>,
    input_metric: MI,
    size: usize,
    constant: TA,
) -> Fallible<Transformation<VectorDomain<AtomDomain<TA>>, MI, VectorDomain<AtomDomain<TA>>, MO>>
where
    TA: 'static + Clone + CheckAtom,
    MI: IsMetricOrdered<Distance = IntDistance>,
    MO: IsMetricOrdered<Distance = IntDistance> + Default,
    (VectorDomain<AtomDomain<TA>>, MI): MetricSpace,
    (VectorDomain<AtomDomain<TA>>, MO): MetricSpace,
{
    if !input_domain.element_domain.member(&constant)? {
        return fallible!(MakeTransformation, "constant must be a member of DA");
    }
    if size == 0 {
        return fallible!(MakeTransformation, "row size must be greater than zero");
    }

    Transformation::new(
        input_domain.clone(),
        input_metric,
        input_domain.with_size(size),
        MO::default(),
        Function::new_fallible(move |arg: &Vec<TA>| {
            Ok(match arg.len().cmp(&size) {
                Ordering::Less | Ordering::Equal => {
                    // number of records to impute. Checked to ensure the subtraction cannot underflow
                    let padding = size.checked_sub(arg.len()).ok_or_else(|| {
                        err!(
                            FailedFunction,
                            "input length must not exceed the target size"
                        )
                    })?;
                    let mut data = Vec::new();
                    // checked allocation: fails with an error instead of panicking
                    // (capacity overflow) or aborting (failed allocation)
                    data.try_reserve_exact(size).map_err(|_| {
                        err!(
                            FailedFunction,
                            "failed to allocate memory to resize dataset to length {size}"
                        )
                    })?;
                    data.extend(arg.iter().cloned());
                    data.extend((0..padding).map(|_| constant.clone()));
                    // Randomize the order only when converting unordered input to ordered output.
                    if MO::ORDERED && !MI::ORDERED {
                        data.shuffle()?;
                    }
                    data
                }
                Ordering::Greater => {
                    let mut data = arg.clone();
                    // if input metric is not ordered, then shuffle so that the slice is a random draw from the data
                    if !MI::ORDERED {
                        data.shuffle()?;
                    }
                    data[..size].to_vec()
                }
            })
        }),
        // Consider when a dataset has zero records and is resized to length 1.
        // The resulting dataset will be `vec![constant]`
        // Now consider a neighboring dataset that differs by one addition of `value`.
        // The resulting dataset will be `vec![value]`.
        // `vec![constant]` and `vec![value]` differ by an addition and deletion, or distance 2.
        // In the worst case, for each addition in the input, there are two changes in the output
        StabilityMap::new_from_constant(2),
    )
}

#[cfg(test)]
mod test;

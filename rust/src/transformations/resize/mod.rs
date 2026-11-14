#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::{InsertDeleteDistance, IntDistance, SymmetricDistance};
use crate::traits::CheckAtom;
use crate::traits::samplers::{Shuffle, sample_uniform_uint_below};

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
            // Reserve the full output vector up front, so that allocation failure
            // depends only on the fixed target size, not on the length of the input.
            let mut data = Vec::new();
            data.try_reserve_exact(size)
                .map_err(|_| err!(FailedFunction, "failed to allocate output vector"))?;

            data.extend(arg.iter().take(size).cloned());

            if arg.len() < size {
                // Impute missing records.
                data.extend((arg.len()..size).map(|_| constant.clone()));
            } else if arg.len() > size && !MI::ORDERED {
                // Uniformly sample without replacement using reservoir sampling.
                for (i, value) in arg.iter().enumerate().skip(size) {
                    let j = sample_uniform_uint_below(i + 1)?;
                    if j < size {
                        data[j] = value.clone();
                    }
                }
            }

            // Randomize the order only when converting unordered input to ordered output.
            if !MI::ORDERED && MO::ORDERED {
                data.shuffle()?;
            }

            Ok(data)
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

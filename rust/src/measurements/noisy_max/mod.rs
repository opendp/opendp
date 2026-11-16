use crate::{
    core::{Function, Measurement},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::LInfDistance,
    traits::{CastInternalRational, InfCast, Number},
};
use dashu::float::FBig;
use opendp_derive::bootstrap;

use super::{TopKMeasure, make_noisy_top_k};

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        negate(default = false),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Citations
/// * [MS20 Permute-and-Flip: A New Mechanism for Differentially Private Selection](https://arxiv.org/abs/2010.12603)
///
/// # Runtime
/// Per release, worst-case runtime is `O(n)`, where `n` is the number of scores.
///
/// # Utility
/// When `scale = 0`, the mechanism returns the exact maximizing index.
/// For positive `scale`, utility improves as score gaps grow relative to `scale`.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable `VectorDomain`
/// * `input_metric` - Metric on the input domain. Must be `LInfDistance`
/// * `output_measure` - One of `MaxDivergence`, `ZeroConcentratedDivergence`
/// * `scale` - Scale for the noise distribution
/// * `negate` - Set to true to return min
///
/// # Generics
/// * `MO` - Output measure. One of `MaxDivergence` or `ZeroConcentratedDivergence`
/// * `TIA` - Atom Input Type. Type of each element in the score vector
pub fn make_noisy_max<MO: TopKMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    scale: f64,
    negate: bool,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, LInfDistance<TIA>, MO, usize>>
where
    TIA: Number + CastInternalRational,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    make_noisy_top_k(input_domain, input_metric, output_measure, 1, scale, negate)
        >> Function::new_fallible(|arg: &Vec<usize>| {
            arg.get(0)
                .cloned()
                .ok_or_else(|| err!(FailedFunction, "candidates set is empty"))
        })
}

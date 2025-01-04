use crate::{
    core::{Function, Measurement},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::LInfDistance,
    traits::{InfCast, Number},
};
use dashu::float::FBig;
use opendp_derive::bootstrap;

use super::{Optimize, SelectionMeasure, make_report_noisy_top_k};

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        optimize(c_type = "char *", rust_type = "String", default = "max"),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable `VectorDomain`
/// * `input_metric` - Metric on the input domain. Must be `LInfDistance`
/// * `output_measure` - One of `MaxDivergence` or `RangeDivergence`
/// * `scale` - Scale for the noise distribution
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `MO` - Output measure. One of `MaxDivergence` or `ZeroConcentratedDivergence`
/// * `TIA` - Atom Input Type. Type of each element in the score vector
pub fn make_noisy_max<MO: TopKMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    scale: f64,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MO>>
where
    TIA: Number,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    make_report_noisy_top_k(
        input_domain,
        input_metric,
        output_measure,
        1,
        scale,
        optimize,
    ) >> Function::new(|arg: &Vec<usize>| arg[0])
}

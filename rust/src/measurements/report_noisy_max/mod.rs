use std::fmt::Display;

use crate::{
    combinators::make_bounded_range_to_pureDP,
    core::{Function, Measurement},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{MaxDivergence, RangeDivergence},
    metrics::LInfDistance,
    traits::{CastInternalRational, InfCast, Number},
};
use dashu::float::FBig;
use opendp_derive::bootstrap;

use super::{SelectionMeasure, make_report_noisy_top_k};

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
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable `VectorDomain`
/// * `input_metric` - Metric on the input domain. Must be `LInfDistance`
/// * `output_measure` - One of `MaxDivergence`, `RangeDivergence`, `ZeroConcentratedDivergence`
/// * `scale` - Scale for the noise distribution
/// * `negate` - Set to true to return min
///
/// # Generics
/// * `MO` - Output measure. Must be one of `MaxDivergence` or `RangeDivergence`
/// * `TIA` - Atom Input Type. Type of each element in the score vector
pub fn make_report_noisy_max<MO: SelectionMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    scale: f64,
    negate: bool,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MO>>
where
    TIA: Number + CastInternalRational,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    make_report_noisy_top_k(input_domain, input_metric, output_measure, 1, scale, negate)
        >> Function::new(|arg: &Vec<usize>| arg[0])
}

#[bootstrap(
    features("contrib"),
    arguments(optimize(c_type = "char *", rust_type = "String", default = "max"),),
    generics(MO(suppress), TIA(suppress))
)]
#[deprecated(since = "0.14.0", note = "use `make_report_noisy_max` instead")]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable `VectorDomain`
/// * `input_metric` - Metric on the input domain. Must be `LInfDistance`
/// * `scale` - Scale for the noise distribution
/// * `negate` - Set to true to return min
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector
pub fn make_report_noisy_max_gumbel<TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    scale: f64,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MaxDivergence>>
where
    TIA: Number + CastInternalRational,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    make_bounded_range_to_pureDP(make_report_noisy_max(
        input_domain,
        input_metric,
        RangeDivergence,
        scale,
        matches!(optimize, Optimize::Max),
    )?)
}

#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "polars", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "polars", serde(rename_all = "lowercase"))]
pub enum Optimize {
    Min,
    Max,
}

impl Display for Optimize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Optimize::Min => f.write_str("min"),
            Optimize::Max => f.write_str("max"),
        }
    }
}

impl TryFrom<&str> for Optimize {
    type Error = crate::error::Error;
    fn try_from(s: &str) -> Fallible<Self> {
        Ok(match s {
            "min" => Optimize::Min,
            "max" => Optimize::Max,
            _ => return fallible!(FailedCast, "optimize must be \"min\" or \"max\""),
        })
    }
}

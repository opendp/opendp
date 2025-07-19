#[cfg(feature = "polars")]
use crate::measurements::expr_report_noisy_max::SelectionDistribution;
use crate::{
    combinators::make_bounded_range_to_zCDP,
    core::{Measure, Measurement},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{exponential::make_permute_and_flip, gumbel::make_report_noisy_top_k_gumbel},
    measures::{MaxDivergence, RangeDivergence, ZeroConcentratedDivergence},
    metrics::LInfDistance,
    traits::{CastInternalRational, InfCast, Number},
};
use dashu::float::FBig;
use opendp_derive::bootstrap;

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

pub(crate) mod exponential;
pub(crate) mod gumbel;

#[bootstrap(
    features("contrib"),
    arguments(
        optimize(c_type = "char *", rust_type = "String"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `output_measure` - One of `MaxDivergence` or `BoundedRange`.
/// * `k` - Number of indices to select.
/// * `scale` - Scale for the noise distribution.
/// * `negate` - Set to true to return bottom k
///
/// # Generics
/// * `MO` - Output Measure.
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
pub fn make_report_noisy_top_k<MO: SelectionMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    k: usize,
    scale: f64,
    negate: bool,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, MO>>
where
    TIA: Number + CastInternalRational,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    output_measure.make(input_domain, input_metric, k, scale, negate)
}

pub trait SelectionMeasure: Measure<Distance = f64> + 'static {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: SelectionDistribution;
    fn make<TIA: Number>(
        self,
        input_domain: VectorDomain<AtomDomain<TIA>>,
        input_metric: LInfDistance<TIA>,
        k: usize,
        scale: f64,
        negate: bool,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, Self>>
    where
        TIA: Number + CastInternalRational,
        FBig: TryFrom<TIA> + TryFrom<f64>,
        f64: InfCast<TIA> + InfCast<usize>;
}

impl SelectionMeasure for RangeDivergence {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: SelectionDistribution = SelectionDistribution::Gumbel;

    fn make<TIA>(
        self,
        input_domain: VectorDomain<AtomDomain<TIA>>,
        input_metric: LInfDistance<TIA>,
        k: usize,
        scale: f64,
        negate: bool,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, Self>>
    where
        TIA: Number,
        FBig: TryFrom<TIA> + TryFrom<f64>,
        f64: InfCast<TIA> + InfCast<usize>,
    {
        make_report_noisy_top_k_gumbel(input_domain, input_metric, k, scale, negate)
    }
}

impl SelectionMeasure for ZeroConcentratedDivergence {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: SelectionDistribution = SelectionDistribution::Gumbel;

    fn make<TIA>(
        self,
        input_domain: VectorDomain<AtomDomain<TIA>>,
        input_metric: LInfDistance<TIA>,
        k: usize,
        scale: f64,
        negate: bool,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, Self>>
    where
        TIA: Number,
        FBig: TryFrom<TIA> + TryFrom<f64>,
        f64: InfCast<TIA> + InfCast<usize>,
    {
        make_bounded_range_to_zCDP(make_report_noisy_top_k_gumbel(
            input_domain,
            input_metric,
            k,
            scale,
            negate,
        )?)
    }
}

impl SelectionMeasure for MaxDivergence {
    #[cfg(feature = "polars")]
    const DISTRIBUTION: SelectionDistribution = SelectionDistribution::Exponential;

    fn make<TIA>(
        self,
        input_domain: VectorDomain<AtomDomain<TIA>>,
        input_metric: LInfDistance<TIA>,
        k: usize,
        scale: f64,
        negate: bool,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, Self>>
    where
        TIA: Number + CastInternalRational,
        FBig: TryFrom<TIA> + TryFrom<f64>,
        f64: InfCast<TIA> + InfCast<usize>,
    {
        make_permute_and_flip(input_domain, input_metric, k, scale, negate)
    }
}

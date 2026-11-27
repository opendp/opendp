mod laplace;
pub use laplace::*;

mod gaussian;
pub use gaussian::*;
use opendp_derive::bootstrap;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace},
    error::Fallible,
    measurements::{DiscreteGaussian, DiscreteLaplace, MakeNoiseThreshold, NoiseDomain},
    measures::{Approximate, MaxDivergence, MultiDP, ZeroConcentratedDivergence},
    metrics::{AbsoluteDistance, L01InfDistance, L02InfDistance},
    traits::Number,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        threshold(c_type = "void *", rust_type = "TV"),
        k(default = b"null"),
    ),
    generics(DI(suppress), MI(suppress), MO(suppress)),
    derived_types(TV = "$get_value_type(get_carrier_type(input_domain))")
)]
/// Make a Measurement that uses propose-test-release to release a hashmap of counts.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Citations
/// * [Rogers23 A Unifying Privacy Analysis Framework for Unknown Domain Algorithms in Differential Privacy](https://arxiv.org/abs/2309.09170)
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/abs/2004.00010)
///
/// # Proof Navigation
/// * [`make_laplace_threshold`] and [`make_gaussian_threshold`] are the distribution-specific entry points.
/// * [`MakeNoiseThreshold`] covers distribution-specific measurement construction.
/// * [`NoiseThresholdPrivacyMap`] covers threshold-specific privacy accounting.
///
/// # Runtime
/// For an input map with `m` entries, each release performs `O(m)` wrapper work
/// plus one draw from the underlying thresholded noise sampler per entry.
///
/// # Utility
/// Utility is governed by the tail of the chosen noise distribution.
/// If an item is separated from the threshold by a margin `g`,
/// false-positive and false-negative probabilities decay according to that tail:
/// exponentially in `g / scale` for Laplace-like tails and
/// subgaussianly in `(g / scale)^2` for Gaussian-like tails.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric for the input domain.
/// * `output_measure` - Privacy measure used for accounting. Legacy approximate measures preserve their historical distribution; `MultiDP` selects from metric geometry.
/// * `scale` - Noise scale parameter.
/// * `threshold` - Exclude counts that are less than this minimum value.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Input Domain.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_noise_threshold<DI: NoiseDomain, MI: Metric, MO: NoiseThresholdMeasureFor<DI, MI>>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    scale: f64,
    threshold: DI::Atom,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    (DI, MI): MetricSpace,
{
    let mut measurement = output_measure.clone().make_noise_threshold(
        (input_domain, input_metric),
        threshold,
        scale,
        k,
    )?;
    measurement.output_measure = output_measure;
    Ok(measurement)
}

pub trait NoiseThresholdMeasureFor<DI: NoiseDomain, MI: Metric>: Measure + 'static {
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>>
    where
        (DI, MI): MetricSpace;
}

impl<DI: NoiseDomain, MI: Metric> NoiseThresholdMeasureFor<DI, MI> for Approximate<MaxDivergence>
where
    (DI, MI): MetricSpace,
    DiscreteLaplace: MakeNoiseThreshold<DI, MI, Self, Threshold = DI::Atom>,
{
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        make_laplace_threshold(input_space.0, input_space.1, scale, threshold, k)
    }
}

impl<DI: NoiseDomain, MI: Metric> NoiseThresholdMeasureFor<DI, MI>
    for Approximate<ZeroConcentratedDivergence>
where
    (DI, MI): MetricSpace,
    DiscreteGaussian: MakeNoiseThreshold<DI, MI, Self, Threshold = DI::Atom>,
{
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        make_gaussian_threshold(input_space.0, input_space.1, scale, threshold, k)
    }
}

impl<DI: NoiseDomain, Q: Number> NoiseThresholdMeasureFor<DI, L01InfDistance<AbsoluteDistance<Q>>>
    for MultiDP
where
    (DI, L01InfDistance<AbsoluteDistance<Q>>): MetricSpace,
    DiscreteLaplace:
        MakeNoiseThreshold<DI, L01InfDistance<AbsoluteDistance<Q>>, Self, Threshold = DI::Atom>,
{
    fn make_noise_threshold(
        self,
        input_space: (DI, L01InfDistance<AbsoluteDistance<Q>>),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, L01InfDistance<AbsoluteDistance<Q>>, Self, DI::Carrier>> {
        make_laplace_threshold(input_space.0, input_space.1, scale, threshold, k)
    }
}

impl<DI: NoiseDomain, Q: Number> NoiseThresholdMeasureFor<DI, L02InfDistance<AbsoluteDistance<Q>>>
    for MultiDP
where
    (DI, L02InfDistance<AbsoluteDistance<Q>>): MetricSpace,
    DiscreteGaussian:
        MakeNoiseThreshold<DI, L02InfDistance<AbsoluteDistance<Q>>, Self, Threshold = DI::Atom>,
{
    fn make_noise_threshold(
        self,
        input_space: (DI, L02InfDistance<AbsoluteDistance<Q>>),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, L02InfDistance<AbsoluteDistance<Q>>, Self, DI::Carrier>> {
        make_gaussian_threshold(input_space.0, input_space.1, scale, threshold, k)
    }
}

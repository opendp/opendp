mod laplace;
pub use laplace::*;

mod gaussian;
pub use gaussian::*;
use opendp_derive::bootstrap;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace},
    error::Fallible,
    measurements::{
        DiscreteGaussian, DiscreteLaplace, MakeNoiseThreshold, NoiseDistribution, NoiseDomain,
        NoiseMetric,
    },
    measures::{Approximate, MultiDP, PureDP, zCDP},
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
        distribution(c_type = "char *", rust_type = b"null", default = b"null"),
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
/// * `output_measure` - Privacy measure used for accounting. `PureDP` and `zCDP` preserve their historical distribution inference; `MultiDP` requires either an explicit distribution or an unambiguous metric.
/// * `scale` - Noise scale parameter.
/// * `threshold` - Exclude counts that are less than this minimum value.
/// * `k` - The noise granularity in terms of 2^k.
/// * `distribution` - Optional distribution: `"laplace"` or `"gaussian"`.
///
/// # Generics
/// * `DI` - Input Domain.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_noise_threshold<DI: NoiseDomain, MI: Metric + NoiseMetric, MO: NoiseThresholdMeasure>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    scale: f64,
    threshold: DI::Atom,
    k: Option<i32>,
    distribution: Option<String>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    MO: NoiseThresholdMeasureFor<DI, MI>,
    (DI, MI): MetricSpace,
{
    let distribution = distribution
        .as_deref()
        .map(NoiseDistribution::try_from)
        .transpose()?
        .or_else(|| MO::legacy_distribution().or_else(|| MI::multidp_distribution()))
        .ok_or_else(|| {
            err!(
                MakeMeasurement,
                "distribution is required when it cannot be inferred"
            )
        })?;

    output_measure.make_noise_threshold(
        (input_domain, input_metric),
        threshold,
        scale,
        k,
        distribution,
    )
}

pub trait NoiseThresholdMeasure: Measure + 'static {
    fn legacy_distribution() -> Option<NoiseDistribution>;
}

impl NoiseThresholdMeasure for Approximate<PureDP> {
    fn legacy_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Laplace)
    }
}

impl NoiseThresholdMeasure for Approximate<zCDP> {
    fn legacy_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Gaussian)
    }
}

impl NoiseThresholdMeasure for Approximate<MultiDP> {
    fn legacy_distribution() -> Option<NoiseDistribution> {
        None
    }
}

pub trait NoiseThresholdMeasureFor<DI: NoiseDomain, MI: Metric>:
    NoiseThresholdMeasure + Sized
{
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>>
    where
        (DI, MI): MetricSpace;
}

impl<DI: NoiseDomain, MI: Metric> NoiseThresholdMeasureFor<DI, MI> for Approximate<PureDP>
where
    (DI, MI): MetricSpace,
    DiscreteLaplace: MakeNoiseThreshold<DI, MI, Approximate<PureDP>, Threshold = DI::Atom>,
{
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        match distribution {
            NoiseDistribution::Laplace => {
                crate::measurements::noise_threshold::distribution::laplace::make_laplace_threshold(
                    input_space.0,
                    input_space.1,
                    scale,
                    threshold,
                    k,
                )
            }
            NoiseDistribution::Gaussian => {
                fallible!(
                    MakeMeasurement,
                    "gaussian distribution is incompatible with ApproxDP"
                )
            }
        }
    }
}

impl<DI: NoiseDomain, MI: Metric> NoiseThresholdMeasureFor<DI, MI> for Approximate<zCDP>
where
    (DI, MI): MetricSpace,
    DiscreteGaussian: MakeNoiseThreshold<DI, MI, Approximate<zCDP>, Threshold = DI::Atom>,
{
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        match distribution {
            NoiseDistribution::Gaussian => crate::measurements::noise_threshold::distribution::gaussian::make_gaussian_threshold(
                input_space.0, input_space.1, scale, threshold, k,
            ),
            NoiseDistribution::Laplace => {
                fallible!(MakeMeasurement, "laplace distribution is incompatible with ApproxZCDP")
            }
        }
    }
}

impl<DI: NoiseDomain, Q: 'static + crate::traits::Number>
    NoiseThresholdMeasureFor<
        DI,
        crate::metrics::L01InfDistance<crate::metrics::AbsoluteDistance<Q>>,
    > for Approximate<MultiDP>
where
    (
        DI,
        crate::metrics::L01InfDistance<crate::metrics::AbsoluteDistance<Q>>,
    ): MetricSpace,
    DiscreteLaplace: MakeNoiseThreshold<
            DI,
            crate::metrics::L01InfDistance<crate::metrics::AbsoluteDistance<Q>>,
            Approximate<MultiDP>,
            Threshold = DI::Atom,
        >,
{
    fn make_noise_threshold(
        self,
        input_space: (
            DI,
            crate::metrics::L01InfDistance<crate::metrics::AbsoluteDistance<Q>>,
        ),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<
        Measurement<
            DI,
            crate::metrics::L01InfDistance<crate::metrics::AbsoluteDistance<Q>>,
            Self,
            DI::Carrier,
        >,
    > {
        match distribution {
            NoiseDistribution::Laplace => {
                crate::measurements::noise_threshold::distribution::laplace::make_laplace_threshold(
                    input_space.0,
                    input_space.1,
                    scale,
                    threshold,
                    k,
                )
            }
            NoiseDistribution::Gaussian => fallible!(
                MakeMeasurement,
                "gaussian distribution is incompatible with L01InfDistance"
            ),
        }
    }
}

impl<DI: NoiseDomain, Q: 'static + crate::traits::Number>
    NoiseThresholdMeasureFor<
        DI,
        crate::metrics::L02InfDistance<crate::metrics::AbsoluteDistance<Q>>,
    > for Approximate<MultiDP>
where
    (
        DI,
        crate::metrics::L02InfDistance<crate::metrics::AbsoluteDistance<Q>>,
    ): MetricSpace,
    DiscreteGaussian: MakeNoiseThreshold<
            DI,
            crate::metrics::L02InfDistance<crate::metrics::AbsoluteDistance<Q>>,
            Approximate<MultiDP>,
            Threshold = DI::Atom,
        >,
{
    fn make_noise_threshold(
        self,
        input_space: (
            DI,
            crate::metrics::L02InfDistance<crate::metrics::AbsoluteDistance<Q>>,
        ),
        threshold: DI::Atom,
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<
        Measurement<
            DI,
            crate::metrics::L02InfDistance<crate::metrics::AbsoluteDistance<Q>>,
            Self,
            DI::Carrier,
        >,
    > {
        match distribution {
            NoiseDistribution::Gaussian => crate::measurements::noise_threshold::distribution::gaussian::make_gaussian_threshold(input_space.0, input_space.1, scale, threshold, k),
            NoiseDistribution::Laplace => fallible!(MakeMeasurement, "laplace distribution is incompatible with L02InfDistance"),
        }
    }
}

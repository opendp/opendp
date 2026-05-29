mod laplace;
pub use laplace::*;

mod gaussian;
pub use gaussian::*;
use opendp_derive::bootstrap;

use crate::{
    core::{Measurement, Metric, MetricSpace},
    error::Fallible,
    measurements::{MakeNoiseThreshold, NoiseDomain, NoiseMeasure},
    measures::Approximate,
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
/// * `output_measure` - Privacy measure. Either `MaxDivergence` or `ZeroConcentratedDivergence`.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `threshold` - Exclude counts that are less than this minimum value.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Input Domain.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_noise_threshold<DI: NoiseDomain, MI: Metric, MO: NoiseMeasure>(
    input_domain: DI,
    input_metric: MI,
    output_measure: Approximate<MO>,
    scale: f64,
    threshold: DI::Atom,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, Approximate<MO>, DI::Carrier>>
where
    MO::Distribution: MakeNoiseThreshold<DI, MI, Approximate<MO>, Threshold = DI::Atom>,
    (DI, MI): MetricSpace,
{
    (output_measure.0)
        .new_distribution(scale, k)
        .make_noise_threshold((input_domain, input_metric), threshold)
}

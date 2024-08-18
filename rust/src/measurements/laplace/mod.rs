use core::f64;

use dashu::{base::Signed, rational::RBig};

use crate::core::{Measure, Metric, MetricSpace};
use crate::{
    core::{Measurement, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    metrics::L1Distance,
    traits::InfCast,
};

use super::{make_noise, MakeNoise, Nature, NoisePrivacyMap, ZExpFamily};

use super::NoiseDomain;

#[cfg(test)]
mod test;

// #[bootstrap(
//     features("contrib"),
//     arguments(k(default = b"null")),
//     generics(D(suppress))
// )]
/// Make a Measurement that adds noise from the Laplace(`scale`) distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
///
/// Internally, all sampling is done using the discrete Laplace distribution.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `k` - The noise granularity in terms of 2^k, only valid for domains over floats.
///
/// # Generics
/// * `D` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
pub fn make_laplace<DI: NoiseDomain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    DI::Atom: Nature<1>,
    ((DI, MI), <DI::Atom as Nature<1>>::Dist): MakeNoise<DI, MI, <DI::Atom as Nature<1>>::Dist, MO>,
    (DI, MI): MetricSpace,
{
    let distribution = DI::Atom::new_distribution(scale, k)?;
    make_noise(input_domain, input_metric, distribution)
}

impl NoisePrivacyMap<L1Distance<RBig>, MaxDivergence, ZExpFamily<1>>
    for ((L1Distance<RBig>, MaxDivergence), ZExpFamily<1>)
{
    fn privacy_map(
        distribution: ZExpFamily<1>,
    ) -> Fallible<PrivacyMap<L1Distance<RBig>, MaxDivergence>> {
        let ZExpFamily { scale } = distribution;
        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
        }
        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in.is_negative() {
                return fallible!(FailedMap, "sensitivity ({}) must be positive", d_in);
            }

            if d_in.is_zero() {
                return Ok(0.0);
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            // d_in / scale
            f64::inf_cast(d_in / scale.clone())
        }))
    }
}

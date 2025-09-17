use core::f64;

use dashu::{base::Signed, rational::RBig};
use opendp_derive::{bootstrap, proven};

use crate::core::{Measure, Metric, MetricSpace};
use crate::measurements::noise::nature::Nature;
use crate::measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily};
use crate::{
    core::{Domain, Measurement, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    metrics::L1Distance,
    traits::InfCast,
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(k(default = b"null")),
    generics(DI(suppress), MI(suppress), MO(default = "MaxDivergence"))
)]
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
/// * `DI` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `MI` - Metric used to measure distance between members of the input domain.
/// * `MO` - Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
pub fn make_laplace<DI: Domain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    DiscreteLaplace: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    DiscreteLaplace { scale, k }.make_noise((input_domain, input_metric))
}

pub struct DiscreteLaplace {
    pub scale: f64,
    pub k: Option<i32>,
}

/// Laplace mechanism
#[proven(proof_path = "measurements/noise/distribution/laplace/MakeNoise_for_DiscreteLaplace.tex")]
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoise<DI, MI, MO> for DiscreteLaplace
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<1>: MakeNoise<DI, MI, MO>,
{
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise(input_space)
    }
}

#[proven(
    proof_path = "measurements/noise/distribution/laplace/NoisePrivacyMap_for_ZExpFamily1.tex"
)]
impl NoisePrivacyMap<L1Distance<RBig>, MaxDivergence> for ZExpFamily<1> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L1Distance<RBig>,
        _output_measure: &MaxDivergence,
    ) -> Fallible<PrivacyMap<L1Distance<RBig>, MaxDivergence>> {
        let ZExpFamily { scale } = self.clone();
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

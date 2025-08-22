use dashu::{rational::RBig, rbig};
use opendp_derive::{bootstrap, proven};

use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily, noise::nature::Nature},
    measures::ZeroConcentratedDivergence,
    metrics::L2Distance,
    traits::InfCast,
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(k(default = b"null")),
    generics(DI(suppress), MI(suppress), MO(default = "ZeroConcentratedDivergence"))
)]
/// Make a Measurement that adds noise from the Gaussian(`scale`) distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`          |
/// | ------------------------------- | ------------ | ----------------------- |
/// | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
pub fn make_gaussian<DI: Domain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    DiscreteGaussian: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    DiscreteGaussian { scale, k }.make_noise((input_domain, input_metric))
}

pub struct DiscreteGaussian {
    pub scale: f64,
    pub k: Option<i32>,
}

/// Gaussian mechanism
#[proven(
    proof_path = "measurements/noise/distribution/gaussian/MakeNoise_for_DiscreteGaussian.tex"
)]
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoise<DI, MI, MO> for DiscreteGaussian
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<2>: MakeNoise<DI, MI, MO>,
{
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise(input_space)
    }
}

#[proven(
    proof_path = "measurements/noise/distribution/gaussian/NoisePrivacyMap_for_ZExpFamily2.tex"
)]
impl NoisePrivacyMap<L2Distance<RBig>, ZeroConcentratedDivergence> for ZExpFamily<2> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L2Distance<RBig>,
        _outut_measure: &ZeroConcentratedDivergence,
    ) -> Fallible<PrivacyMap<L2Distance<RBig>, ZeroConcentratedDivergence>> {
        let ZExpFamily { scale } = self.clone();
        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({scale}) must be non-negative");
        }
        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in < &RBig::ZERO {
                return fallible!(FailedMap, "sensitivity ({d_in}) must be non-negative");
            }

            if d_in.is_zero() {
                return Ok(0.0);
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            f64::inf_cast((d_in / scale.clone()).pow(2) / rbig!(2))
        }))
    }
}

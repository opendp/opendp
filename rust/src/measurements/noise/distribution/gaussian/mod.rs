use dashu::{rational::RBig, rbig};
use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measurements::{noise::nature::Nature, MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily},
    measures::ZeroConcentratedDivergence,
    metrics::L2Distance,
    traits::InfCast,
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

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
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    Gaussian: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    Gaussian { scale, k }.make_noise((input_domain, input_metric))
}

pub struct Gaussian {
    pub scale: f64,
    pub k: Option<i32>,
}

/// Gaussian mechanism
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoise<DI, MI, MO> for Gaussian
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature<2>,
    <DI::Atom as Nature<2>>::RV: MakeNoise<DI, MI, MO>,
{
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise(input_space)
    }
}

impl NoisePrivacyMap<L2Distance<RBig>, ZeroConcentratedDivergence> for ZExpFamily<2> {
    fn noise_privacy_map(
        self,
    ) -> Fallible<PrivacyMap<L2Distance<RBig>, ZeroConcentratedDivergence>> {
        let ZExpFamily { scale } = self;
        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale must not be negative");
        }
        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in < &RBig::ZERO {
                return fallible!(FailedMap, "sensitivity ({}) must be positive", d_in);
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

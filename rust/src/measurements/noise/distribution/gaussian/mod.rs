use core::f64;

use dashu::{integer::UBig, rational::RBig, rbig};
use opendp_derive::{bootstrap, proven};

use crate::{
    accuracy::{
        conservative_discrete_gaussian_tail_to_alpha,
        conservative_discrete_gaussian_tail_to_alpha_lower,
    },
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily, noise::nature::Nature},
    measures::{Approximate, ZeroConcentratedDivergence},
    metrics::L2Distance,
    traits::{InfCast, InfSub},
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
        radius(c_type = "void *", rust_type = "Option<T>", default = b"null"),
        k(default = b"null")
    ),
    generics(DI(suppress), MI(suppress), MO(default = "ZeroConcentratedDivergence")),
    derived_types(T = "$get_atom(get_carrier_type(input_domain))")
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
/// * `radius` - The radius of noise to be added. This is used to bound the tail of the distribution.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
pub fn make_gaussian<DI: NoiseDomain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    radius: Option<DI::Atom>,
    k: Option<i32>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    DiscreteGaussian<DI::Atom>: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    DiscreteGaussian { scale, k, radius }.make_noise((input_domain, input_metric))
}

pub struct DiscreteGaussian<T> {
    pub scale: f64,
    pub k: Option<i32>,
    pub radius: Option<T>,
}

/// Gaussian mechanism
#[proven(
    proof_path = "measurements/noise/distribution/gaussian/MakeNoise_for_DiscreteGaussian.tex"
)]
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoise<DI, MI, MO>
    for DiscreteGaussian<DI::Atom>
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<2>: MakeNoise<DI, MI, MO>,
{
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>> {
        DI::Atom::new_distribution(self.scale, self.k, self.radius)?.make_noise(input_space)
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
        if let Some(ref radius) = self.radius {
            return fallible!(
                MakeMeasurement,
                "radius ({}) introduces a delta parameter that is not compatible with zCDP",
                radius
            );
        }
        let scale = self.scale.clone();
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

impl NoisePrivacyMap<L2Distance<RBig>, Approximate<ZeroConcentratedDivergence>> for ZExpFamily<2> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L2Distance<RBig>,
        output_measure: &Approximate<ZeroConcentratedDivergence>,
    ) -> Fallible<PrivacyMap<L2Distance<RBig>, Approximate<ZeroConcentratedDivergence>>> {
        let distribution = ZExpFamily {
            scale: self.scale.clone(),
            radius: None,
        };

        let noise_privacy_map =
            distribution.noise_privacy_map(&L2Distance::default(), &output_measure.0)?;

        let scale = self.scale.clone();
        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
        }

        let radius = self.radius.clone();
        if radius == Some(UBig::ZERO) {
            return fallible!(MakeMeasurement, "radius must not be non-zero");
        }

        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in.is_zero() {
                return Ok((0.0, 0.0));
            }

            if scale.clone().is_zero() {
                return Ok((f64::INFINITY, 0.0));
            }

            let rho = noise_privacy_map.eval(d_in)?;

            if let Some(r) = radius.clone() {
                let (_, d_in_floor) = d_in.floor().into_parts();
                if r <= d_in_floor {
                    return Ok((0.0, 1.0));
                } else {
                    let large_tail_upper_bound = conservative_discrete_gaussian_tail_to_alpha(
                        scale.clone(),
                        r.clone() - d_in_floor,
                    )?;
                    let small_tail_upper_bound =
                        conservative_discrete_gaussian_tail_to_alpha_lower(scale.clone(), r)?;
                    let delta = large_tail_upper_bound.inf_sub(&small_tail_upper_bound)?;
                    return Ok((rho, delta));
                };
            } else {
                return Ok((rho, 0.0));
            };
        }))
    }
}

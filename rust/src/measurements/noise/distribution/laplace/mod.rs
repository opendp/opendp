use core::f64;
use dashu::{base::Signed, integer::UBig, rational::RBig};
use opendp_derive::{bootstrap, proven};

use crate::core::{Measure, Metric, MetricSpace};
use crate::measurements::noise::nature::Nature;
use crate::measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily};
use crate::traits::InfSub;
use crate::{
    accuracy::{
        conservative_discrete_laplacian_tail_to_alpha,
        conservative_discrete_laplacian_tail_to_alpha_lower,
    },
    core::{Measurement, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MaxDivergence},
    metrics::L1Distance,
    traits::InfCast,
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
    generics(DI(suppress), MI(suppress), MO(default = "MaxDivergence")),
    derived_types(T = "$get_atom(get_carrier_type(input_domain))")
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
/// * `radius` - The radius of noise to be added. This is used to bound the tail of the distribution.
/// * `k` - The noise granularity in terms of 2^k, only valid for domains over floats.
///
/// # Generics
/// * `DI` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `MI` - Metric used to measure distance between members of the input domain.
/// * `MO` - Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
pub fn make_laplace<DI: NoiseDomain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    radius: Option<DI::Atom>,
    k: Option<i32>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    DiscreteLaplace<DI::Atom>: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    DiscreteLaplace { scale, k, radius }.make_noise((input_domain, input_metric))
}

pub struct DiscreteLaplace<T> {
    pub scale: f64,
    pub k: Option<i32>,
    pub radius: Option<T>,
}

/// Laplace mechanism
#[proven(proof_path = "measurements/noise/distribution/laplace/MakeNoise_for_DiscreteLaplace.tex")]
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoise<DI, MI, MO>
    for DiscreteLaplace<DI::Atom>
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<1>: MakeNoise<DI, MI, MO>,
{
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>> {
        DI::Atom::new_distribution(self.scale, self.k, self.radius)?.make_noise(input_space)
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
        if let Some(radius) = &self.radius {
            return fallible!(
                MakeMeasurement,
                "radius ({radius}) introduces a delta parameter that is not compatible with pure-DP"
            );
        }
        let scale = self.scale.clone();
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

impl NoisePrivacyMap<L1Distance<RBig>, Approximate<MaxDivergence>> for ZExpFamily<1> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L1Distance<RBig>,
        output_measure: &Approximate<MaxDivergence>,
    ) -> Fallible<PrivacyMap<L1Distance<RBig>, Approximate<MaxDivergence>>> {
        let distribution = ZExpFamily {
            scale: self.scale.clone(),
            radius: None,
        };

        let noise_privacy_map =
            distribution.noise_privacy_map(&L1Distance::default(), &output_measure.0)?;

        let scale = self.scale.clone();
        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
        }

        let radius = self.radius.clone();
        if radius == Some(UBig::ZERO) {
            return fallible!(MakeMeasurement, "radius must not be non-zero");
        }

        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in.is_negative() {
                return fallible!(FailedMap, "sensitivity ({}) must be positive", d_in);
            }

            if d_in.is_zero() {
                return Ok((0.0, 0.0));
            }

            if scale.is_zero() {
                return Ok((f64::INFINITY, 0.0));
            }

            let epsilon = noise_privacy_map.eval(d_in)?;

            if let Some(r) = radius.clone() {
                let (_, d_in_floor) = d_in.floor().into_parts();
                if r <= d_in_floor {
                    return Ok((0.0, 1.0));
                } else {
                    let large_tail_upper_bound = conservative_discrete_laplacian_tail_to_alpha(
                        scale.clone(),
                        r.clone() - d_in_floor,
                    )?;
                    let small_tail_upper_bound =
                        conservative_discrete_laplacian_tail_to_alpha_lower(
                            scale.clone(),
                            r.clone(),
                        )?;
                    let delta = large_tail_upper_bound.inf_sub(&small_tail_upper_bound)?;
                    return Ok((epsilon, delta));
                };
            } else {
                return Ok((epsilon, 0.0));
            };
        }))
    }
}

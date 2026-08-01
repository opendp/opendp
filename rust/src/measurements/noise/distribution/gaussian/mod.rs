use dashu::{rational::RBig, rbig};
use opendp_derive::{bootstrap, proven};

use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily, noise::nature::Nature},
    measures::{PrivacyCurve, PrivacyCurveDP, zCDP},
    metrics::L2Distance,
    traits::{DInterval, InfCast},
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(k(default = b"null")),
    generics(DI(suppress), MI(suppress), MO(default = "zCDP"))
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
/// * `input_domain` - Domain of the data type to be released.
/// * `input_metric` - Metric of the data type to be released.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Domain of the data to be released. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `MO` - Output Measure. The only valid measure is `zCDP`.
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
    DiscreteGaussian { scale, k }.make_noise((input_domain, input_metric), MO::default())
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
    fn make_noise(
        self,
        input_space: (DI, MI),
        output_measure: MO,
    ) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise(input_space, output_measure)
    }
}

#[proven(
    proof_path = "measurements/noise/distribution/gaussian/NoisePrivacyMap_for_ZExpFamily2.tex"
)]
impl NoisePrivacyMap<L2Distance<RBig>, zCDP> for ZExpFamily<2> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L2Distance<RBig>,
        _outut_measure: &zCDP,
    ) -> Fallible<PrivacyMap<L2Distance<RBig>, zCDP>> {
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

#[allow(non_snake_case)]
impl NoisePrivacyMap<L2Distance<RBig>, PrivacyCurveDP> for ZExpFamily<2> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L2Distance<RBig>,
        _output_measure: &PrivacyCurveDP,
    ) -> Fallible<PrivacyMap<L2Distance<RBig>, PrivacyCurveDP>> {
        let ZExpFamily { scale } = self.clone();

        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({scale}) must not be negative");
        }

        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in < &RBig::ZERO {
                return fallible!(FailedMap, "sensitivity ({d_in}) must be non-negative");
            }

            if d_in == &RBig::ZERO {
                return Ok(PrivacyCurve::new().with_zCDP(0.0)?);
            }

            if scale == RBig::ZERO {
                return fallible!(
                    FailedMap,
                    "nonzero sensitivity with zero scale has no finite privacy guarantee"
                );
            }

            let sigma = f64::inf_cast(scale.clone())?;
            let sensitivity = f64::inf_cast(d_in.clone())?;

            if !sigma.is_finite() || sigma <= 0.0 {
                return fallible!(FailedMap, "scale ({sigma}) must be finite and positive");
            }
            if !sensitivity.is_finite() || sensitivity < 0.0 {
                return fallible!(
                    FailedMap,
                    "sensitivity ({sensitivity}) must be finite and non-negative"
                );
            }

            Ok(PrivacyCurve::new().with_zCDP(gaussian_zcdp_upper(sensitivity, sigma)?)?)
        }))
    }
}

fn gaussian_zcdp_upper(sensitivity: f64, sigma: f64) -> Fallible<f64> {
    check_sigma(sigma)?;

    if sensitivity == 0.0 {
        return Ok(0.0);
    }

    let ratio = DInterval::from_approx(sensitivity)?.div(DInterval::from_approx(sigma)?)?;
    DInterval::point(0.5)?
        .mul(ratio.clone().mul(ratio)?)?
        .upper_f64()
}

fn check_sigma(sigma: f64) -> Fallible<()> {
    if !sigma.is_finite() || sigma <= 0.0 {
        return fallible!(FailedMap, "sigma ({sigma}) must be finite and positive");
    }
    Ok(())
}

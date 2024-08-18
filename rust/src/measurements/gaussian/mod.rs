use dashu::{rational::RBig, rbig};

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::L2Distance,
    traits::InfCast,
};

use super::{make_noise, MakeNoise, Nature, NoiseDomain, NoisePrivacyMap, ZExpFamily};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

// #[bootstrap(
//     features("contrib"),
//     arguments(k(default = b"null")),
//     generics(D(suppress), MO(default = "ZeroConcentratedDivergence"), QI(suppress))
// )]
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
pub fn make_gaussian<DI: NoiseDomain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    DI::Atom: Nature<2>,
    ((DI, MI), <DI::Atom as Nature<2>>::Dist): MakeNoise<DI, MI, <DI::Atom as Nature<2>>::Dist, MO>,
    (DI, MI): MetricSpace,
{
    let distribution = DI::Atom::new_distribution(scale, k)?;
    make_noise(input_domain, input_metric, distribution)
}

impl NoisePrivacyMap<L2Distance<RBig>, ZeroConcentratedDivergence, ZExpFamily<2>>
    for (
        (L2Distance<RBig>, ZeroConcentratedDivergence),
        ZExpFamily<2>,
    )
{
    fn privacy_map(
        distribution: ZExpFamily<2>,
    ) -> Fallible<PrivacyMap<L2Distance<RBig>, ZeroConcentratedDivergence>> {
        let ZExpFamily { scale } = distribution;
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

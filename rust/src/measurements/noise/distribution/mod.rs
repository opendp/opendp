use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::MakeNoise,
    measures::{MultiDP, PureDP, zCDP},
    traits::CheckAtom,
};

use opendp_derive::bootstrap;

mod gaussian;
pub use gaussian::*;

mod geometric;
pub use geometric::*;

mod laplace;
pub use laplace::*;

#[cfg(feature = "ffi")]
mod ffi;

pub trait NoiseDomain: Domain {
    type Atom: 'static;
}

impl<T: 'static + CheckAtom> NoiseDomain for AtomDomain<T> {
    type Atom = T;
}

impl<T: 'static + CheckAtom> NoiseDomain for VectorDomain<AtomDomain<T>> {
    type Atom = T;
}

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        k(default = b"null"),
        distribution(c_type = "char *", rust_type = b"null", default = b"null")
    ),
    generics(DI(suppress), MI(suppress), MO(suppress))
)]
/// Make a Measurement that adds noise from the appropriate distribution to the input.
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
/// * `output_measure` - Privacy measure used for accounting. `PureDP` and `zCDP` preserve their historical distribution inference; `MultiDP` requires either an explicit distribution or an unambiguous metric.
/// * `scale` - Noise scale parameter.
/// * `k` - The noise granularity in terms of 2^k.
/// * `distribution` - Optional distribution: `"laplace"` or `"gaussian"`.
///
/// # Generics
/// * `DI` - Domain of the data to be released. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `MO` - Output measure used for privacy accounting.
pub fn make_noise<DI: Domain, MI: Metric + NoiseMetric, MO: NoiseMeasure>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    scale: f64,
    k: Option<i32>,
    distribution: Option<String>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    MO: NoiseMeasureFor<DI, MI>,
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

    output_measure.make_noise((input_domain, input_metric), scale, k, distribution)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseDistribution {
    Laplace,
    Gaussian,
}

impl TryFrom<&str> for NoiseDistribution {
    type Error = crate::error::Error;

    fn try_from(value: &str) -> Fallible<Self> {
        match value.to_ascii_lowercase().as_str() {
            "laplace" => Ok(Self::Laplace),
            "gaussian" => Ok(Self::Gaussian),
            _ => fallible!(
                FailedCast,
                "distribution must be \"laplace\" or \"gaussian\""
            ),
        }
    }
}

pub trait NoiseMetric {
    fn multidp_distribution() -> Option<NoiseDistribution>;
}

impl<Q> NoiseMetric for crate::metrics::AbsoluteDistance<Q> {
    fn multidp_distribution() -> Option<NoiseDistribution> {
        None
    }
}

impl<Q> NoiseMetric for crate::metrics::L1Distance<Q> {
    fn multidp_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Laplace)
    }
}

impl<Q> NoiseMetric for crate::metrics::L2Distance<Q> {
    fn multidp_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Gaussian)
    }
}

impl<Q> NoiseMetric for crate::metrics::L01InfDistance<crate::metrics::AbsoluteDistance<Q>> {
    fn multidp_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Laplace)
    }
}

impl<Q> NoiseMetric for crate::metrics::L02InfDistance<crate::metrics::AbsoluteDistance<Q>> {
    fn multidp_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Gaussian)
    }
}

pub trait NoiseMeasure: Measure + 'static {
    fn legacy_distribution() -> Option<NoiseDistribution>;
}

impl NoiseMeasure for PureDP {
    fn legacy_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Laplace)
    }
}

impl NoiseMeasure for zCDP {
    fn legacy_distribution() -> Option<NoiseDistribution> {
        Some(NoiseDistribution::Gaussian)
    }
}

impl NoiseMeasure for MultiDP {
    fn legacy_distribution() -> Option<NoiseDistribution> {
        None
    }
}

pub trait NoiseMeasureFor<DI: Domain, MI: Metric>: NoiseMeasure + Sized {
    fn make_noise(
        self,
        input_space: (DI, MI),
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>>
    where
        (DI, MI): MetricSpace;
}

impl<DI: Domain, MI: Metric> NoiseMeasureFor<DI, MI> for PureDP
where
    (DI, MI): MetricSpace,
    DiscreteLaplace: MakeNoise<DI, MI, PureDP>,
{
    fn make_noise(
        self,
        input_space: (DI, MI),
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        match distribution {
            NoiseDistribution::Laplace => DiscreteLaplace { scale, k }.make_noise(input_space),
            NoiseDistribution::Gaussian => {
                fallible!(
                    MakeMeasurement,
                    "gaussian distribution is incompatible with PureDP"
                )
            }
        }
    }
}

impl<DI: Domain, MI: Metric> NoiseMeasureFor<DI, MI> for zCDP
where
    (DI, MI): MetricSpace,
    DiscreteGaussian: MakeNoise<DI, MI, zCDP>,
{
    fn make_noise(
        self,
        input_space: (DI, MI),
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        match distribution {
            NoiseDistribution::Gaussian => DiscreteGaussian { scale, k }.make_noise(input_space),
            NoiseDistribution::Laplace => {
                fallible!(
                    MakeMeasurement,
                    "laplace distribution is incompatible with zCDP"
                )
            }
        }
    }
}

impl<DI: Domain, Q: 'static> NoiseMeasureFor<DI, crate::metrics::L1Distance<Q>> for MultiDP
where
    (DI, crate::metrics::L1Distance<Q>): MetricSpace,
    DiscreteLaplace: MakeNoise<DI, crate::metrics::L1Distance<Q>, MultiDP>,
{
    fn make_noise(
        self,
        input_space: (DI, crate::metrics::L1Distance<Q>),
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, crate::metrics::L1Distance<Q>, Self, DI::Carrier>> {
        match distribution {
            NoiseDistribution::Laplace => DiscreteLaplace { scale, k }.make_noise(input_space),
            NoiseDistribution::Gaussian => fallible!(
                MakeMeasurement,
                "gaussian distribution is incompatible with L1Distance"
            ),
        }
    }
}

impl<DI: Domain, Q: 'static> NoiseMeasureFor<DI, crate::metrics::L2Distance<Q>> for MultiDP
where
    (DI, crate::metrics::L2Distance<Q>): MetricSpace,
    DiscreteGaussian: MakeNoise<DI, crate::metrics::L2Distance<Q>, MultiDP>,
{
    fn make_noise(
        self,
        input_space: (DI, crate::metrics::L2Distance<Q>),
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, crate::metrics::L2Distance<Q>, Self, DI::Carrier>> {
        match distribution {
            NoiseDistribution::Gaussian => DiscreteGaussian { scale, k }.make_noise(input_space),
            NoiseDistribution::Laplace => fallible!(
                MakeMeasurement,
                "laplace distribution is incompatible with L2Distance"
            ),
        }
    }
}

impl<DI: Domain, Q: 'static> NoiseMeasureFor<DI, crate::metrics::AbsoluteDistance<Q>> for MultiDP
where
    (DI, crate::metrics::AbsoluteDistance<Q>): MetricSpace,
    DiscreteLaplace: MakeNoise<DI, crate::metrics::AbsoluteDistance<Q>, MultiDP>,
    DiscreteGaussian: MakeNoise<DI, crate::metrics::AbsoluteDistance<Q>, MultiDP>,
{
    fn make_noise(
        self,
        input_space: (DI, crate::metrics::AbsoluteDistance<Q>),
        scale: f64,
        k: Option<i32>,
        distribution: NoiseDistribution,
    ) -> Fallible<Measurement<DI, crate::metrics::AbsoluteDistance<Q>, Self, DI::Carrier>> {
        match distribution {
            NoiseDistribution::Laplace => DiscreteLaplace { scale, k }.make_noise(input_space),
            NoiseDistribution::Gaussian => DiscreteGaussian { scale, k }.make_noise(input_space),
        }
    }
}

#[cfg(test)]
mod test {
    use dashu::{integer::IBig, rbig};

    use super::*;
    use crate::{
        core::Measurement,
        domains::{AtomDomain, VectorDomain},
        metrics::{L1Distance, L2Distance},
    };

    #[test]
    fn test_make_noise_pure_dp_selects_laplace() -> Fallible<()> {
        let measurement: Measurement<_, _, PureDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L1Distance::default(),
            PureDP,
            1.0,
            None,
            None,
        )?;
        assert_eq!(measurement.map(&rbig!(1))?, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_noise_pure_dp_float_selects_laplace() -> Fallible<()> {
        let measurement: Measurement<_, _, PureDP, f64> = make_noise(
            AtomDomain::<f64>::new_non_nan(),
            crate::metrics::AbsoluteDistance::<f64>::default(),
            PureDP,
            1.0,
            None,
            None,
        )?;
        assert_eq!(measurement.map(&1.0)?, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_noise_multidp_infers_geometry_and_requires_scalar_distribution() -> Fallible<()> {
        let laplace: Measurement<_, _, MultiDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L1Distance::default(),
            MultiDP::default(),
            1.0,
            None,
            None,
        )?;
        assert!(laplace.map(&rbig!(1))?.epsilon(1e-3)?.is_finite());

        let gaussian: Measurement<_, _, MultiDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L2Distance::default(),
            MultiDP::default(),
            1.0,
            None,
            None,
        )?;
        assert!(gaussian.map(&rbig!(1))?.epsilon(1e-3)?.is_finite());

        assert!(
            make_noise(
                AtomDomain::<f64>::new_non_nan(),
                crate::metrics::AbsoluteDistance::<f64>::default(),
                MultiDP::default(),
                1.0,
                None,
                None,
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn test_make_noise_zcdp_selects_gaussian() -> Fallible<()> {
        let measurement: Measurement<_, _, zCDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L2Distance::default(),
            zCDP,
            1.0,
            None,
            None,
        )?;
        let guarantee: f64 = measurement.map(&rbig!(1))?;
        assert!(guarantee.is_finite());
        Ok(())
    }
}

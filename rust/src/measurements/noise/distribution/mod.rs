use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::MakeNoise,
    measures::{AccountingPreference, MaxDivergence, MultiDP, ZeroConcentratedDivergence},
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
/// * `output_measure` - Privacy measure used for accounting. `MaxDivergence` and `ZeroConcentratedDivergence` preserve their historical distribution inference; `MultiDP` uses its accounting preference when the metric does not determine the distribution.
/// * `scale` - Noise scale parameter.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Domain of the data to be released. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `MO` - Output measure used for privacy accounting.
pub fn make_noise<DI: Domain, MI: Metric, MO: NoiseMeasureFor<DI, MI>>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    (DI, MI): MetricSpace,
{
    let mut measurement =
        output_measure
            .clone()
            .make_noise((input_domain, input_metric), scale, k)?;
    measurement.output_measure = output_measure;
    Ok(measurement)
}

/// Dispatch to a noise constructor for a compatible measure and metric.
/// The public `make_noise` function preserves the supplied accounting preference.
pub trait NoiseMeasureFor<DI: Domain, MI: Metric>: Measure + 'static {
    fn make_noise(
        self,
        input_space: (DI, MI),
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>>
    where
        (DI, MI): MetricSpace;
}

impl<DI: Domain, MI: Metric> NoiseMeasureFor<DI, MI> for MaxDivergence
where
    (DI, MI): MetricSpace,
    DiscreteLaplace: MakeNoise<DI, MI, Self>,
{
    fn make_noise(
        self,
        input_space: (DI, MI),
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        DiscreteLaplace { scale, k }.make_noise(input_space)
    }
}

impl<DI: Domain, MI: Metric> NoiseMeasureFor<DI, MI> for ZeroConcentratedDivergence
where
    (DI, MI): MetricSpace,
    DiscreteGaussian: MakeNoise<DI, MI, Self>,
{
    fn make_noise(
        self,
        input_space: (DI, MI),
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, MI, Self, DI::Carrier>> {
        DiscreteGaussian { scale, k }.make_noise(input_space)
    }
}

impl<DI: Domain, Q: 'static> NoiseMeasureFor<DI, crate::metrics::L1Distance<Q>> for MultiDP
where
    (DI, crate::metrics::L1Distance<Q>): MetricSpace,
    DiscreteLaplace: MakeNoise<DI, crate::metrics::L1Distance<Q>, Self>,
{
    fn make_noise(
        self,
        input_space: (DI, crate::metrics::L1Distance<Q>),
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, crate::metrics::L1Distance<Q>, Self, DI::Carrier>> {
        DiscreteLaplace { scale, k }.make_noise(input_space)
    }
}

impl<DI: Domain, Q: 'static> NoiseMeasureFor<DI, crate::metrics::L2Distance<Q>> for MultiDP
where
    (DI, crate::metrics::L2Distance<Q>): MetricSpace,
    DiscreteGaussian: MakeNoise<DI, crate::metrics::L2Distance<Q>, Self>,
{
    fn make_noise(
        self,
        input_space: (DI, crate::metrics::L2Distance<Q>),
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, crate::metrics::L2Distance<Q>, Self, DI::Carrier>> {
        DiscreteGaussian { scale, k }.make_noise(input_space)
    }
}

impl<DI: Domain, Q: 'static> NoiseMeasureFor<DI, crate::metrics::AbsoluteDistance<Q>> for MultiDP
where
    (DI, crate::metrics::AbsoluteDistance<Q>): MetricSpace,
    DiscreteLaplace: MakeNoise<DI, crate::metrics::AbsoluteDistance<Q>, Self>,
    DiscreteGaussian: MakeNoise<DI, crate::metrics::AbsoluteDistance<Q>, Self>,
{
    fn make_noise(
        self,
        input_space: (DI, crate::metrics::AbsoluteDistance<Q>),
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<DI, crate::metrics::AbsoluteDistance<Q>, Self, DI::Carrier>> {
        match self.preference() {
            AccountingPreference::PureDp => DiscreteLaplace { scale, k }.make_noise(input_space),
            AccountingPreference::Zcdp => DiscreteGaussian { scale, k }.make_noise(input_space),
            AccountingPreference::Auto => fallible!(
                MakeMeasurement,
                "accounting preference is required for MultiDP scalar noise"
            ),
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
        let measurement: Measurement<_, _, MaxDivergence, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L1Distance::default(),
            MaxDivergence,
            1.0,
            None,
        )?;
        assert_eq!(measurement.map(&rbig!(1))?, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_noise_pure_dp_float_selects_laplace() -> Fallible<()> {
        let measurement: Measurement<_, _, MaxDivergence, f64> = make_noise(
            AtomDomain::<f64>::new_non_nan(),
            crate::metrics::AbsoluteDistance::<f64>::default(),
            MaxDivergence,
            1.0,
            None,
        )?;
        assert_eq!(measurement.map(&1.0)?, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_noise_multidp_infers_geometry_and_requires_scalar_preference() -> Fallible<()> {
        let laplace: Measurement<_, _, MultiDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L1Distance::default(),
            MultiDP::new(AccountingPreference::Zcdp),
            1.0,
            None,
        )?;
        assert_eq!(
            laplace.output_measure.preference(),
            AccountingPreference::Zcdp
        );
        assert_eq!(laplace.map(&rbig!(1))?.epsilon(0.0)?, 1.0);

        let gaussian: Measurement<_, _, MultiDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L2Distance::default(),
            MultiDP::new(AccountingPreference::PureDp),
            1.0,
            None,
        )?;
        assert_eq!(
            gaussian.output_measure.preference(),
            AccountingPreference::PureDp
        );
        assert_eq!(gaussian.map(&rbig!(1))?.epsilon(0.0)?, f64::INFINITY);

        assert!(
            make_noise(
                AtomDomain::<f64>::new_non_nan(),
                crate::metrics::AbsoluteDistance::<f64>::default(),
                MultiDP::default(),
                1.0,
                None,
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn test_make_noise_multidp_preference() -> Fallible<()> {
        use crate::metrics::AbsoluteDistance;
        for (preference, expected) in [
            (AccountingPreference::PureDp, true),
            (AccountingPreference::Zcdp, false),
        ] {
            let measurement = make_noise(
                AtomDomain::<f64>::new_non_nan(),
                AbsoluteDistance::<f64>::default(),
                MultiDP::new(preference),
                1.0,
                None,
            )?;
            assert_eq!(measurement.output_measure.preference(), preference);
            assert_eq!(measurement.map(&1.0)?.epsilon(0.0)?.is_finite(), expected);
        }
        Ok(())
    }

    #[test]
    fn test_make_noise_zcdp_selects_gaussian() -> Fallible<()> {
        let measurement: Measurement<_, _, ZeroConcentratedDivergence, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L2Distance::default(),
            ZeroConcentratedDivergence,
            1.0,
            None,
        )?;
        let guarantee: f64 = measurement.map(&rbig!(1))?;
        assert!(guarantee.is_finite());
        Ok(())
    }
}

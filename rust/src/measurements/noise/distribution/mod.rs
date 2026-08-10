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
        k(default = b"null"),
        privacy_measure(c_type = "AnyMeasure *", rust_type = b"null")
    ),
    generics(DI(suppress), MI(suppress), MSelect(suppress))
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
/// * `privacy_measure` - Privacy measure selecting the noise mechanism. `PureDP` selects discrete Laplace; `zCDP` selects discrete Gaussian.
/// * `scale` - Noise scale parameter.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Domain of the data to be released. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `MSelect` - Privacy measure selecting the noise mechanism. `PureDP` selects discrete Laplace; `zCDP` selects discrete Gaussian.
pub fn make_noise<DI: Domain, MI: Metric, MSelect: NoiseMeasure>(
    input_domain: DI,
    input_metric: MI,
    privacy_measure: MSelect,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MultiDP, DI::Carrier>>
where
    MSelect::Distribution: MakeNoise<DI, MI, MultiDP>,
    (DI, MI): MetricSpace,
{
    privacy_measure
        .new_distribution(scale, k)
        .make_noise((input_domain, input_metric))
}

pub trait NoiseMeasure: Measure + 'static {
    type Distribution;
    fn new_distribution(self, scale: f64, k: Option<i32>) -> Self::Distribution;
}

impl NoiseMeasure for PureDP {
    type Distribution = DiscreteLaplace;

    fn new_distribution(self, scale: f64, k: Option<i32>) -> Self::Distribution {
        DiscreteLaplace { scale, k }
    }
}

impl NoiseMeasure for zCDP {
    type Distribution = DiscreteGaussian;

    fn new_distribution(self, scale: f64, k: Option<i32>) -> Self::Distribution {
        DiscreteGaussian { scale, k }
    }
}

#[cfg(test)]
mod test {
    use dashu::{integer::IBig, rbig};

    use super::*;
    use crate::{
        core::Measurement,
        domains::{AtomDomain, VectorDomain},
        measures::{PrivacyGuarantee, Purity},
        metrics::{L1Distance, L2Distance},
    };

    #[test]
    fn test_make_noise_pure_dp_selects_multidp_laplace() -> Fallible<()> {
        let measurement: Measurement<_, _, MultiDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L1Distance::default(),
            PureDP,
            1.0,
            None,
        )?;
        let capabilities = measurement.output_measure.capabilities();
        assert_eq!(capabilities.profile(), Some(Purity::Pure));
        assert_eq!(capabilities.renyi(), Some(Purity::Pure));
        assert_eq!(capabilities.zcdp(), Some(Purity::Pure));
        assert!(!capabilities.tradeoff());
        assert!(!capabilities.gaussian());

        let guarantee = measurement.map(&rbig!(1))?;
        assert_eq!(guarantee.epsilon(0.0)?, 1.0);
        assert!(guarantee.delta(1.0)?.is_finite());
        Ok(())
    }

    #[test]
    fn test_make_noise_pure_dp_float_selects_multidp_laplace() -> Fallible<()> {
        let measurement: Measurement<_, _, MultiDP, f64> = make_noise(
            AtomDomain::<f64>::new_non_nan(),
            crate::metrics::AbsoluteDistance::<f64>::default(),
            PureDP,
            1.0,
            None,
        )?;
        assert_eq!(measurement.map(&1.0)?.epsilon(0.0)?, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_noise_zcdp_selects_multidp_gaussian() -> Fallible<()> {
        let measurement: Measurement<_, _, MultiDP, Vec<IBig>> = make_noise(
            VectorDomain::new(AtomDomain::<IBig>::default()),
            L2Distance::default(),
            zCDP,
            1.0,
            None,
        )?;
        let capabilities = measurement.output_measure.capabilities();
        assert_eq!(capabilities.profile(), None);
        assert_eq!(capabilities.renyi(), None);
        assert_eq!(capabilities.zcdp(), Some(Purity::Pure));
        assert!(!capabilities.tradeoff());
        assert!(!capabilities.gaussian());

        let guarantee: PrivacyGuarantee = measurement.map(&rbig!(1))?;
        assert!(guarantee.delta(1.0)?.is_finite());
        assert!(guarantee.epsilon(1e-3)?.is_finite());
        Ok(())
    }
}

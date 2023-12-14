use std::convert::TryFrom;

use dashu::{base::Signed, integer::IBig, rational::RBig};
use opendp_derive::bootstrap;

use crate::{
    core::{Measurement, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    traits::{samplers::sample_discrete_laplace, InfCast, SaturatingCast},
};

use super::BaseDiscreteLaplaceDomain;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *")),
    generics(D(suppress))
)]
/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
/// using an efficient algorithm on rational bignums.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
///
/// # Citations
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
///
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `QO` - Data type of the output distance and scale.
pub fn make_base_discrete_laplace_cks20<D, QO>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    D: BaseDiscreteLaplaceDomain,
    D::Atom: crate::traits::Integer + SaturatingCast<IBig>,
    (D, D::InputMetric): MetricSpace,
    QO: crate::traits::Float + InfCast<D::Atom>,
    RBig: TryFrom<QO>,
    IBig: From<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let scale_rational =
        RBig::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Measurement::new(
        input_domain,
        if scale.is_zero() {
            D::new_map_function(move |arg: &D::Atom| Ok(*arg))
        } else {
            D::new_map_function(move |arg: &D::Atom| {
                let arg = IBig::from(*arg);
                let noise = sample_discrete_laplace(scale_rational.clone())?;
                Ok(D::Atom::saturating_cast(arg + noise))
            })
        },
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            let d_in = QO::inf_cast(*d_in)?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if d_in.is_zero() {
                return Ok(QO::zero());
            }
            if scale.is_zero() {
                return Ok(QO::infinity());
            }
            // d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}

/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
/// directly using bignum types from [`rug`].
///
/// Set `D` to change the input data type and input metric:
///
/// | `D`                                | input type     | `D::InputMetric`            |
/// | ---------------------------------- | -------------- | --------------------------- |
/// | `AtomDomain<Integer>` (default)     | `Integer`      | `AbsoluteDistance<Integer>` |
/// | `VectorDomain<AtomDomain<Integer>>` | `Vec<Integer>` | `L1Distance<Integer>`       |
///
/// # Citations
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
///
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<Integer>>` or `AtomDomain<Integer>`
pub fn make_base_discrete_laplace_cks20_rug<D>(
    scale: RBig,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<RBig>>>
where
    D: BaseDiscreteLaplaceDomain<Atom = IBig>,
    (D, D::InputMetric): MetricSpace,
{
    if scale.is_negative() || scale.is_zero() {
        return fallible!(MakeMeasurement, "scale must be positive");
    }

    Measurement::new(
        D::default(),
        D::new_map_function(enclose!(scale, move |arg: &IBig| {
            sample_discrete_laplace(scale.clone()).map(|n| arg + n)
        })),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyMap::new(move |d_in: &IBig| (RBig::from(d_in.clone()) / &scale)),
    )
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{domains::AtomDomain, metrics::AbsoluteDistance};

    // there is a distributional test in the accuracy module

    #[test]
    fn test_make_base_discrete_laplace_cks20() -> Fallible<()> {
        let meas = make_base_discrete_laplace_cks20(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            1e30f64,
        )?;
        println!("{:?}", meas.invoke(&0)?);
        assert!(meas.check(&1, &1e30f64)?);

        let meas = make_base_discrete_laplace_cks20(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            0.,
        )?;
        assert_eq!(meas.invoke(&0)?, 0);
        assert_eq!(meas.map(&0)?, 0.);
        assert_eq!(meas.map(&1)?, f64::INFINITY);

        let meas = make_base_discrete_laplace_cks20(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            f64::MAX,
        )?;
        println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

        Ok(())
    }

    #[test]
    fn test_make_base_discrete_laplace_cks20_rug() -> Fallible<()> {
        let _1e30 = RBig::try_from(1e30f64).unwrap();
        let meas = make_base_discrete_laplace_cks20_rug::<AtomDomain<_>>(_1e30.clone())?;
        println!("{:?}", meas.invoke(&IBig::ZERO)?);
        assert!(meas.check(&IBig::ONE, &_1e30)?);

        assert!(make_base_discrete_laplace_cks20_rug::<AtomDomain<_>>(RBig::ZERO).is_err());

        let f64_max = RBig::try_from(f64::MAX).unwrap();
        let meas = make_base_discrete_laplace_cks20_rug::<AtomDomain<_>>(f64_max)?;
        println!(
            "sample with scale=f64::MAX: {:?}",
            meas.invoke(&IBig::ZERO)?
        );

        Ok(())
    }
}

use std::convert::TryFrom;

use az::{SaturatingAs, SaturatingCast};
use rug::{Complete, Integer, Rational};

use crate::{
    core::{Measurement, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    traits::{InfCast, samplers::sample_discrete_laplace},
};

use super::DiscreteLaplaceDomain;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_base_discrete_laplace_cks20<D, QO>(
    scale: QO,
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<QO>>>
where
    D: DiscreteLaplaceDomain,
    D::Atom: crate::traits::Integer,
    QO: crate::traits::Float + InfCast<D::Atom>,
    Rational: TryFrom<QO>,
    Integer: From<D::Atom> + SaturatingCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let scale_rational =
        Rational::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(move |arg: &D::Atom| {
            let arg = Integer::from(arg.clone());
            let noise = sample_discrete_laplace(scale_rational.clone())?;
            Ok((arg + noise).saturating_as())
        }),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            let d_in = QO::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if d_in.is_zero() {
                return Ok(QO::zero())
            }
            if scale.is_zero() {
                return Ok(QO::infinity());
            }
            // d_in / scale
            d_in.inf_div(&scale)
        }),
    ))
}


pub fn make_base_discrete_laplace_cks20_rug<D>(
    scale: Rational,
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<Rational>>>
where
    D: DiscreteLaplaceDomain<Atom = Integer>,
{
    if scale <= 0 {
        return fallible!(MakeMeasurement, "scale must be positive");
    }

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(enclose!(scale, move |arg: &Integer| {
            sample_discrete_laplace(scale.clone()).map(|n| arg + n)
        })),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyMap::new(move |d_in: &Integer| (d_in / &scale).complete()),
    ))
}

#[cfg(test)]
mod test {
    use num::{One, Zero};

    use super::*;
    use crate::{domains::AllDomain, error::ExplainUnwrap};

    // there is a distributional test in the accuracy module

    #[test]
    fn test_make_base_discrete_laplace_cks20() -> Fallible<()> {
        let meas = make_base_discrete_laplace_cks20::<AllDomain<_>, _>(1e30f64)?;
        println!("{:?}", meas.invoke(&0)?);
        assert!(meas.check(&1, &1e30f64)?);

        let meas = make_base_discrete_laplace_cks20::<AllDomain<_>, _>(0.)?;
        assert_eq!(meas.invoke(&0)?, 0);
        assert_eq!(meas.map(&0)?, 0.);
        assert_eq!(meas.map(&1)?, f64::INFINITY);

        let meas = make_base_discrete_laplace_cks20::<AllDomain<_>, _>(f64::MAX)?;
        println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

        Ok(())
    }

    #[test]
    fn test_make_base_discrete_laplace_cks20_rug() -> Fallible<()> {
        let _1e30 = Rational::try_from(1e30f64).unwrap_test();
        let meas = make_base_discrete_laplace_cks20_rug::<AllDomain<_>>(_1e30.clone())?;
        println!("{:?}", meas.invoke(&Integer::zero())?);
        assert!(meas.check(&Integer::one(), &_1e30)?);

        assert!(make_base_discrete_laplace_cks20_rug::<AllDomain<_>>(Rational::zero()).is_err());
        
        let f64_max = Rational::try_from(f64::MAX).unwrap_test();
        let meas = make_base_discrete_laplace_cks20_rug::<AllDomain<_>>(f64_max)?;
        println!("sample with scale=f64::MAX: {:?}", meas.invoke(&Integer::zero())?);

        Ok(())
    }
}
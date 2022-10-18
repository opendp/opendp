use std::convert::TryFrom;

use az::{SaturatingAs, SaturatingCast};
use num::{traits::Pow, Float as _, Zero};
use opendp_derive::bootstrap;
use rug::{Integer, Rational};

use crate::{
    core::{Measure, Measurement, PrivacyMap, SensitivityMetric},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{samplers::sample_discrete_gaussian, CheckNull, Float, InfCast, Number},
};

#[cfg(feature = "ffi")]
mod ffi;

use super::MappableDomain;

#[doc(hidden)]
pub trait DiscreteGaussianDomain<QI>: MappableDomain + Default {
    type InputMetric: SensitivityMetric<Distance = QI> + Default;
}
impl<T: Clone + CheckNull, QI> DiscreteGaussianDomain<QI> for AllDomain<T> {
    type InputMetric = AbsoluteDistance<QI>;
}
impl<T: Clone + CheckNull, QI> DiscreteGaussianDomain<QI> for VectorDomain<AllDomain<T>> {
    type InputMetric = L2Distance<QI>;
}

#[doc(hidden)]
pub trait DiscreteGaussianMeasure<DI, QI>: Measure + Default
where
    DI: DiscreteGaussianDomain<QI>,
{
    type Atom: Float;
    fn new_forward_map(scale: Self::Atom) -> Fallible<PrivacyMap<DI::InputMetric, Self>>;
}

impl<DI, QI, QO> DiscreteGaussianMeasure<DI, QI> for ZeroConcentratedDivergence<QO>
where
    DI: DiscreteGaussianDomain<QI>,
    QI: Number,
    QO: Float + InfCast<QI>,
    Rational: TryFrom<QO>,
{
    type Atom = QO;

    fn new_forward_map(scale: Self::Atom) -> Fallible<PrivacyMap<DI::InputMetric, Self>> {
        let _2 = QO::exact_int_cast(2)?;

        Ok(PrivacyMap::new_fallible(move |d_in: &QI| {
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

            // (d_in / scale)^2 / 2
            (d_in.inf_div(&scale)?).inf_pow(&_2)?.inf_div(&_2)
        }))
    }
}

#[bootstrap(
    features("contrib"),
    arguments(
        scale(rust_type = "QO", c_type = "void *")),
    generics(
        D(default = "AllDomain<int>"),
        MO(default = "ZeroConcentratedDivergence<QO>", generics = "QO"),
        QI(default = "int")),
    derived_types(QO = "$get_atom_or_infer(MO, scale)")
)]
/// Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.
/// 
/// Set `D` to change the input data type and input metric:
/// 
/// | `D`                          | input type   | `D::InputMetric`        |
/// | ---------------------------- | ------------ | ----------------------- |
/// | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<QI>`  |
/// | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L2Distance<QI>`        |
/// 
/// # Arguments
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k. 
/// 
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`.
/// * `MO` - Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
/// * `QI` - Input distance. The type of sensitivities. Can be any integer or float.
pub fn make_base_discrete_gaussian<D, MO, QI>(
    scale: MO::Atom,
) -> Fallible<Measurement<D, D, D::InputMetric, MO>>
where
    D: DiscreteGaussianDomain<QI>,
    Integer: From<D::Atom> + SaturatingCast<D::Atom>,

    MO: DiscreteGaussianMeasure<D, QI>,
    Rational: TryFrom<MO::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let scale_rational =
        Rational::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Ok(Measurement::new(
        D::default(),
        D::default(),
        if scale.is_zero() {
            D::new_map_function(move |arg: &D::Atom| Ok(arg.clone()))
        } else {
            D::new_map_function(move |arg: &D::Atom| {
                // exact conversion to bignum int
                let arg = Integer::from(arg.clone());
                // exact sampling of noise
                let noise = sample_discrete_gaussian(scale_rational.clone())?;
                // exact addition, and then postprocess by casting to D::Atom
                //     clamp to the data type's bounds if out of range
                Ok((arg + noise).saturating_as())
            })
        },
        D::InputMetric::default(),
        MO::default(),
        MO::new_forward_map(scale)?,
    ))
}

pub fn make_base_discrete_gaussian_rug<D>(
    scale: Rational,
) -> Fallible<Measurement<D, D, D::InputMetric, ZeroConcentratedDivergence<Rational>>>
where
    D: DiscreteGaussianDomain<Rational, Atom = Integer>,
{
    if scale <= 0 {
        return fallible!(MakeMeasurement, "scale must be positive");
    }

    Ok(Measurement::new(
        D::default(),
        D::default(),
        D::new_map_function(enclose!(scale, move |arg: &Integer| {
            sample_discrete_gaussian(scale.clone()).map(|n| arg + n)
        })),
        D::InputMetric::default(),
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new(move |d_in: &Rational| (d_in.clone() / &scale).pow(2) / 2),
    ))
}

#[cfg(test)]
mod test {
    use num::{One, Zero};

    use super::*;
    use crate::{domains::AllDomain, error::ExplainUnwrap};

    // there is a distributional test in the accuracy module

    #[test]
    fn test_make_base_discrete_gaussian() -> Fallible<()> {
        let meas = make_base_discrete_gaussian::<AllDomain<_>, ZeroConcentratedDivergence<_>, f32>(1e30f64)?;
        println!("{:?}", meas.invoke(&0)?);
        assert!(meas.check(&1., &1e30f64.recip().powi(2))?);

        let meas = make_base_discrete_gaussian::<AllDomain<_>, ZeroConcentratedDivergence<_>, i32>(0.)?;
        assert_eq!(meas.invoke(&0)?, 0);
        assert_eq!(meas.map(&0)?, 0.);
        assert_eq!(meas.map(&1)?, f64::INFINITY);

        let meas = make_base_discrete_gaussian::<AllDomain<_>, ZeroConcentratedDivergence<_>, f64>(f64::MAX)?;
        println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

        Ok(())
    }

    #[test]
    fn test_make_base_discrete_gaussian_rug() -> Fallible<()> {
        let _1e30 = Rational::try_from(1e30f64).unwrap_test();
        let meas = make_base_discrete_gaussian_rug::<AllDomain<_>>(_1e30.clone())?;
        println!("{:?}", meas.invoke(&Integer::zero())?);
        assert!(meas.check(&Rational::one(), &_1e30)?);

        assert!(make_base_discrete_gaussian_rug::<AllDomain<_>>(Rational::zero()).is_err());

        let f64_max = Rational::try_from(f64::MAX).unwrap_test();
        let meas = make_base_discrete_gaussian_rug::<AllDomain<_>>(f64_max)?;
        println!(
            "sample with scale=f64::MAX: {:?}",
            meas.invoke(&Integer::zero())?
        );

        Ok(())
    }
}

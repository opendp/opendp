use std::convert::TryFrom;

use dashu::{base::Signed, integer::IBig, rational::RBig};
use num::{Float as _, Zero};
use opendp_derive::bootstrap;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::MappableDomain,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{
        samplers::sample_discrete_gaussian, CheckAtom, Float, InfCast, Number, SaturatingCast,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

#[doc(hidden)]
pub trait BaseDiscreteGaussianDomain<QI>: MappableDomain + Default {
    type InputMetric: Metric<Distance = QI> + Default;
}
impl<T: Clone + CheckAtom, QI> BaseDiscreteGaussianDomain<QI> for AtomDomain<T> {
    type InputMetric = AbsoluteDistance<QI>;
}
impl<T: Clone + CheckAtom, QI> BaseDiscreteGaussianDomain<QI> for VectorDomain<AtomDomain<T>> {
    type InputMetric = L2Distance<QI>;
}

#[doc(hidden)]
pub trait DiscreteGaussianMeasure<DI, QI>: Measure + Default
where
    DI: BaseDiscreteGaussianDomain<QI>,
{
    type Atom: Float;
    fn new_forward_map(scale: Self::Atom) -> Fallible<PrivacyMap<DI::InputMetric, Self>>;
}

impl<DI, QI, QO> DiscreteGaussianMeasure<DI, QI> for ZeroConcentratedDivergence<QO>
where
    DI: BaseDiscreteGaussianDomain<QI>,
    QI: Number,
    QO: Float + InfCast<QI>,
    RBig: TryFrom<QO>,
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
            (d_in.inf_div(&scale)?).inf_powi(2.into())?.inf_div(&_2)
        }))
    }
}

#[bootstrap(
    features("contrib"),
    arguments(scale(rust_type = "QO", c_type = "void *")),
    generics(
        D(suppress),
        MO(default = "ZeroConcentratedDivergence<QO>", generics = "QO"),
        QI(suppress)
    ),
    derived_types(QO = "$get_atom_or_infer(MO, scale)")
)]
/// Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
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
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MO` - Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
/// * `QI` - Input distance. The type of sensitivities. Can be any integer or float.
pub fn make_base_discrete_gaussian<D, MO, QI>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: MO::Atom,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MO>>
where
    D: BaseDiscreteGaussianDomain<QI>,
    (D, D::InputMetric): MetricSpace,
    D::Atom: SaturatingCast<IBig>,
    IBig: From<D::Atom>,

    MO: DiscreteGaussianMeasure<D, QI>,
    RBig: TryFrom<MO::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let scale_rational =
        RBig::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Measurement::new(
        input_domain,
        if scale.is_zero() {
            D::new_map_function(move |arg: &D::Atom| Ok(arg.clone()))
        } else {
            D::new_map_function(move |arg: &D::Atom| {
                // exact conversion to bignum int
                let arg = IBig::from(arg.clone());
                // exact sampling of noise
                let noise = sample_discrete_gaussian(scale_rational.clone())?;
                // exact addition, and then postprocess by casting to D::Atom
                //     clamp to the data type's bounds if out of range
                Ok(D::Atom::saturating_cast(arg + noise))
            })
        },
        input_metric,
        MO::default(),
        MO::new_forward_map(scale)?,
    )
}

pub fn make_base_discrete_gaussian_rug<D>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: RBig,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, ZeroConcentratedDivergence<RBig>>>
where
    D: BaseDiscreteGaussianDomain<RBig, Atom = IBig>,
    (D, D::InputMetric): MetricSpace,
{
    if scale.is_negative() || scale.is_zero() {
        return fallible!(MakeMeasurement, "scale must be positive");
    }

    Measurement::new(
        input_domain,
        D::new_map_function(enclose!(scale, move |arg: &IBig| {
            sample_discrete_gaussian(scale.clone()).map(|n| arg + n)
        })),
        input_metric,
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new(move |d_in: &RBig| (d_in.clone() / &scale).pow(2) / RBig::from(2)),
    )
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::domains::AtomDomain;

    // there is a distributional test in the accuracy module

    #[test]
    fn test_make_base_discrete_gaussian() -> Fallible<()> {
        let meas = make_base_discrete_gaussian::<_, ZeroConcentratedDivergence<_>, f32>(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            1e30f64,
        )?;
        println!("{:?}", meas.invoke(&0)?);
        assert!(meas.check(&1., &1e30f64.recip().powi(2))?);

        let meas = make_base_discrete_gaussian::<_, ZeroConcentratedDivergence<_>, i32>(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            0.,
        )?;
        assert_eq!(meas.invoke(&0)?, 0);
        assert_eq!(meas.map(&0)?, 0.);
        assert_eq!(meas.map(&1)?, f64::INFINITY);

        let meas = make_base_discrete_gaussian::<_, ZeroConcentratedDivergence<_>, f64>(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            f64::MAX,
        )?;
        println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

        Ok(())
    }

    #[test]
    fn test_make_base_discrete_gaussian_rug() -> Fallible<()> {
        let _1e30 = RBig::try_from(1e30f64)?;
        let meas = make_base_discrete_gaussian_rug(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            _1e30.clone(),
        )?;
        println!("{:?}", meas.invoke(&IBig::ZERO)?);
        assert!(meas.check(&RBig::ONE, &_1e30)?);

        assert!(make_base_discrete_gaussian_rug(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            RBig::ZERO
        )
        .is_err());

        let f64_max = RBig::try_from(f64::MAX).unwrap();
        let meas = make_base_discrete_gaussian_rug(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            f64_max,
        )?;
        println!(
            "sample with scale=f64::MAX: {:?}",
            meas.invoke(&IBig::ZERO)?
        );

        Ok(())
    }
}

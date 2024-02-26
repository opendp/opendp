use dashu::rational::RBig;
use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{cartesian, Float, InfCast, Number},
};

#[cfg(feature = "contrib")]
mod float;
#[cfg(feature = "contrib")]
pub use float::*;

#[cfg(feature = "contrib")]
mod integer;
#[cfg(feature = "contrib")]
pub use integer::*;

#[cfg(feature = "ffi")]
mod ffi;

#[doc(hidden)]
pub trait GaussianMeasure<MI>: Measure + Default
where
    MI: Metric,
{
    type Atom;
    fn new_forward_map(scale: Self::Atom, relaxation: Self::Atom)
        -> Fallible<PrivacyMap<MI, Self>>;
}

impl<MI, QO> GaussianMeasure<MI> for ZeroConcentratedDivergence<QO>
where
    MI: Metric,
    MI::Distance: Number,
    QO: Float + InfCast<MI::Distance>,
    RBig: TryFrom<QO>,
{
    type Atom = QO;

    fn new_forward_map(
        scale: Self::Atom,
        relaxation: Self::Atom,
    ) -> Fallible<PrivacyMap<MI, Self>> {
        let _2 = QO::exact_int_cast(2)?;

        Ok(PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let d_in = QO::inf_cast(*d_in)?;

            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }

            // increase d_in by the relaxation
            //   * if float, this will be the worst-case rounding of the discretization
            //   * if integer, this will be zero
            let d_in = d_in.inf_add(&relaxation)?;

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

pub trait GaussianDomain<MO, QI>: Domain
where
    (Self, Self::InputMetric): MetricSpace,
    MO: Measure,
{
    type InputMetric: Metric<Distance = QI>;
    fn make_gaussian(
        input_domain: Self,
        input_metric: Self::InputMetric,
        scale: MO::Distance,
        k: Option<i32>,
    ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MO>>;
}

macro_rules! impl_make_gaussian_float {
    ($($ty:ty)+) => {$(
        impl GaussianDomain<ZeroConcentratedDivergence<$ty>, $ty> for AtomDomain<$ty> {
            type InputMetric = AbsoluteDistance<$ty>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
                k: Option<i32>
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, ZeroConcentratedDivergence<$ty>>>
            {
                make_scalar_float_gaussian(input_domain, input_metric, scale, k)
            }
        }

        impl GaussianDomain<ZeroConcentratedDivergence<$ty>, $ty> for VectorDomain<AtomDomain<$ty>> {
            type InputMetric = L2Distance<$ty>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
                k: Option<i32>
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, ZeroConcentratedDivergence<$ty>>>
            {
                make_vector_float_gaussian(input_domain, input_metric, scale, k)
            }
        }
    )+}
}

impl_make_gaussian_float!(f32 f64);

macro_rules! impl_make_gaussian_int {
    ($T:ty, $QO:ty) => {
        impl<QI: Number> GaussianDomain<ZeroConcentratedDivergence<$QO>, QI> for AtomDomain<$T>
        where
            $QO: InfCast<QI>,
        {
            type InputMetric = AbsoluteDistance<QI>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
                k: Option<i32>,
            ) -> Fallible<
                Measurement<
                    Self,
                    Self::Carrier,
                    Self::InputMetric,
                    ZeroConcentratedDivergence<$QO>,
                >,
            > {
                if k.is_some() {
                    return fallible!(MakeMeasurement, "k is only valid for domains over floats");
                }
                make_scalar_integer_gaussian(input_domain, input_metric, scale)
            }
        }

        impl<QI: Number> GaussianDomain<ZeroConcentratedDivergence<$QO>, QI>
            for VectorDomain<AtomDomain<$T>>
        where
            $QO: InfCast<QI>,
        {
            type InputMetric = L2Distance<QI>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
                k: Option<i32>,
            ) -> Fallible<
                Measurement<
                    Self,
                    Self::Carrier,
                    Self::InputMetric,
                    ZeroConcentratedDivergence<$QO>,
                >,
            > {
                if k.is_some() {
                    return fallible!(MakeMeasurement, "k is only valid for domains over floats");
                }
                make_vector_integer_gaussian(input_domain, input_metric, scale)
            }
        }
    };
}
cartesian! {[i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize], [f32, f64], impl_make_gaussian_int}

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *", rust_type = "QO"), k(default = b"null")),
    generics(
        D(suppress),
        MO(default = "ZeroConcentratedDivergence<QO>", generics = "QO"),
        QI(suppress)
    ),
    derived_types(QO = "$get_atom_or_infer(MO, scale)")
)]
/// Make a asdf Measurement that adds noise from the gaussian(`scale`) distribution to the input.
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
/// * `D` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
/// * `QI` - Input distance. The type of sensitivities. Can be any integer or float.
pub fn make_gaussian<D: GaussianDomain<MO, QI>, MO: 'static + Measure, QI: 'static>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: MO::Distance,
    k: Option<i32>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MO>>
where
    (D, D::InputMetric): MetricSpace,
{
    D::make_gaussian(input_domain, input_metric, scale, k)
}

#[cfg(test)]
mod test {

    use crate::metrics::{AbsoluteDistance, L2Distance};

    use super::*;
    use num::{One, Zero};

    #[test]
    fn test_all() -> Fallible<()> {
        macro_rules! test_gaussian_with_ty {
            ($($ty:ty),+) => {$(
                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(AtomDomain::<$ty>::default(), AbsoluteDistance::<$ty>::default(), 1., None)?;
                meas.invoke(&<$ty>::zero())?;
                meas.map(&<$ty>::one())?;

                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(VectorDomain::new(AtomDomain::<$ty>::default()), L2Distance::<$ty>::default(), 1., None)?;
                meas.invoke(&vec![<$ty>::zero()])?;
                meas.map(&<$ty>::one())?;
            )+}
        }
        test_gaussian_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64);
        Ok(())
    }

    #[test]
    fn test_other_qi() -> Fallible<()> {
        macro_rules! test_gaussian_with_ty {
            ($($ty:ty),+) => {$(
                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(AtomDomain::<$ty>::default(), AbsoluteDistance::<f64>::default(), 1., None)?;
                meas.invoke(&<$ty>::zero())?;
                meas.map(&1.)?;

                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(VectorDomain::new(AtomDomain::<$ty>::default()), L2Distance::<f64>::default(), 1., None)?;
                meas.invoke(&vec![<$ty>::zero()])?;
                meas.map(&1.)?;
            )+}
        }
        test_gaussian_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128);
        Ok(())
    }
}

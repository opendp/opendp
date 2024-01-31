use opendp_derive::bootstrap;

use crate::{
    core::{Measure, Measurement, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{RenyiDivergence, ZeroConcentratedDivergence},
    traits::{cartesian, InfCast, Number},
};

#[cfg(feature = "contrib")]
mod continuous;
#[cfg(feature = "contrib")]
pub use continuous::*;

#[cfg(feature = "contrib")]
mod discrete;
#[cfg(feature = "contrib")]
pub use discrete::*;

#[cfg(feature = "ffi")]
mod ffi;

pub trait MeasureAtom: Measure {
    type Atom;
}

impl<T> MeasureAtom for RenyiDivergence<T> {
    type Atom = T;
}

impl<T> MeasureAtom for ZeroConcentratedDivergence<T> {
    type Atom = T;
}

pub trait MakeGaussian<MO: MeasureAtom, QI>: BaseDiscreteGaussianDomain<QI>
where
    (Self, Self::InputMetric): MetricSpace,
{
    fn make_gaussian(
        input_domain: Self,
        input_metric: Self::InputMetric,
        scale: MO::Atom,
    ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MO>>;
}

macro_rules! impl_make_gaussian_float {
    ($($ty:ty)+) => {$(
        impl MakeGaussian<ZeroConcentratedDivergence<$ty>, $ty> for AtomDomain<$ty> {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, ZeroConcentratedDivergence<$ty>>>
            {
                make_base_gaussian(input_domain, input_metric, scale, None)
            }
        }

        impl MakeGaussian<ZeroConcentratedDivergence<$ty>, $ty> for VectorDomain<AtomDomain<$ty>> {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, ZeroConcentratedDivergence<$ty>>>
            {
                make_base_gaussian(input_domain, input_metric, scale, None)
            }
        }

        impl MakeGaussian<RenyiDivergence<$ty>, $ty> for AtomDomain<$ty> {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, RenyiDivergence<$ty>>>
            {
                make_base_gaussian(input_domain, input_metric, scale, None)
            }
        }

        impl MakeGaussian<RenyiDivergence<$ty>, $ty> for VectorDomain<AtomDomain<$ty>> {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, RenyiDivergence<$ty>>>
            {
                make_base_gaussian(input_domain, input_metric, scale, None)
            }
        }
    )+}
}

impl_make_gaussian_float!(f32 f64);

macro_rules! impl_make_gaussian_int {
    ($T:ty, $QO:ty) => {
        impl<QI: Number> MakeGaussian<ZeroConcentratedDivergence<$QO>, QI> for AtomDomain<$T>
        where
            $QO: InfCast<QI>,
        {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
            ) -> Fallible<
                Measurement<
                    Self,
                    Self::Carrier,
                    Self::InputMetric,
                    ZeroConcentratedDivergence<$QO>,
                >,
            > {
                make_base_discrete_gaussian(input_domain, input_metric, scale)
            }
        }

        impl<QI: Number> MakeGaussian<ZeroConcentratedDivergence<$QO>, QI>
            for VectorDomain<AtomDomain<$T>>
        where
            $QO: InfCast<QI>,
        {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
            ) -> Fallible<
                Measurement<
                    Self,
                    Self::Carrier,
                    Self::InputMetric,
                    ZeroConcentratedDivergence<$QO>,
                >,
            > {
                make_base_discrete_gaussian(input_domain, input_metric, scale)
            }
        }

        impl<QI: Number> MakeGaussian<RenyiDivergence<$QO>, QI> for AtomDomain<$T>
        where
            $QO: InfCast<QI>,
        {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, RenyiDivergence<$QO>>>
            {
                make_base_discrete_gaussian(input_domain, input_metric, scale)
            }
        }

        impl<QI: Number> MakeGaussian<RenyiDivergence<$QO>, QI> for VectorDomain<AtomDomain<$T>>
        where
            $QO: InfCast<QI>,
        {
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, RenyiDivergence<$QO>>>
            {
                make_base_discrete_gaussian(input_domain, input_metric, scale)
            }
        }
    };
}
cartesian! {[i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize], [f32, f64], impl_make_gaussian_int}

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *", rust_type = "$get_atom(MO)")),
    generics(
        D(suppress),
        MO(default = "ZeroConcentratedDivergence<QO>", generics = "QO"),
        QI(suppress)
    ),
    derived_types(QO = "$get_atom_or_infer(MO, scale)")
)]
/// Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
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
pub fn make_gaussian<D: MakeGaussian<MO, QI>, MO: 'static + MeasureAtom, QI: 'static>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: MO::Atom,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MO>>
where
    (D, D::InputMetric): MetricSpace,
{
    D::make_gaussian(input_domain, input_metric, scale)
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
                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(AtomDomain::<$ty>::default(), AbsoluteDistance::<$ty>::default(), 1.)?;
                meas.invoke(&<$ty>::zero())?;
                meas.map(&<$ty>::one())?;

                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(VectorDomain::new(AtomDomain::<$ty>::default()), L2Distance::<$ty>::default(), 1.)?;
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
                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(AtomDomain::<$ty>::default(), AbsoluteDistance::<f64>::default(), 1.)?;
                meas.invoke(&<$ty>::zero())?;
                meas.map(&1.)?;

                let meas = make_gaussian::<_, ZeroConcentratedDivergence<_>, _>(VectorDomain::new(AtomDomain::<$ty>::default()), L2Distance::<f64>::default(), 1.)?;
                meas.invoke(&vec![<$ty>::zero()])?;
                meas.map(&1.)?;
            )+}
        }
        test_gaussian_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128);
        Ok(())
    }
}

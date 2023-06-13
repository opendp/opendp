use opendp_derive::bootstrap;

use crate::{
    core::{Measurement, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    traits::cartesian,
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

pub trait MakeLaplace<QO>: BaseLaplaceDomain
where
    (Self, Self::InputMetric): MetricSpace,
{
    fn make_laplace(
        input_domain: Self,
        input_metric: Self::InputMetric,
        scale: QO,
    ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<QO>>>;
}

macro_rules! impl_make_laplace_float {
    ($($ty:ty)+) => {$(
        impl MakeLaplace<$ty> for AtomDomain<$ty> {
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$ty>>>
            {
                make_base_laplace(input_domain, input_metric, scale, None)
            }
        }

        impl MakeLaplace<$ty> for VectorDomain<AtomDomain<$ty>> {
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$ty>>>
            {
                make_base_laplace(input_domain, input_metric, scale, None)
            }
        }
    )+}
}

impl_make_laplace_float!(f32 f64);

macro_rules! impl_make_laplace_int {
    ($T:ty, $QO:ty) => {
        impl MakeLaplace<$QO> for AtomDomain<$T> {
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$QO>>>
            {
                make_base_discrete_laplace(input_domain, input_metric, scale)
            }
        }

        impl MakeLaplace<$QO> for VectorDomain<AtomDomain<$T>> {
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$QO>>>
            {
                make_base_discrete_laplace(input_domain, input_metric, scale)
            }
        }
    };
}
cartesian! {[i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize], [f32, f64], impl_make_laplace_int}

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *")),
    generics(D(suppress), QO(default = "float"))
)]
pub fn make_laplace<D: MakeLaplace<QO>, QO: 'static>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    (D, D::InputMetric): MetricSpace,
{
    D::make_laplace(input_domain, input_metric, scale)
}

#[cfg(test)]
mod test {

    use super::*;
    use num::{One, Zero};

    #[test]
    fn test_all() -> Fallible<()> {
        macro_rules! test_laplace_with_ty {
            ($($ty:ty),+) => {$(
                let meas = make_laplace(AtomDomain::<$ty>::default(), Default::default(), 1.)?;
                meas.invoke(&<$ty>::zero())?;
                meas.map(&<$ty>::one())?;

                let meas = make_laplace(VectorDomain::new(AtomDomain::<$ty>::default()), Default::default(), 1.)?;
                meas.invoke(&vec![<$ty>::zero()])?;
                meas.map(&<$ty>::one())?;
            )+}
        }
        test_laplace_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64);
        Ok(())
    }
}

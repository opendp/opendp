use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{InfAdd, InfCast, InfDiv, InfPowI, Number},
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

pub trait GaussianMeasure<MI>: Measure + Default
where
    MI: Metric,
{
    fn new_forward_map(scale: f64, relaxation: f64) -> PrivacyMap<MI, Self>;
}

pub(crate) fn gaussian_zcdp_map<QI>(scale: f64, relaxation: f64) -> impl Fn(&QI) -> Fallible<f64>
where
    QI: Clone,
    f64: InfCast<QI>,
{
    move |d_in: &QI| {
        let d_in = f64::inf_cast(d_in.clone())?;

        if d_in.is_sign_negative() {
            return fallible!(InvalidDistance, "sensitivity must be non-negative");
        }

        // increase d_in by the relaxation
        //   * if float, this will be the worst-case rounding of the discretization
        //   * if integer, this will be zero
        let d_in = d_in.inf_add(&relaxation)?;

        if d_in.is_zero() {
            return Ok(0.0);
        }

        if scale.is_zero() {
            return Ok(f64::INFINITY);
        }

        // (d_in / scale)^2 / 2
        d_in.inf_div(&scale)?.inf_powi(2.into())?.inf_div(&2.)
    }
}

impl<MI> GaussianMeasure<MI> for ZeroConcentratedDivergence
where
    MI: Metric,
    MI::Distance: Number,
    f64: InfCast<MI::Distance>,
{
    fn new_forward_map(scale: f64, relaxation: f64) -> PrivacyMap<MI, Self> {
        PrivacyMap::new_fallible(gaussian_zcdp_map(scale, relaxation))
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
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MO>>;
}

macro_rules! impl_make_gaussian_float {
    ($($ty:ty)+) => {$(
        impl GaussianDomain<ZeroConcentratedDivergence, $ty> for AtomDomain<$ty> {
            type InputMetric = AbsoluteDistance<$ty>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: f64,
                k: Option<i32>
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, ZeroConcentratedDivergence>>
            {
                make_scalar_float_gaussian(input_domain, input_metric, scale, k)
            }
        }

        impl GaussianDomain<ZeroConcentratedDivergence, $ty> for VectorDomain<AtomDomain<$ty>> {
            type InputMetric = L2Distance<$ty>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: f64,
                k: Option<i32>
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, ZeroConcentratedDivergence>>
            {
                make_vector_float_gaussian(input_domain, input_metric, scale, k)
            }
        }
    )+}
}

impl_make_gaussian_float!(f32 f64);

macro_rules! impl_make_gaussian_int {
    ($($T:ty)+) => {
        $(impl<QI: Number> GaussianDomain<ZeroConcentratedDivergence, QI> for AtomDomain<$T>
        where
            f64: InfCast<QI>,
        {
            type InputMetric = AbsoluteDistance<QI>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: f64,
                k: Option<i32>,
            ) -> Fallible<
                Measurement<
                    Self,
                    Self::Carrier,
                    Self::InputMetric,
                    ZeroConcentratedDivergence,
                >,
            > {
                if k.is_some() {
                    return fallible!(MakeMeasurement, "k is only valid for domains over floats");
                }
                make_scalar_integer_gaussian(input_domain, input_metric, scale)
            }
        }

        impl<QI: Number> GaussianDomain<ZeroConcentratedDivergence, QI>
            for VectorDomain<AtomDomain<$T>>
        where
            f64: InfCast<QI>,
        {
            type InputMetric = L2Distance<QI>;
            fn make_gaussian(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: f64,
                k: Option<i32>,
            ) -> Fallible<
                Measurement<
                    Self,
                    Self::Carrier,
                    Self::InputMetric,
                    ZeroConcentratedDivergence,
                >,
            > {
                if k.is_some() {
                    return fallible!(MakeMeasurement, "k is only valid for domains over floats");
                }
                make_vector_integer_gaussian(input_domain, input_metric, scale)
            }
        })+
    };
}
impl_make_gaussian_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

#[bootstrap(
    features("contrib"),
    arguments(k(default = b"null")),
    generics(D(suppress), MO(default = "ZeroConcentratedDivergence"), QI(suppress))
)]
/// Make a Measurement that adds noise from the Gaussian(`scale`) distribution to the input.
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
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
/// * `QI` - Input distance. The type of sensitivities. Can be any integer or float.
pub fn make_gaussian<D: GaussianDomain<MO, QI>, MO: 'static + Measure, QI: 'static>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MO>>
where
    (D, D::InputMetric): MetricSpace,
{
    D::make_gaussian(input_domain, input_metric, scale, k)
}

#[cfg(test)]
mod test;

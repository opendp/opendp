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

pub trait GaussianMeasure<MI>: Measure + Default
where
    MI: Metric,
{
    type Atom;
    fn new_forward_map(scale: Self::Atom, relaxation: Self::Atom) -> PrivacyMap<MI, Self>;
}

pub(crate) fn gaussian_zcdp_map<QI, QO>(scale: QO, relaxation: QO) -> impl Fn(&QI) -> Fallible<QO>
where
    QI: Clone,
    QO: Float + InfCast<QI>,
{
    let _2 = QO::exact_int_cast(2).unwrap();
    move |d_in: &QI| {
        let d_in = QO::inf_cast(d_in.clone())?;

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
    }
}

impl<MI, QO> GaussianMeasure<MI> for ZeroConcentratedDivergence<QO>
where
    MI: Metric,
    MI::Distance: Number,
    QO: Float + InfCast<MI::Distance>,
    RBig: TryFrom<QO>,
{
    type Atom = QO;

    fn new_forward_map(scale: Self::Atom, relaxation: Self::Atom) -> PrivacyMap<MI, Self> {
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
cartesian! {[i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize], [f32, f64], impl_make_gaussian_int}

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
mod test;

use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Measurement, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{cartesian, Float, InfCast},
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

pub(crate) fn laplace_puredp_map<QI, QO>(scale: QO, relaxation: QO) -> impl Fn(&QI) -> Fallible<QO>
where
    QI: Clone,
    QO: Float + InfCast<QI>,
{
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

        // d_in / scale
        d_in.inf_div(&scale)
    }
}

pub trait LaplaceDomain<QO>: Domain
where
    (Self, Self::InputMetric): MetricSpace,
{
    type InputMetric: Metric;
    fn make_laplace(
        input_domain: Self,
        input_metric: Self::InputMetric,
        scale: QO,
        k: Option<i32>,
    ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<QO>>>;
}

macro_rules! impl_make_laplace_float {
    ($($ty:ty)+) => {$(
        impl LaplaceDomain<$ty> for AtomDomain<$ty> {
            type InputMetric = AbsoluteDistance<$ty>;
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
                k: Option<i32>,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$ty>>>
            {
                make_scalar_float_laplace(input_domain, input_metric, scale, k)
            }
        }

        impl LaplaceDomain<$ty> for VectorDomain<AtomDomain<$ty>> {
            type InputMetric = L1Distance<$ty>;
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $ty,
                k: Option<i32>,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$ty>>>
            {
                make_vector_float_laplace(input_domain, input_metric, scale, k)
            }
        }
    )+}
}

impl_make_laplace_float!(f32 f64);

macro_rules! impl_make_laplace_int {
    ($T:ty, $QO:ty) => {
        impl LaplaceDomain<$QO> for AtomDomain<$T> {
            type InputMetric = AbsoluteDistance<$T>;
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
                k: Option<i32>,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$QO>>>
            {
                if k.is_some() {
                    return fallible!(MakeMeasurement, "k is only valid for domains over floats");
                }
                make_scalar_integer_laplace(input_domain, input_metric, scale)
            }
        }

        impl LaplaceDomain<$QO> for VectorDomain<AtomDomain<$T>> {
            type InputMetric = L1Distance<$T>;
            fn make_laplace(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
                k: Option<i32>,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$QO>>>
            {
                if k.is_some() {
                    return fallible!(MakeMeasurement, "k is only valid for domains over floats");
                }
                make_vector_integer_laplace(input_domain, input_metric, scale)
            }
        }
    };
}
cartesian! {[i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize], [f32, f64], impl_make_laplace_int}

#[bootstrap(
    features("contrib"),
    arguments(
        scale(c_type = "void *", rust_type = "$get_atom(QO)"),
        k(default = b"null")
    ),
    generics(D(suppress), QO(default = "float"))
)]
/// Make a Measurement that adds noise from the Laplace(`scale`) distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
///
/// Internally, all sampling is done using the discrete Laplace distribution.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `k` - The noise granularity in terms of 2^k, only valid for domains over floats.
///
/// # Generics
/// * `D` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `QO` - Data type of the output distance and scale. `f32` or `f64`.
pub fn make_laplace<D: LaplaceDomain<QO>, QO: 'static>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
    k: Option<i32>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    (D, D::InputMetric): MetricSpace,
{
    D::make_laplace(input_domain, input_metric, scale, k)
}

#[cfg(test)]
mod test;

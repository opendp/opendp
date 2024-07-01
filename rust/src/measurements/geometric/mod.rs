use opendp_derive::bootstrap;

use crate::{
    core::{Measurement, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    traits::cartesian,
};

mod integer;
use integer::{make_scalar_geometric, make_vector_geometric};

use super::LaplaceDomain;

#[cfg(feature = "ffi")]
mod ffi;

pub trait GeometricDomain<QO>: LaplaceDomain<QO>
where
    (Self, Self::InputMetric): MetricSpace,
{
    fn make_geometric(
        input_domain: Self,
        input_metric: Self::InputMetric,
        scale: QO,
        bounds: Option<(
            <Self::InputMetric as Metric>::Distance,
            <Self::InputMetric as Metric>::Distance,
        )>,
    ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<QO>>>;
}

macro_rules! impl_make_geometric_int {
    ($T:ty, $QO:ty) => {
        impl GeometricDomain<$QO> for AtomDomain<$T> {
            fn make_geometric(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
                bounds: Option<(
                    <Self::InputMetric as Metric>::Distance,
                    <Self::InputMetric as Metric>::Distance,
                )>,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$QO>>>
            {
                if bounds.is_some() {
                    make_scalar_geometric(input_domain, input_metric, scale, bounds)
                } else {
                    Self::make_laplace(input_domain, input_metric, scale, None)
                }
            }
        }

        impl GeometricDomain<$QO> for VectorDomain<AtomDomain<$T>> {
            fn make_geometric(
                input_domain: Self,
                input_metric: Self::InputMetric,
                scale: $QO,
                bounds: Option<(
                    <Self::InputMetric as Metric>::Distance,
                    <Self::InputMetric as Metric>::Distance,
                )>,
            ) -> Fallible<Measurement<Self, Self::Carrier, Self::InputMetric, MaxDivergence<$QO>>>
            {
                if bounds.is_some() {
                    make_vector_geometric(input_domain, input_metric, scale, bounds)
                } else {
                    Self::make_laplace(input_domain, input_metric, scale, None)
                }
            }
        }
    };
}
cartesian! {[i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize], [f32, f64], impl_make_geometric_int}

#[bootstrap(
    features("contrib"),
    arguments(
        scale(c_type = "void *"),
        bounds(rust_type = "OptionT", default = b"null")
    ),
    generics(D(suppress)),
    derived_types(
        T = "$get_atom(get_carrier_type(input_domain))",
        OptionT = "Option<(T, T)>"
    )
)]
/// Equivalent to `make_laplace` but restricted to an integer support.
/// Can specify `bounds` to run the algorithm in near constant-time.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
/// * `bounds` - Set bounds on the count to make the algorithm run in constant-time.
///
/// # Arguments
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `QO` - Data type of the scale and output distance
pub fn make_geometric<D: 'static + GeometricDomain<QO>, QO: 'static>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
    bounds: Option<(
        <D::InputMetric as Metric>::Distance,
        <D::InputMetric as Metric>::Distance,
    )>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    (D, D::InputMetric): MetricSpace,
{
    if bounds.is_none() {
        D::make_laplace(input_domain, input_metric, scale, None)
    } else {
        D::make_geometric(input_domain, input_metric, scale, bounds)
    }
}

use num::Float as _;
use opendp_derive::bootstrap;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain, LazyFrameDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L1Distance, L2Distance},
    traits::{samplers::SampleDiscreteGaussianZ2k, CheckAtom, ExactIntCast, Float, FloatBits},
};

use super::{get_discretization_consts, MappableDomain};

//#[cfg(feature = "ffi")]
//mod ffi;

/* #[bootstrap(
    features("contrib"),
    arguments(
        scale(rust_type = "T", c_type = "void *"),
        k(default = -1074, rust_type = "i32", c_type = "uint32_t")),
    generics(
        D(default = "AtomDomain<T>", generics = "T"),
        MO(default = "ZeroConcentratedDivergence<T>", generics = "T")),
    derived_types(T = "$get_atom_or_infer(D, scale)")
)] */

/// Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
///
/// Set `D` to change the input data type and input metric:
///
///
/// | `D`                          | input type   | `D::InputMetric`       |
/// | ---------------------------- | ------------ | ---------------------- |
/// | `AtomDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
/// | `VectorDomain<AtomDomain<T>>` | `Vec<T>`     | `L2Distance<T>`        |
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
pub fn make_dataframe_gaussian<D, MO, T: Float>(
    scale: D::Atom,
    k: Option<i32>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MO>>
where
    D: LazyFrameDomain,
    (D, D::InputMetric): L1Distance<T>,
    MO: GaussianMeasure<D>,
    i32: ExactIntCast<<D::Atom as FloatBits>::Bits>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let (k, relaxation) = get_discretization_consts(k)?;

    Measurement::new(
        D::default(),
        D::new_map_function(move |arg: &D::Atom| {
            D::Atom::sample_discrete_gaussian_Z2k(*arg, scale, k)
        }),
        D::InputMetric::default(),
        MO::default(),
        MO::new_forward_map(scale, relaxation),
    )
}
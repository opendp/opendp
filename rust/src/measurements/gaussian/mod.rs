use num::Float as _;
use opendp_derive::bootstrap;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{samplers::SampleDiscreteGaussianZ2k, CheckAtom, ExactIntCast, Float, FloatBits},
};

use super::{get_discretization_consts, MappableDomain};

#[cfg(feature = "ffi")]
mod ffi;

#[doc(hidden)]
pub trait GaussianDomain: MappableDomain + Default {
    type InputMetric: Metric<Distance = Self::Atom> + Default;
}
impl<T: Clone + CheckAtom> GaussianDomain for AtomDomain<T> {
    type InputMetric = AbsoluteDistance<T>;
}
impl<T: Clone + CheckAtom> GaussianDomain for VectorDomain<AtomDomain<T>> {
    type InputMetric = L2Distance<T>;
}

#[doc(hidden)]
pub trait GaussianMeasure<DI: GaussianDomain>: Measure + Default {
    fn new_forward_map(scale: DI::Atom, relaxation: DI::Atom) -> PrivacyMap<DI::InputMetric, Self>;
}

impl<DI, Q> GaussianMeasure<DI> for ZeroConcentratedDivergence<Q>
where
    DI: GaussianDomain<Atom = Q>,
    Q: Float,
{
    fn new_forward_map(scale: Q, relaxation: Q) -> PrivacyMap<DI::InputMetric, Self> {
        let _2 = Q::one() + Q::one();
        PrivacyMap::new_fallible(move |d_in: &Q| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale.is_zero() {
                return Ok(Q::infinity());
            }

            // d_in is loosened by the size of the granularization
            let d_in = d_in.inf_add(&relaxation)?;

            // (d_in / scale)^2 / 2
            (d_in.inf_div(&scale)?).inf_pow(&_2)?.inf_div(&_2)
        })
    }
}

#[bootstrap(
    features("contrib"),
    arguments(
        scale(rust_type = "T", c_type = "void *"),
        k(default = -1074, rust_type = "i32", c_type = "uint32_t")),
    generics(
        D(suppress),
        MO(default = "ZeroConcentratedDivergence<T>", generics = "T")),
    derived_types(T = "$get_atom_or_infer(get_carrier_type(input_domain), scale)")
)]
/// Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(T)`       |
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
pub fn make_base_gaussian<D, MO>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: D::Atom,
    k: Option<i32>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MO>>
where
    D: GaussianDomain,
    D::Atom: Float + SampleDiscreteGaussianZ2k,
    (D, D::InputMetric): MetricSpace,
    MO: GaussianMeasure<D>,
    i32: ExactIntCast<<D::Atom as FloatBits>::Bits>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let (k, relaxation) = get_discretization_consts(k)?;

    Measurement::new(
        input_domain,
        D::new_map_function(move |arg: &D::Atom| {
            D::Atom::sample_discrete_gaussian_Z2k(*arg, scale, k)
        }),
        input_metric,
        MO::default(),
        MO::new_forward_map(scale, relaxation),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement =
            make_base_gaussian::<_, ZeroConcentratedDivergence<_>>(
                AtomDomain::default(),
                AbsoluteDistance::default(),
                1.0f64, None)?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &0.0050000001)?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement =
            make_base_gaussian::<_, ZeroConcentratedDivergence<_>>(
                VectorDomain::new(AtomDomain::default()),
                L2Distance::default(),
                1.0f64, None)?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.map(&0.1)? <= 0.0050000001);
        Ok(())
    }
}

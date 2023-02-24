#[cfg(feature = "ffi")]
mod ffi;

use num::{Float as _, Zero};
use opendp_derive::bootstrap;

use crate::core::{Measurement, Metric, PrivacyMap};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::measures::MaxDivergence;
use crate::metrics::{AbsoluteDistance, L1Distance};
use crate::traits::samplers::SampleDiscreteLaplaceZ2k;
use crate::traits::{CheckAtom, ExactIntCast, Float, FloatBits, InfAdd, InfDiv};

use super::MappableDomain;

#[doc(hidden)]
pub trait LaplaceDomain: MappableDomain + Default {
    type InputMetric: Metric<Distance = Self::Atom> + Default;
}
impl<T: Clone + CheckAtom> LaplaceDomain for AtomDomain<T> {
    type InputMetric = AbsoluteDistance<T>;
}
impl<T: Clone + CheckAtom> LaplaceDomain for VectorDomain<AtomDomain<T>> {
    type InputMetric = L1Distance<T>;
}

#[bootstrap(
    features("contrib"),
    arguments(
        scale(rust_type = "T", c_type = "void *"),
        k(default = -1074, rust_type = "i32", c_type = "uint32_t")),
    generics(
        D(default = "AtomDomain<T>", generics = "T")),
    derived_types(T = "$get_atom_or_infer(D, scale)")
)]
/// Make a Measurement that adds noise from the laplace(`scale`) distribution to a scalar value.
///
/// Set `D` to change the input data type and input metric:
///
/// | `D`                          | input type   | `D::InputMetric`       |
/// | ---------------------------- | ------------ | ---------------------- |
/// | `AtomDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
/// | `VectorDomain<AtomDomain<T>>` | `Vec<T>`     | `L1Distance<T>`        |
///
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
pub fn make_base_laplace<D>(
    scale: D::Atom,
    k: Option<i32>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<D::Atom>>>
where
    D: LaplaceDomain,
    D::Carrier: 'static,
    D::Atom: Float + SampleDiscreteLaplaceZ2k,
    i32: ExactIntCast<<D::Atom as FloatBits>::Bits>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let (k, relaxation) = get_discretization_consts(k)?;

    Ok(Measurement::new(
        D::default(),
        D::new_map_function(move |arg: &D::Atom| {
            D::Atom::sample_discrete_laplace_Z2k(*arg, scale, k)
        }),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale.is_zero() {
                return Ok(D::Atom::infinity());
            }

            // increase d_in by the worst-case rounding of the discretization
            let d_in = d_in.inf_add(&relaxation)?;

            // d_in / scale
            d_in.inf_div(&scale)
        }),
    ))
}

// proof should show that the return is always a valid (k, relaxation) pairing
pub(crate) fn get_discretization_consts<T>(k: Option<i32>) -> Fallible<(i32, T)>
where
    T: Float,
    i32: ExactIntCast<T::Bits>,
{
    // the discretization may only be as fine as the subnormal ulp
    let k_min =
        -i32::exact_int_cast(T::EXPONENT_BIAS)? - i32::exact_int_cast(T::MANTISSA_BITS)? + 1;
    let k = k.unwrap_or(k_min).max(k_min);

    let _2 = T::exact_int_cast(2)?;

    // input has granularity 2^{k_min} (subnormal float precision)
    let input_gran = _2.neg_inf_pow(&T::exact_int_cast(k_min)?)?;
    // discretization rounds to the nearest 2^k
    let output_gran = _2.inf_pow(&T::exact_int_cast(k)?)?;

    // the worst-case increase in sensitivity due to discretization is
    //     the range, minus the smallest step in the range
    let relaxation = output_gran.inf_sub(&input_gran)?;

    Ok((k, relaxation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{metrics::SymmetricDistance, transformations::make_sized_bounded_mean};

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (make_sized_bounded_mean::<SymmetricDistance, _>(3, (10.0, 12.0))?
            >> make_base_laplace(1.0, None)?)?;
        let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
        Ok(())
    }

    #[test]
    fn test_big_laplace() -> Fallible<()> {
        let chain = make_base_laplace::<AtomDomain<f64>>(f64::MAX, None)?;
        println!("{:?}", chain.invoke(&f64::MAX)?);
        Ok(())
    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<AtomDomain<_>>(1.0, None)?;
        let _ret = measurement.invoke(&0.0)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_laplace::<VectorDomain<_>>(1.0, None)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }
}

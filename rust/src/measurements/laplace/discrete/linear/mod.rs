#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::{
    core::{Measurement, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    traits::{samplers::SampleDiscreteLaplaceLinear, Float, InfCast, Integer},
};

use super::BaseDiscreteLaplaceDomain;

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
/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
/// using a linear-time algorithm on finite data types.
///
/// This algorithm can be executed in constant time if bounds are passed.
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
///
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
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `QO` - Data type of the scale and output distance.
pub fn make_base_discrete_laplace_linear<D, QO>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
    bounds: Option<(D::Atom, D::Atom)>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    D: BaseDiscreteLaplaceDomain,
    (D, D::InputMetric): MetricSpace,
    D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
    QO: Float + InfCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    if bounds
        .as_ref()
        .map(|(lower, upper)| lower > upper)
        .unwrap_or(false)
    {
        return fallible!(MakeMeasurement, "lower may not be greater than upper");
    }

    Measurement::new(
        input_domain,
        D::new_map_function(move |v: &D::Atom| {
            D::Atom::sample_discrete_laplace_linear(*v, scale, bounds)
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            let d_in = QO::inf_cast(*d_in)?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if d_in.is_zero() {
                return Ok(QO::zero());
            }
            if scale.is_zero() {
                return Ok(QO::infinity());
            }
            // d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}

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
/// An alias for `make_base_discrete_laplace_linear`.
/// If you don't need timing side-channel protections via `bounds`,
/// `make_base_discrete_laplace` is more efficient.
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
pub fn make_base_geometric<D, QO>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
    bounds: Option<(D::Atom, D::Atom)>,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    D: BaseDiscreteLaplaceDomain,
    (D, D::InputMetric): MetricSpace,
    D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
    QO: Float + InfCast<D::Atom>,
{
    make_base_discrete_laplace_linear(input_domain, input_metric, scale, bounds)
}

#[cfg(test)]
mod tests {
    use crate::{
        domains::{AtomDomain, VectorDomain},
        metrics::AbsoluteDistance,
    };

    use super::*;

    #[test]
    fn test_make_discrete_laplace_mechanism_bounded() -> Fallible<()> {
        let measurement = make_base_discrete_laplace_linear(
            AtomDomain::<i32>::default(),
            AbsoluteDistance::<i32>::default(),
            10.0,
            Some((200, 210)),
        )?;
        let arg = 205;
        let _ret = measurement.invoke(&arg)?;
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_discrete_laplace_mechanism_bounded() -> Fallible<()> {
        let measurement = make_base_discrete_laplace_linear(
            VectorDomain::new(AtomDomain::default()),
            Default::default(),
            10.0,
            Some((200, 210)),
        )?;
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg)?;
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5)?);
        Ok(())
    }

    #[test]
    fn test_make_discrete_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace_linear(
            AtomDomain::default(),
            Default::default(),
            10.0,
            None,
        )?;
        let arg = 205;
        let _ret = measurement.invoke(&arg)?;
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_discrete_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace_linear(
            VectorDomain::new(AtomDomain::default()),
            Default::default(),
            10.0,
            None,
        )?;
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg)?;
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5)?);
        Ok(())
    }
}

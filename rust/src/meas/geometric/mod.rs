#[cfg(feature = "ffi")]
mod ffi;

use crate::core::{Domain, Function, Measurement, PrivacyMap, SensitivityMetric};
use crate::domains::{AllDomain, VectorDomain};
use crate::error::*;
use crate::measures::MaxDivergence;
use crate::metrics::{AbsoluteDistance, L1Distance, Modular};
use crate::traits::samplers::{SampleTwoSidedGeometric, Tail};
use crate::traits::{Float, InfCast, Integer};

pub trait GeometricDomain<P>: Domain {
    type InputMetric: SensitivityMetric<Distance = Self::Atom> + Default;
    // Atom is an alias for Self::InputMetric::Distance.
    // It would be possible to fill this with associated type defaults: https://github.com/rust-lang/rust/issues/29661
    type Atom;
    fn new() -> Self;
    fn noise_function(scale: P, tail: Tail<Self::Atom>) -> Function<Self, Self>;
}

impl<T, P> GeometricDomain<P> for AllDomain<T>
where
    T: 'static + Integer + SampleTwoSidedGeometric<P>,
    P: 'static + Float,
{
    type InputMetric = AbsoluteDistance<T>;
    type Atom = T;

    fn new() -> Self {
        AllDomain::new()
    }
    fn noise_function(scale: P, tail: Tail<T>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            T::sample_two_sided_geometric(*arg, scale, tail)
        })
    }
}

impl<T, P> GeometricDomain<P> for VectorDomain<AllDomain<T>>
where
    T: 'static + Integer + SampleTwoSidedGeometric<P>,
    P: 'static + Float,
{
    type InputMetric = L1Distance<T>;
    type Atom = T;

    fn new() -> Self {
        VectorDomain::new_all()
    }
    fn noise_function(scale: P, tail: Tail<T>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            arg.iter()
                .map(|v| T::sample_two_sided_geometric(*v, scale, tail))
                .collect()
        })
    }
}

pub fn make_base_geometric<D, QO>(
    scale: QO,
    bounds: Option<(D::Atom, D::Atom)>,
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<QO>>>
where
    D: GeometricDomain<QO>,
    D::Atom: Integer,
    QO: Float + InfCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    if (bounds.as_ref())
        .map(|(lower, upper)| lower > upper)
        .unwrap_or(false)
    {
        return fallible!(MakeMeasurement, "lower may not be greater than upper");
    }

    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale, Tail::Censored(bounds)),
        D::InputMetric::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            let d_in = QO::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale.is_zero() {
                return Ok(QO::infinity());
            }
            // d_in / scale
            d_in.inf_div(&scale)
        }),
    ))
}

pub fn make_base_modular_geometric<D, QO>(
    scale: QO
) -> Fallible<Measurement<D, D, Modular<D::InputMetric>, MaxDivergence<QO>>>
where
    D: GeometricDomain<QO>,
    D::Atom: Integer,
    QO: Float + InfCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(scale, Tail::Modular),
        Modular::new(D::InputMetric::default()),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            let d_in = QO::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale.is_zero() {
                return Ok(QO::infinity());
            }
            // d_in / scale
            d_in.inf_div(&scale)
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_geometric_mechanism_bounded() {
        let measurement =
            make_base_geometric::<AllDomain<_>, f64>(10.0, Some((200, 210))).unwrap_test();
        let arg = 205;
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }

    #[test]
    fn test_make_vector_geometric_mechanism_bounded() {
        let measurement =
            make_base_geometric::<VectorDomain<_>, f64>(10.0, Some((200, 210))).unwrap_test();
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }

    #[test]
    fn test_make_geometric_mechanism() {
        let measurement = make_base_geometric::<AllDomain<_>, f64>(10.0, None).unwrap_test();
        let arg = 205;
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }

    #[test]
    fn test_make_vector_geometric_mechanism() {
        let measurement = make_base_geometric::<VectorDomain<_>, f64>(10.0, None).unwrap_test();
        let arg = vec![1, 2, 3, 4];
        let _ret = measurement.invoke(&arg).unwrap_test();
        println!("{:?}", _ret);

        assert!(measurement.check(&1, &0.5).unwrap_test());
    }
}

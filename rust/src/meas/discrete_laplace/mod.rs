use crate::{
    core::{Domain, Measurement, SensitivityMetric, Function},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{CheckNull, Float, InfCast, Integer},
    traits::samplers::SampleDiscreteLaplaceLinear
};

#[cfg(feature="use-mpfr")]
use az::SaturatingCast;

#[cfg(feature="ffi")]
mod ffi;

#[cfg(feature="use-mpfr")]
mod cks20;
#[cfg(feature="use-mpfr")]
pub use cks20::*;

mod linear;
pub use linear::*;

pub trait MappableDomain: Domain {
    type Atom;
    fn map_over(
        arg: &Self::Carrier,
        func: &impl Fn(&Self::Atom) -> Fallible<Self::Atom>,
    ) -> Fallible<Self::Carrier>;

    fn new_map_function(func: impl Fn(&Self::Atom) -> Fallible<Self::Atom> + 'static) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| Self::map_over(arg, &func))
    }
}

impl<T: CheckNull> MappableDomain for AllDomain<T> {
    type Atom = T;
    fn map_over(
        arg: &Self::Carrier,
        func: &impl Fn(&Self::Atom) -> Fallible<Self::Atom>,
    ) -> Fallible<Self::Carrier> {
        (func)(arg)
    }
}
impl<D: MappableDomain> MappableDomain for VectorDomain<D> {
    type Atom = D::Atom;
    fn map_over(
        arg: &Vec<D::Carrier>,
        func: &impl Fn(&Self::Atom) -> Fallible<Self::Atom>,
    ) -> Fallible<Self::Carrier> {
        arg.iter().map(|v| D::map_over(v, func)).collect()
    }
}

pub trait DiscreteLaplaceDomain: MappableDomain + Default {
    type InputMetric: SensitivityMetric<Distance = Self::Atom> + Default;
}
impl<T: CheckNull> DiscreteLaplaceDomain for AllDomain<T> {
    type InputMetric = AbsoluteDistance<T>;
}
impl<T: CheckNull> DiscreteLaplaceDomain for VectorDomain<AllDomain<T>> {
    type InputMetric = L1Distance<T>;
}

#[cfg(feature="use-mpfr")]
pub fn make_base_discrete_laplace<D, QO>(
    scale: QO,
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<QO>>>
where
    D: DiscreteLaplaceDomain,
    D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
    QO: Float + InfCast<D::Atom> + InfCast<D::Atom>,
    rug::Rational: std::convert::TryFrom<QO>,
    rug::Integer: From<D::Atom> + SaturatingCast<D::Atom>,
{
    if scale > QO::exact_int_cast(3)? {
        make_base_discrete_laplace_cks20(scale)
    } else {
        make_base_discrete_laplace_linear(scale, None)
    }
}

#[cfg(not(feature="use-mpfr"))]
pub fn make_base_discrete_laplace<D, QO>(
    scale: QO,
) -> Fallible<Measurement<D, D, D::InputMetric, MaxDivergence<QO>>>
where
    D: DiscreteLaplaceDomain,
    D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
    QO: Float + InfCast<D::Atom>,
{
    make_base_discrete_laplace_linear(scale, None)
}

use crate::{
    core::{Domain, Function, Measurement, SensitivityMetric},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::samplers::SampleDiscreteLaplaceLinear,
    traits::{CheckNull, Float, InfCast, Integer},
};

#[cfg(feature = "use-mpfr")]
use az::SaturatingCast;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "use-mpfr")]
mod cks20;
#[cfg(feature = "use-mpfr")]
pub use cks20::*;

mod linear;
pub use linear::*;

pub trait MappableDomain: Domain {
    type Atom;
    fn map_over(
        arg: &Self::Carrier,
        func: &impl Fn(&Self::Atom) -> Fallible<Self::Atom>,
    ) -> Fallible<Self::Carrier>;

    fn new_map_function(
        func: impl Fn(&Self::Atom) -> Fallible<Self::Atom> + 'static,
    ) -> Function<Self, Self> {
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

#[cfg(feature = "use-mpfr")]
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
    // benchmarking results at different levels of σ
    // src in /rust/benches/discrete_laplace/main.rs

    // execute bench via:
    //     cargo bench --bench discrete_laplace --features untrusted

    // σ  linear cks20
    // 1   4.907  9.176
    // 2   5.614 10.565
    // 3   6.585 11.592
    // 4   7.450 10.742
    // 5   8.320 12.364
    // 6   9.213 11.722
    // 7  10.106 11.216
    // 8  11.061 10.737
    // 9  11.836 12.884
    // 10 12.653 12.482
    // 11 13.605 12.248
    // 12 14.465 13.320
    // 13 16.545 11.767
    // 14 31.647 15.229
    // 15 25.852 15.177
    // 16 20.179 11.465
    // 17 19.120 13.478
    // 18 19.768 12.982
    // 19 20.777 12.977
    if scale > QO::exact_int_cast(10)? {
        make_base_discrete_laplace_cks20(scale)
    } else {
        make_base_discrete_laplace_linear(scale, None)
    }
}

#[cfg(not(feature = "use-mpfr"))]
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

use crate::{
    core::{Domain, Function, Measurement1, Metric},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::samplers::SampleDiscreteLaplaceLinear,
    traits::{CheckNull, Float, InfCast, Integer},
};

use opendp_derive::bootstrap;

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

#[doc(hidden)]
pub trait MappableDomain: Domain where Self::Carrier: Sized {
    type Atom: Clone;
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

impl<T: 'static + Clone + CheckNull> MappableDomain for AllDomain<T> {
    type Atom = T;
    fn map_over(
        arg: &Self::Carrier,
        func: &impl Fn(&Self::Atom) -> Fallible<Self::Atom>,
    ) -> Fallible<Self::Carrier> {
        (func)(arg)
    }
}
impl<D: MappableDomain> MappableDomain for VectorDomain<D> where D::Carrier: Sized {
    type Atom = D::Atom;
    fn map_over(
        arg: &Vec<D::Carrier>,
        func: &impl Fn(&Self::Atom) -> Fallible<Self::Atom>,
    ) -> Fallible<Self::Carrier> {
        arg.iter().map(|v| D::map_over(v, func)).collect()
    }
}

#[doc(hidden)]
pub trait DiscreteLaplaceDomain: MappableDomain + Default where Self::Carrier: Sized {
    type InputMetric: Metric<Distance = Self::Atom> + Default;
}
impl<T: 'static + Clone + CheckNull> DiscreteLaplaceDomain for AllDomain<T> {
    type InputMetric = AbsoluteDistance<T>;
}
impl<T: 'static + Clone + CheckNull> DiscreteLaplaceDomain for VectorDomain<AllDomain<T>> {
    type InputMetric = L1Distance<T>;
}

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *")),
    generics(D(default = "AllDomain<int>"))
)]
/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input.
/// 
/// Set `D` to change the input data type and input metric:
/// 
/// | `D`                          | input type   | `D::InputMetric`       |
/// | ---------------------------- | ------------ | ---------------------- |
/// | `AllDomain<T>` (default)     | `T`          | `AbsoluteDistance<T>`  |
/// | `VectorDomain<AllDomain<T>>` | `Vec<T>`     | `L1Distance<T>`        |
/// 
/// This uses `make_base_discrete_laplace_cks20` if scale is greater than 10, otherwise it uses `make_base_discrete_laplace_linear`.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
/// 
/// # Arguments
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
/// 
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AllDomain<T>>` or `AllDomain<T>`
/// * `QO` - Data type of the output distance and scale. `f32` or `f64`.
#[cfg(feature = "use-mpfr")]
pub fn make_base_discrete_laplace<D, QO>(
    scale: QO,
) -> Fallible<Measurement1<D, D, D::InputMetric, MaxDivergence<QO>>>
where
    D: 'static + DiscreteLaplaceDomain,
    D::Carrier: Sized,
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
) -> Fallible<Measurement1<D, D, D::InputMetric, MaxDivergence<QO>>>
where
    D: 'static + DiscreteLaplaceDomain,
    D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
    QO: Float + InfCast<D::Atom>,
{
    make_base_discrete_laplace_linear(scale, None)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domains::AllDomain;

    // there is a distributional test in the accuracy module

    #[test]
    fn test_make_base_discrete_laplace() -> Fallible<()> {
        let meas = make_base_discrete_laplace::<AllDomain<_>, _>(1f64)?;
        println!("{:?}", meas.invoke1(&0)?);
        assert!(meas.check(&1, &1.)?);
        Ok(())
    }
}
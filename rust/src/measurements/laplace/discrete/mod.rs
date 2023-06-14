use crate::{
    core::{Domain, Function, Measurement, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{samplers::SampleDiscreteLaplaceLinear, CheckAtom},
    traits::{Float, InfCast, Integer},
};

#[cfg(feature = "use-mpfr")]
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
pub trait MappableDomain: Domain {
    type Atom: Clone;
    fn map_over(
        arg: &Self::Carrier,
        func: &impl Fn(&Self::Atom) -> Fallible<Self::Atom>,
    ) -> Fallible<Self::Carrier>;

    fn new_map_function(
        func: impl Fn(&Self::Atom) -> Fallible<Self::Atom> + 'static,
    ) -> Function<Self::Carrier, Self::Carrier> {
        Function::new_fallible(move |arg: &Self::Carrier| Self::map_over(arg, &func))
    }
}

impl<T: Clone + CheckAtom> MappableDomain for AtomDomain<T> {
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

#[doc(hidden)]
pub trait BaseDiscreteLaplaceDomain: MappableDomain + Default {
    type InputMetric: Metric<Distance = Self::Atom> + Default;
}
impl<T: Clone + CheckAtom> BaseDiscreteLaplaceDomain for AtomDomain<T> {
    type InputMetric = AbsoluteDistance<T>;
}
impl<T: Clone + CheckAtom> BaseDiscreteLaplaceDomain for VectorDomain<AtomDomain<T>> {
    type InputMetric = L1Distance<T>;
}

#[bootstrap(
    features("contrib"),
    arguments(scale(c_type = "void *")),
    generics(D(suppress))
)]
/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
///
/// This uses `make_base_discrete_laplace_cks20` if scale is greater than 10, otherwise it uses `make_base_discrete_laplace_linear`.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
///
/// # Generics
/// * `QO` - Data type of the output distance and scale. `f32` or `f64`.
#[cfg(feature = "use-mpfr")]
pub fn make_base_discrete_laplace<D, QO>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    D: BaseDiscreteLaplaceDomain,
    D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
    (D, D::InputMetric): MetricSpace,
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
        make_base_discrete_laplace_cks20(input_domain, input_metric, scale)
    } else {
        make_base_discrete_laplace_linear(input_domain, input_metric, scale, None)
    }
}

#[cfg(not(feature = "use-mpfr"))]
pub fn make_base_discrete_laplace<D, QO>(
    input_domain: D,
    input_metric: D::InputMetric,
    scale: QO,
) -> Fallible<Measurement<D, D::Carrier, D::InputMetric, MaxDivergence<QO>>>
where
    D: BaseDiscreteLaplaceDomain,
    (D, D::InputMetric): MetricSpace,
    D::Atom: Integer + SampleDiscreteLaplaceLinear<QO>,
    QO: Float + InfCast<D::Atom>,
{
    make_base_discrete_laplace_linear(input_domain, input_metric, scale, None)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domains::AtomDomain;

    // there is a distributional test in the accuracy module

    #[test]
    fn test_make_base_discrete_laplace() -> Fallible<()> {
        let meas = make_base_discrete_laplace(AtomDomain::default(), Default::default(), 1f64)?;
        println!("{:?}", meas.invoke(&0)?);
        assert!(meas.check(&1, &1.)?);
        Ok(())
    }
}

use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::MakeNoise,
    measures::{PureDP, zCDP},
    traits::CheckAtom,
};

use opendp_derive::bootstrap;

mod gaussian;
pub use gaussian::*;

mod geometric;
pub use geometric::*;

mod laplace;
pub use laplace::*;

#[cfg(feature = "ffi")]
mod ffi;

pub trait NoiseDomain: Domain {
    type Atom: 'static;
}

impl<T: 'static + CheckAtom> NoiseDomain for AtomDomain<T> {
    type Atom = T;
}

impl<T: 'static + CheckAtom> NoiseDomain for VectorDomain<AtomDomain<T>> {
    type Atom = T;
}

#[bootstrap(
    features("contrib"),
    arguments(
        k(default = b"null"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null")
    ),
    generics(DI(suppress), MI(suppress), MO(suppress))
)]
/// Make a Measurement that adds noise from the appropriate distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`          |
/// | ------------------------------- | ------------ | ----------------------- |
/// | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be released.
/// * `input_metric` - Metric of the data type to be released.
/// * `output_measure` - Privacy measure. Either `PureDP` or `zCDP`.
/// * `scale` - Noise scale parameter.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Domain of the data to be released. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `MO` - Output Measure. Either `PureDP` or `zCDP`.
pub fn make_noise<DI: Domain, MI: Metric, MO: NoiseMeasure>(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    MO::Distribution: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    output_measure
        .new_distribution(scale, k)
        .make_noise((input_domain, input_metric))
}

pub trait NoiseMeasure: Measure + 'static {
    type Distribution;
    fn new_distribution(self, scale: f64, k: Option<i32>) -> Self::Distribution;
}

impl NoiseMeasure for PureDP {
    type Distribution = DiscreteLaplace;

    fn new_distribution(self, scale: f64, k: Option<i32>) -> Self::Distribution {
        DiscreteLaplace { scale, k }
    }
}

impl NoiseMeasure for zCDP {
    type Distribution = DiscreteGaussian;

    fn new_distribution(self, scale: f64, k: Option<i32>) -> Self::Distribution {
        DiscreteGaussian { scale, k }
    }
}

use dashu::{integer::IBig, rational::RBig};
use num::FromPrimitive;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily},
    metrics::{AbsoluteDistance, L1Distance},
    traits::{samplers::sample_discrete_laplace_linear, ExactIntCast, Integer, SaturatingCast},
    transformations::{make_vec, then_index_or_default},
};

use super::Laplace;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type = "OptionT", default = b"null")),
    generics(DI(suppress), MI(suppress), MO(default = "MaxDivergence")),
    derived_types(
        T = "$get_atom(get_carrier_type(input_domain))",
        OptionT = "Option<(T, T)>"
    )
)]
/// Equivalent to `make_laplace` but restricted to an integer support.
/// Can specify `bounds` to run the algorithm in near constant-time.
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
/// # Arguments
/// * `DI` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `MI` - Metric used to measure distance between members of the input domain.
pub fn make_geometric<DI: NoiseDomain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    bounds: Option<(DI::Atom, DI::Atom)>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    Laplace: MakeNoise<DI, MI, MO>,
    ConstantTimeGeometric<DI::Atom>: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    let input_space = (input_domain, input_metric);
    if let Some(bounds) = bounds {
        ConstantTimeGeometric { scale, bounds }.make_noise(input_space)
    } else {
        Laplace { scale, k: None }.make_noise(input_space)
    }
}

#[derive(Clone)]
pub struct ConstantTimeGeometric<T> {
    scale: f64,
    bounds: (T, T),
}

// scalar geometric mechanism
impl<MO: 'static + Measure, T: Integer> MakeNoise<AtomDomain<T>, AbsoluteDistance<T>, MO>
    for ConstantTimeGeometric<T>
where
    T: Integer + SaturatingCast<IBig>,
    IBig: From<T>,
    RBig: TryFrom<T>,
    usize: ExactIntCast<T>,
    ConstantTimeGeometric<T>: MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<T>, MO>,
    ZExpFamily<1>: NoisePrivacyMap<L1Distance<RBig>, MO>,
{
    fn make_noise(
        self,
        input_space: (AtomDomain<T>, AbsoluteDistance<T>),
    ) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<T>, MO>> {
        let t_vec = make_vec(input_space)?;
        let m_geom = self.make_noise(t_vec.output_space())?;
        t_vec >> m_geom >> then_index_or_default(0)
    }
}

// vector geometric mechanism
impl<MO: 'static + Measure, T: Integer> MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<T>, MO>
    for ConstantTimeGeometric<T>
where
    T: Integer + SaturatingCast<IBig>,
    IBig: From<T>,
    usize: ExactIntCast<T>,
    RBig: TryFrom<T>,
    ZExpFamily<1>: NoisePrivacyMap<L1Distance<RBig>, MO>,
{
    fn make_noise(
        self,
        (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, L1Distance<T>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L1Distance<T>, MO>> {
        let ConstantTimeGeometric {
            scale,
            bounds: (lower, upper),
        } = self;
        if lower > upper {
            return fallible!(MakeMeasurement, "lower may not be greater than upper");
        }

        let distribution = ZExpFamily {
            scale: RBig::from_f64(scale)
                .ok_or_else(|| err!(MakeTransformation, "scale ({}) must be finite", scale))?,
        };

        let privacy_map = distribution.noise_privacy_map()?;

        Measurement::new(
            input_domain,
            Function::new_fallible(move |arg: &Vec<T>| {
                arg.iter()
                    .map(|v| sample_discrete_laplace_linear::<T, f64>(*v, scale, (lower, upper)))
                    .collect()
            }),
            input_metric,
            MO::default(),
            PrivacyMap::new_fallible(move |d_in: &T| {
                let d_in = RBig::try_from(d_in.clone())
                    .map_err(|_| err!(FailedMap, "d_in ({}) must be finite", d_in))?;
                privacy_map.eval(&d_in)
            }),
        )
    }
}

use dashu::{integer::IBig, rational::RBig};

use crate::{
    core::{Function, Measure, Measurement, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{samplers::sample_discrete_laplace_linear, ExactIntCast, Integer, SaturatingCast},
    transformations::{make_vec, then_index},
};

use super::{
    make_noise, MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily, IntExpFamily, Nature
};

#[cfg(feature = "ffi")]
mod ffi;

#[derive(Clone)]
pub struct ConstantTimeGeometric<T> {
    scale: f64,
    bounds: (T, T),
}

impl<MO: 'static + Measure, T: Integer>
    MakeNoise<AtomDomain<T>, AbsoluteDistance<T>, ConstantTimeGeometric<T>, MO>
    for (
        (AtomDomain<T>, AbsoluteDistance<T>),
        ConstantTimeGeometric<T>,
    )
where
    T: Integer + SaturatingCast<IBig>,
    IBig: From<T>,
    RBig: TryFrom<T>,
    usize: ExactIntCast<T>,
    (
        (VectorDomain<AtomDomain<T>>, L1Distance<T>),
        ConstantTimeGeometric<T>,
    ): MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<T>, ConstantTimeGeometric<T>, MO>,
    ((L1Distance<T>, MO), ConstantTimeGeometric<T>):
        NoisePrivacyMap<L1Distance<T>, MO, ConstantTimeGeometric<T>>,
{
    fn make_noise(self) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<T>, MO>> {
        let (input_space, distribution) = self;
        let t_vec = make_vec(input_space)?;
        let m_geom = (t_vec.output_space(), distribution).make_noise()?;
        t_vec >> m_geom >> then_index(0)
    }
}

impl<MO: 'static + Measure, T: Integer>
    MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<T>, ConstantTimeGeometric<T>, MO>
    for (
        (VectorDomain<AtomDomain<T>>, L1Distance<T>),
        ConstantTimeGeometric<T>,
    )
where
    T: Integer + SaturatingCast<IBig>,
    IBig: From<T>,
    usize: ExactIntCast<T>,
    ((L1Distance<T>, MO), ZExpFamily<2>):
        NoisePrivacyMap<L1Distance<T>, MO, ZExpFamily<2>>,
{
    fn make_noise(
        self,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L1Distance<T>, MO>> {
        let ((input_domain, input_metric), distribution) = self;
        let ConstantTimeGeometric {
            scale,
            bounds: (lower, upper),
        } = distribution;
        if lower > upper {
            return fallible!(MakeMeasurement, "lower may not be greater than upper");
        }

        let distribution = ZExpFamily {
            scale: RBig::try_from(scale)
                .map_err(|_| err!(MakeTransformation, "scale ({}) must be finite", scale))?,
        };

        Measurement::new(
            input_domain,
            Function::new_fallible(move |arg: &Vec<T>| {
                arg.iter()
                    .map(|v| sample_discrete_laplace_linear::<T, f64>(*v, scale, (lower, upper)))
                    .collect()
            }),
            input_metric,
            MO::default(),
            <((L1Distance<T>, MO), ZExpFamily<2>)>::privacy_map(distribution)?,
        )
    }
}

// #[bootstrap(
//     features("contrib"),
//     arguments(bounds(rust_type = "OptionT", default = b"null")),
//     generics(D(suppress)),
//     derived_types(
//         T = "$get_atom(get_carrier_type(input_domain))",
//         OptionT = "Option<(T, T)>"
//     )
// )]
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
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
pub fn make_geometric<DI: NoiseDomain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    bounds: Option<(DI::Atom, DI::Atom)>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    ((DI, MI), IntExpFamily<1>): MakeNoise<DI, MI, IntExpFamily<1>, MO>,
    ((DI, MI), ConstantTimeGeometric<DI::Atom>): MakeNoise<DI, MI, ConstantTimeGeometric<DI::Atom>, MO>,
    (DI, MI): MetricSpace,
    DI::Atom: Nature<1, Dist=IntExpFamily<1>>,
{
    if let Some(bounds) = bounds {
        let distribution = ConstantTimeGeometric { scale, bounds };
        make_noise(input_domain, input_metric, distribution)
    } else {
        let distribution: IntExpFamily<1> = DI::Atom::new_distribution(scale, None)?;
        make_noise(input_domain, input_metric, distribution)
    }
}

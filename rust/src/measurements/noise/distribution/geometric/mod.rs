use dashu::rational::RBig;
use num::FromPrimitive;
use opendp_derive::{bootstrap, proven};

use crate::{
    core::{Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily},
    metrics::{AbsoluteDistance, L1Distance, ModularMetric},
    traits::{ExactIntCast, InfExp, InfSub, Integer, samplers::sample_discrete_laplace_linear},
    transformations::{make_vec, then_index_or_default},
};

use super::DiscreteLaplace;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

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
/// # Generics
/// * `DI` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `MI` - Metric used to measure distance between members of the input domain.
/// * `MO` - Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
pub fn make_geometric<DI: NoiseDomain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    bounds: Option<(DI::Atom, DI::Atom)>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    DiscreteLaplace: MakeNoise<DI, MI, MO>,
    ConstantTimeGeometric<DI::Atom>: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    let input_space = (input_domain, input_metric);
    if let Some(bounds) = bounds {
        ConstantTimeGeometric { scale, bounds }.make_noise(input_space)
    } else {
        DiscreteLaplace { scale, k: None }.make_noise(input_space)
    }
}

pub struct ConstantTimeGeometric<T> {
    scale: f64,
    bounds: (T, T),
}

// scalar geometric mechanism
#[proven(
    proof_path = "measurements/noise/distribution/geometric/MakeNoise_AtomDomain_for_ConstantTimeGeometric.tex"
)]
impl<T, MO> MakeNoise<AtomDomain<T>, AbsoluteDistance<T>, MO> for ConstantTimeGeometric<T>
where
    T: Integer,
    MO: 'static + Measure,
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
#[proven(
    proof_path = "measurements/noise/distribution/geometric/MakeNoise_VectorDomain_for_ConstantTimeGeometric.tex"
)]
impl<T, MO> MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<T>, MO> for ConstantTimeGeometric<T>
where
    T: Integer,
    MO: 'static + Measure,
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
            divisor: None,
        };
        let output_measure = MO::default();

        let privacy_map = distribution
            .noise_privacy_map(&L1Distance::new(input_metric.modular()), &output_measure)?;

        let p = 1f64.neg_inf_sub(&(-scale.recip()).inf_exp()?)?;
        if !(0.0..=1.0).contains(&p) {
            return fallible!(
                MakeMeasurement,
                "p ({p}) must be in (0, 1]. This is likely because the noise scale is so large that conservative arithmetic causes the probability of termination to go negative"
            );
        }

        Measurement::new(
            input_domain,
            Function::new_fallible(move |arg: &Vec<T>| {
                arg.iter()
                    .map(|v| sample_discrete_laplace_linear::<T, f64>(*v, scale, (lower, upper)))
                    .collect()
            }),
            input_metric,
            output_measure,
            PrivacyMap::new_fallible(move |d_in: &T| {
                let d_in = RBig::try_from(d_in.clone())
                    .map_err(|_| err!(FailedMap, "d_in ({d_in}) must be finite"))?;
                privacy_map.eval(&d_in)
            }),
        )
    }
}

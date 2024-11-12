use dashu::{
    integer::{fast_div::ConstDivisor, IBig},
    rational::RBig,
};

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    traits::samplers::{sample_discrete_gaussian, sample_discrete_laplace},
};

pub(crate) mod nature;

mod distribution;
pub use distribution::*;

/// Make a measurement that samples from a random variable `RV`.
pub trait MakeNoise<DI: Domain, MI: Metric, MO: Measure>
where
    (DI, MI): MetricSpace,
{
    /// # Proof Definition
    /// For any choice of arguments to `self`,
    /// returns a valid measurement or error.
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>;
}

/// Create a privacy map for a mechanism that perturbs each element in a vector with a sample from a random variable `RV`.
pub trait NoisePrivacyMap<MI: Metric, MO: Measure>: Sample {
    fn noise_privacy_map(&self, input_metric: &MI) -> Fallible<PrivacyMap<MI, MO>>;
}

pub struct ZExpFamily<const P: usize> {
    pub scale: RBig,
    pub divisor: Option<ConstDivisor>,
}

pub trait SampleDiscreteNoise: 'static + Send + Sync {
    fn sample_discrete_noise(&self) -> Fallible<IBig>;
}
impl SampleDiscreteNoise for ZExpFamily<1> {
    fn sample_discrete_noise(&self) -> Fallible<IBig> {
        sample_discrete_laplace(self.scale.clone())
    }
}
impl SampleDiscreteNoise for ZExpFamily<2> {
    fn sample_discrete_noise(&self) -> Fallible<IBig> {
        sample_discrete_gaussian(self.scale.clone())
    }
}

pub trait Sample: SampleDiscreteNoise {
    fn sample(&self, shift: &IBig) -> Fallible<IBig>;
}

impl<const P: usize> Sample for ZExpFamily<P>
where
    Self: SampleDiscreteNoise,
{
    fn sample(&self, shift: &IBig) -> Fallible<IBig> {
        let mut sample = shift + self.sample_discrete_noise()?;

        if let Some(divisor) = &self.divisor {
            sample %= divisor
        }

        Ok(sample)
    }
}

impl<MI: Metric, MO: 'static + Measure, RV: Sample>
    MakeNoise<VectorDomain<AtomDomain<IBig>>, MI, MO> for RV
where
    (VectorDomain<AtomDomain<IBig>>, MI): MetricSpace,
    RV: NoisePrivacyMap<MI, MO>,
{
    /// Make a Measurement that adds noise from the discrete distribution RV to each value in the input.
    fn make_noise(
        self,
        (input_domain, input_metric): (VectorDomain<AtomDomain<IBig>>, MI),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<IBig>>, Vec<IBig>, MI, MO>> {
        let privacy_map = self.noise_privacy_map(&input_metric)?;
        Measurement::new(
            input_domain,
            Function::new_fallible(move |x: &Vec<IBig>| {
                x.into_iter().map(|x_i| self.sample(x_i)).collect()
            }),
            input_metric,
            MO::default(),
            privacy_map,
        )
    }
}

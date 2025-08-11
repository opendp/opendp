use dashu::{integer::IBig, rational::RBig};
use opendp_derive::proven;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    traits::samplers::{sample_discrete_gaussian, sample_discrete_laplace},
};

#[cfg(test)]
mod test;

pub(crate) mod nature;

mod distribution;
pub use distribution::*;

/// Make a measurement that samples from a random variable `RV`.
pub trait MakeNoise<DI: Domain, MI: Metric, MO: Measure>
where
    (DI, MI): MetricSpace,
{
    /// # Proof Definition
    /// For any choice of `self`, and any `input_space`,
    /// returns a valid measurement or error.
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>;
}

/// Create a privacy map for a mechanism that perturbs each element in a vector with a sample from a random variable `RV`.
pub trait NoisePrivacyMap<MI: Metric, MO: Measure>: Sample {
    /// # Proof Definition
    /// Given a distribution `self`,
    /// returns `Err(e)` if `self` is not a valid distribution.
    /// Otherwise the output is `Ok(privacy_map)`
    /// where `privacy_map` observes the following:
    ///
    /// Define `function(x) = x + Z` where `Z` is a vector of iid samples from `self`.
    ///
    /// For every pair of elements $x, x'$ in `VectorDomain<AtomDomain<IBig>>`,
    /// and for every pair (`d_in`, `d_out`),
    /// where `d_in` has the associated type for `input_metric` and `d_out` has the associated type for `output_measure`,
    /// if $x, x'$ are `d_in`-close under `input_metric`, `privacy_map(d_in)` does not raise an exception,
    /// and `privacy_map(d_in) <= d_out`,
    /// then `function(x)`, `function(x')` are `d_out`-close under `output_measure`.
    fn noise_privacy_map(
        &self,
        input_metric: &MI,
        output_measure: &MO,
    ) -> Fallible<PrivacyMap<MI, MO>>;
}

/// # Proof Definition
/// Scale must be non-negative.
///
/// If scale is zero, represents the constant distribution zero.
///
/// Otherwise, when P is one, represents a discrete laplace random variable.
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{e^{-1/scale} - 1}{e^{-1/scale} + 1} e^{-|x|/scale}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{Z}(0, scale)
/// ```
///
/// Otherwise, when P is two, represents a discrete gaussian random variable.
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{e^{-\frac{x^2}{2\sigma^2}}}{\sum_{y\in\mathbb{Z}}e^{-\frac{y^2}{2\sigma^2}}}, \quad
/// \text{where } X \sim \mathcal{N}_\mathbb{Z}(0, \sigma^2)
/// ```
/// where $\sigma = scale$.
#[derive(Clone)]
pub struct ZExpFamily<const P: usize> {
    pub scale: RBig,
}

pub trait Sample: 'static + Clone + Send + Sync {
    /// # Proof Definition
    /// `self` represents a valid distribution.
    ///
    /// Either returns `Err(e)` independently of the input `shift`,
    /// or `Ok(shift + Z)` where `Z` is a sample from the distribution defined by `self`.
    fn sample(&self, shift: &IBig) -> Fallible<IBig>;
}

#[proven(proof_path = "measurements/noise/Sample_for_ZExpFamily1.tex")]
impl Sample for ZExpFamily<1> {
    fn sample(&self, shift: &IBig) -> Fallible<IBig> {
        Ok(shift + sample_discrete_laplace(self.scale.clone())?)
    }
}

#[proven(proof_path = "measurements/noise/Sample_for_ZExpFamily2.tex")]
impl Sample for ZExpFamily<2> {
    fn sample(&self, shift: &IBig) -> Fallible<IBig> {
        Ok(shift + sample_discrete_gaussian(self.scale.clone())?)
    }
}

#[proven(proof_path = "measurements/noise/MakeNoise_IBig_for_RV.tex")]
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
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<IBig>>, MI, MO, Vec<IBig>>> {
        let distribution = self.clone();
        let output_measure = MO::default();
        let privacy_map = self.noise_privacy_map(&input_metric, &output_measure)?;
        Measurement::new(
            input_domain,
            input_metric,
            output_measure,
            Function::new_fallible(move |x: &Vec<IBig>| {
                x.into_iter().map(|x_i| distribution.sample(x_i)).collect()
            }),
            privacy_map,
        )
    }
}

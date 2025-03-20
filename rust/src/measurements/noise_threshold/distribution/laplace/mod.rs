use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
};
use opendp_derive::bootstrap;

use crate::{
    accuracy::conservative_discrete_laplacian_tail_to_alpha,
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measurements::{
        nature::Nature, Laplace, MakeNoiseThreshold, NoiseDomain, NoiseThresholdPrivacyMap,
        ZExpFamily,
    },
    measures::{Approximate, MaxDivergence},
    metrics::{AbsoluteDistance, PartitionDistance},
    traits::{InfCast, InfPowI, InfSub},
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(threshold(c_type = "void *", rust_type = "TV"), k(default = b"null")),
    generics(DI(suppress), MI(suppress), MO(default = "Approximate<MaxDivergence>")),
    derived_types(TV = "$get_value_type(get_carrier_type(input_domain))")
)]
/// Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric for the input domain.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `threshold` - Exclude counts that are less than this minimum value.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Input Domain.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_laplace_threshold<DI: NoiseDomain, MI: Metric, MO: 'static + Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    threshold: DI::Atom,
    k: Option<i32>,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<1>: MakeNoiseThreshold<DI, MI, MO, Threshold = DI::Atom>,
{
    Laplace { scale, k }.make_noise_threshold((input_domain, input_metric), threshold)
}

impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoiseThreshold<DI, MI, MO> for Laplace
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<1>: MakeNoiseThreshold<DI, MI, MO, Threshold = DI::Atom>,
{
    type Threshold = DI::Atom;
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
    ) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise_threshold(input_space, threshold)
    }
}

impl NoiseThresholdPrivacyMap<PartitionDistance<AbsoluteDistance<UBig>>, Approximate<MaxDivergence>>
    for ZExpFamily<1>
{
    fn noise_threshold_privacy_map(
        self,
        threshold: IBig,
    ) -> Fallible<PrivacyMap<PartitionDistance<AbsoluteDistance<UBig>>, Approximate<MaxDivergence>>>
    {
        let ZExpFamily { scale } = self;
        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
        }
        Ok(PrivacyMap::new_fallible(
            move |(l0, l1, li): &(u32, UBig, UBig)| {
                if l1.is_zero() {
                    return Ok((0.0, 0.0));
                }

                if scale.is_zero() {
                    return Ok((f64::INFINITY, 1.0));
                }

                let epsilon = f64::inf_cast(RBig::from(l1.clone()) / &scale)?;

                if li.as_ibig() >= &threshold {
                    return fallible!(FailedMap, "threshold must be greater than {:?}", li);
                }

                let distance_to_instability = u32::try_from(&threshold - li.as_ibig())?;
                let delta_single = conservative_discrete_laplacian_tail_to_alpha(
                    f64::neg_inf_cast(scale.clone())?,
                    distance_to_instability,
                )?;

                let delta_joint: f64 = (1.0).inf_sub(
                    &(1.0)
                        .neg_inf_sub(&delta_single)?
                        .neg_inf_powi(IBig::from(*l0))?,
                )?;

                // delta is only sensibly at most 1
                Ok((epsilon, delta_joint.min(1.0)))
            },
        ))
    }
}

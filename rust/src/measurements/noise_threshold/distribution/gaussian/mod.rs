use dashu::{
    base::Sign,
    integer::{IBig, UBig},
    rational::RBig,
};
use opendp_derive::{bootstrap, proven};

use crate::{
    accuracy::conservative_discrete_gaussian_tail_to_alpha,
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measurements::{
        DiscreteGaussian, MakeNoiseThreshold, NoiseDomain, NoisePrivacyMap,
        NoiseThresholdPrivacyMap, ZExpFamily, nature::Nature,
    },
    measures::{Approximate, ZeroConcentratedDivergence},
    metrics::{AbsoluteDistance, L2Distance, L02InfDistance},
    traits::{InfPowI, InfSqrt, InfSub},
};

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(threshold(c_type = "void *", rust_type = "TV"), k(default = b"null")),
    generics(
        DI(suppress),
        MI(suppress),
        MO(default = "Approximate<ZeroConcentratedDivergence>")
    ),
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
/// * `threshold` - Exclude pairs with values whose distance from zero exceeds this value.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Input Domain.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_gaussian_threshold<DI: NoiseDomain, MI: Metric, MO: 'static + Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    threshold: DI::Atom,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    DiscreteGaussian: MakeNoiseThreshold<DI, MI, MO, Threshold = DI::Atom>,
    (DI, MI): MetricSpace,
{
    DiscreteGaussian { scale, k }.make_noise_threshold((input_domain, input_metric), threshold)
}

#[proven(
    proof_path = "measurements/noise_threshold/distribution/gaussian/MakeNoiseThreshold_for_DiscreteGaussian.tex"
)]
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoiseThreshold<DI, MI, MO>
    for DiscreteGaussian
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<2>: MakeNoiseThreshold<DI, MI, MO, Threshold = DI::Atom>,
{
    type Threshold = DI::Atom;
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: DI::Atom,
    ) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise_threshold(input_space, threshold)
    }
}

#[proven(
    proof_path = "measurements/noise_threshold/distribution/gaussian/NoiseThresholdPrivacyMap_for_ZExpFamily2.tex"
)]
impl
    NoiseThresholdPrivacyMap<
        L02InfDistance<AbsoluteDistance<RBig>>,
        Approximate<ZeroConcentratedDivergence>,
    > for ZExpFamily<2>
{
    fn noise_threshold_privacy_map(
        &self,
        _input_metric: &L02InfDistance<AbsoluteDistance<RBig>>,
        output_measure: &Approximate<ZeroConcentratedDivergence>,
        threshold: UBig,
    ) -> Fallible<
        PrivacyMap<L02InfDistance<AbsoluteDistance<RBig>>, Approximate<ZeroConcentratedDivergence>>,
    > {
        let noise_privacy_map =
            self.noise_privacy_map(&L2Distance::default(), &output_measure.0)?;
        let ZExpFamily { scale } = self.clone();

        Ok(PrivacyMap::new_fallible(
            move |(l0, l2, li): &(u32, RBig, RBig)| {
                let (Sign::Positive, li) = li.floor().into_parts() else {
                    return fallible!(
                        FailedMap,
                        "l-infinity sensitivity ({li}) must be non-negative"
                    );
                };

                let l2 = (l2.clone()).min(&li * RBig::try_from(f64::from(*l0).inf_sqrt()?)?);
                let li = li.min(l2.floor().into_parts().1);

                if l2.is_zero() {
                    return Ok((0.0, 0.0));
                }

                if scale.is_zero() {
                    return Ok((f64::INFINITY, 1.0));
                }

                let rho = noise_privacy_map.eval(&l2)?;

                if li > threshold {
                    return fallible!(
                        FailedMap,
                        "threshold ({threshold}) must not be smaller than l-infinity sensitivity {li}"
                    );
                }

                let d_instability = &threshold - li;

                let delta_single =
                    conservative_discrete_gaussian_tail_to_alpha(scale.clone(), d_instability)?;

                let delta_joint: f64 = (1.0).inf_sub(
                    &(1.0)
                        .neg_inf_sub(&delta_single)?
                        .neg_inf_powi(IBig::from(*l0))?,
                )?;

                // delta is only sensibly at most 1
                Ok((rho, delta_joint.min(1.0)))
            },
        ))
    }
}

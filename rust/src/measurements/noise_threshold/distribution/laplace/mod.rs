use dashu::{
    base::Sign,
    integer::{IBig, UBig},
    rational::RBig,
};
use opendp_derive::{bootstrap, proven};

use crate::{
    accuracy::{
        conservative_continuous_laplacian_tail_to_alpha,
        conservative_discrete_laplacian_tail_to_alpha,
    },
    core::{Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measurements::{
        DiscreteLaplace, MakeNoiseThreshold, NoiseDomain, NoisePrivacyMap,
        NoiseThresholdPrivacyMap, ZExpFamily, nature::Nature,
    },
    measures::{Approximate, PureDP},
    metrics::{AbsoluteDistance, L1Distance, L01InfDistance},
    traits::{InfPowI, InfSub, option_min},
};

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(threshold(c_type = "void *", rust_type = "TV"), k(default = b"null")),
    generics(DI(suppress), MI(suppress), MO(default = "Approximate<PureDP>")),
    derived_types(TV = "$get_value_type(get_carrier_type(input_domain))")
)]
/// Make a Measurement that uses propose-test-release to release a hashmap of counts.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Citations
/// * [Rogers23 A Unifying Privacy Analysis Framework for Unknown Domain Algorithms in Differential Privacy](https://arxiv.org/abs/2309.09170)
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/abs/2004.00010)
///
/// # Proof Navigation
/// * This constructor delegates to [`MakeNoiseThreshold`] for [`DiscreteLaplace`].
/// * Threshold-specific privacy accounting is implemented by [`NoiseThresholdPrivacyMap`].
/// * See also [`make_noise_threshold`] for the distribution-agnostic entry point.
///
/// # Runtime
/// For an input map with `m` entries, each release performs `O(m)` wrapper work
/// plus one discrete-Laplace draw per entry.
///
/// # Utility
/// If an item's value is separated from the threshold by margin `g`,
/// the probability of thresholding it incorrectly decays as `O(exp(-g / scale))`
/// up to constants from the discrete/continuous Laplace tail bounds.
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
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    DiscreteLaplace: MakeNoiseThreshold<DI, MI, MO, Threshold = DI::Atom>,
    (DI, MI): MetricSpace,
{
    DiscreteLaplace { scale, k }.make_noise_threshold((input_domain, input_metric), threshold)
}

#[proven(
    proof_path = "measurements/noise_threshold/distribution/laplace/MakeNoiseThreshold_for_DiscreteLaplace.tex"
)]
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoiseThreshold<DI, MI, MO>
    for DiscreteLaplace
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
    ) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise_threshold(input_space, threshold)
    }
}

#[proven(
    proof_path = "measurements/noise_threshold/distribution/laplace/NoiseThresholdPrivacyMap_for_ZExpFamily1.tex"
)]
impl NoiseThresholdPrivacyMap<L01InfDistance<AbsoluteDistance<RBig>>, Approximate<PureDP>>
    for ZExpFamily<1>
{
    fn noise_threshold_privacy_map(
        &self,
        _input_metric: &L01InfDistance<AbsoluteDistance<RBig>>,
        output_measure: &Approximate<PureDP>,
        threshold: UBig,
    ) -> Fallible<PrivacyMap<L01InfDistance<AbsoluteDistance<RBig>>, Approximate<PureDP>>> {
        let noise_privacy_map =
            self.noise_privacy_map(&L1Distance::default(), &output_measure.0)?;
        let ZExpFamily { scale } = self.clone();

        Ok(PrivacyMap::new_fallible(
            move |(l0, l1, li): &(u32, RBig, RBig)| {
                let (Sign::Positive, l1) = l1.floor().into_parts() else {
                    return fallible!(FailedMap, "l1 sensitivity ({l1}) must be non-negative");
                };

                let (Sign::Positive, li) = li.floor().into_parts() else {
                    return fallible!(
                        FailedMap,
                        "l-infinity sensitivity ({li}) must be non-negative"
                    );
                };

                let l1 = l1.min(&li * l0);
                let li = li.min(l1.clone());

                if l1.is_zero() {
                    return Ok((0.0, 0.0));
                }

                if scale.is_zero() {
                    return Ok((f64::INFINITY, 1.0));
                }

                let rho = noise_privacy_map.eval(&RBig::from(l1))?;

                if li > threshold {
                    return fallible!(
                        FailedMap,
                        "threshold ({threshold}) must not be smaller than l-infinity sensitivity {li}"
                    );
                }

                let d_instability = &threshold - li;

                let delta_single = option_min(
                    conservative_discrete_laplacian_tail_to_alpha(
                        scale.clone(),
                        d_instability.clone(),
                    )
                    .ok(),
                    conservative_continuous_laplacian_tail_to_alpha(
                        scale.clone(),
                        d_instability.into(),
                    )
                    .ok(),
                )
                .ok_or_else(|| err!(FailedMap, "failed to compute tail bound in privacy map"))?;

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

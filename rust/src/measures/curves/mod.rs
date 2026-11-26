use std::sync::Arc;

use crate::{
    error::Fallible,
    traits::SInterval,
    utilities::search::{Bands, fallible_binary_search},
};

mod approxdp;
#[cfg(feature = "idealized-numerics")]
mod gaussiandp;
#[cfg(all(feature = "ffi", feature = "idealized-numerics"))]
mod gaussiandp_ffi;
pub(crate) mod logspace;
mod profile_to_tradeoff;
mod renyidp;
#[cfg(feature = "ffi")]
mod renyidp_ffi;
#[cfg(feature = "ffi")]
mod zcdp_ffi;

#[cfg(feature = "ffi")]
mod reporting_ffi;
use logspace::{
    check_delta, check_log_delta, delta_to_log_lower_unchecked, delta_to_log_upper_unchecked,
    log_to_delta_upper, logaddexp_upper,
};

#[cfg(test)]
mod test;

type LogDeltaFn = dyn Fn(&[f64]) -> Fallible<Vec<f64>> + Send + Sync;

/// An analytical privacy profile with a conservative log-delta core.
///
/// The total delta bound is `min(1, delta_slack + exp(log_delta(epsilon)))`.
/// Callbacks must be functionally pure, nonincreasing, and conservative; only
/// their pointwise ranges and output lengths are checked. All input batches
/// must be nondecreasing, permitting duplicates. Callbacks receive whole batches,
/// including singleton batches during generic inversion.
#[derive(Clone)]
pub struct PrivacyProfile {
    log_delta: Arc<LogDeltaFn>,
    delta_slack: f64,
}

/// A single independently certified (epsilon, delta)-DP bound.
///
/// Source parameters are retained exactly, without conversion to log space.
/// A zero delta certifies pure DP. Construction validates numerical ranges,
/// not whether the certificate holds for a particular privacy relation.
#[derive(Clone, Debug, PartialEq)]
struct ApproxDPRepr {
    epsilon: f64,
    delta: f64,
}

impl ApproxDPRepr {
    fn new(epsilon: f64, delta: f64) -> Fallible<Self> {
        check_epsilon(epsilon)?;
        if !epsilon.is_finite() {
            return fallible!(FailedMap, "certificate epsilon must be finite");
        }
        check_delta(delta)?;
        Ok(Self { epsilon, delta })
    }

    // The symmetric approximate-DP tradeoff implies, below the certificate
    // endpoint, delta + (1-delta) * (1-exp(epsilon-self.epsilon))
    // / (1+exp(-self.epsilon)). Use expm1 to preserve the endpoint.
    fn delta_interval(&self, epsilon: f64) -> Fallible<SInterval> {
        let delta = SInterval::point(self.delta)?;
        if epsilon >= self.epsilon || self.delta == 1.0 {
            return Ok(delta);
        }

        let one = SInterval::point(1.0)?;
        let difference = (SInterval::point(epsilon)? - SInterval::point(self.epsilon)?)?;
        // For extreme arguments the trivial upper bound avoids exhausting
        // the software exponent range.
        if difference.lower_f64()? < -1024.0 {
            return Ok(one);
        }
        let numerator = -difference.exp_m1()?;
        let denominator = self.denominator()?;
        Ok((delta.clone() + (((one.clone() - delta)? * numerator)? / denominator)?)?.min(one))
    }

    // A lower enclosure of 1+exp(-epsilon) is sufficient for the forward
    // conversion. Beyond this range use the weaker lower bound one.
    fn denominator(&self) -> Fallible<SInterval> {
        let one = SInterval::point(1.0)?;
        if self.epsilon > 1024.0 {
            return Ok(one);
        }
        one + SInterval::point(-self.epsilon)?.exp()?
    }

    fn delta(&self, epsilon: f64) -> Fallible<f64> {
        self.delta_interval(epsilon)?.upper_f64()
    }

    fn log_delta(&self, epsilon: f64) -> Fallible<f64> {
        if epsilon >= self.epsilon {
            return delta_to_log_upper_unchecked(self.delta);
        }
        let delta = self.delta_interval(epsilon)?;
        Ok(delta.ln()?.upper_f64()?.min(0.0))
    }

    fn epsilon(&self, delta: f64) -> Fallible<f64> {
        if delta == 1.0 {
            return Ok(0.0);
        }
        if delta < self.delta {
            return Ok(f64::INFINITY);
        }
        if delta == self.delta {
            return Ok(self.epsilon);
        }
        self.epsilon_from_complement(
            (SInterval::point(1.0)? - SInterval::point(delta)?)?,
        )
    }

    fn epsilon_from_log_delta(&self, log_delta: f64) -> Fallible<f64> {
        if log_delta == 0.0 {
            return Ok(0.0);
        }
        if log_delta < delta_to_log_upper_unchecked(self.delta)? {
            return Ok(f64::INFINITY);
        }
        if log_delta == f64::NEG_INFINITY || log_delta < -1024.0 {
            return Ok(self.epsilon);
        }
        self.epsilon_from_complement(-SInterval::point(log_delta)?.exp_m1()?)
    }

    fn epsilon_from_complement(&self, complement: SInterval) -> Fallible<f64> {
        if self.epsilon == 0.0 {
            return Ok(0.0);
        }
        // Invert the symmetric tradeoff: exp(epsilon-epsilon0)
        // = r - (1-r)*exp(-epsilon0), r=(1-delta)/(1-delta0).
        let one = SInterval::point(1.0)?;
        let ratio = (complement / (one.clone() - SInterval::point(self.delta)?)?)?
            .min(one.clone());
        let exponent = if self.epsilon > 1024.0 {
            // Omitting the nonnegative correction gives an upper bound.
            ratio
        } else {
            let inverse = SInterval::point(-self.epsilon)?.exp()?;
            (ratio.clone() - ((one - ratio)? * inverse.clone())?)?.max(inverse)
        };
        Ok((SInterval::point(self.epsilon)? + exponent.ln()?)?
            .upper_f64()?
            .max(0.0)
            .min(self.epsilon))
    }
}

/// Numeric privacy facts returned by a [`MultiDP`] privacy map.
///
/// Each populated field independently certifies the same privacy relation;
/// the fields are conjunctive, not alternative materializations. Attaching one
/// representation preserves the other. Approximate DP is not eagerly converted
/// into a profile, and profile-specific delta slack remains owned by the profile.
#[derive(Clone, Default)]
pub struct PrivacyGuarantee {
    approx_dp: Option<ApproxDPRepr>,
    profile: Option<PrivacyProfile>,
    renyi: Option<RenyiRepresentation>,
    #[cfg(feature = "idealized-numerics")]
    gaussian: Option<GaussianDPRepresentation>,
    zcdp: Option<ZCDPRepresentation>,
}

impl PrivacyGuarantee {
    /// Construct a vacuous guarantee, certifying no nontrivial privacy bound.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a single approximate-DP certificate, retaining the profile.
    /// Validates the parameters, not whether the certificate holds.
    pub fn with_approx_dp(mut self, epsilon: f64, delta: f64) -> Fallible<Self> {
        self.approx_dp = Some(ApproxDPRepr::new(epsilon, delta)?);
        Ok(self)
    }

    /// Attach a profile representation, retaining the approximate-DP certificate.
    pub fn with_profile(mut self, profile: PrivacyProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Attach an RDP representation, retaining all other representations.
    ///
    /// The callback must return conservative, nonnegative epsilon bounds for
    /// nondecreasing finite orders greater than one, permitting duplicates.
    /// Output lengths must match; optimizers evaluate singleton batches.
    /// `source_delta` retains OpenDP's additive source-delta semantics.
    #[allow(non_snake_case)]
    pub fn with_renyiDP(
        mut self,
        curve: impl Fn(&[f64]) -> Fallible<Vec<f64>> + Send + Sync + 'static,
        source_delta: f64,
    ) -> Fallible<Self> {
        check_delta(source_delta)?;
        self.renyi = Some(RenyiRepresentation {
            curve: Arc::new(curve),
            source_delta,
        });
        Ok(self)
    }

    /// Attach a rho-zCDP representation with additive source-delta semantics.
    #[allow(non_snake_case)]
    pub fn with_zCDP(mut self, rho: f64, source_delta: f64) -> Fallible<Self> {
        check_rho(rho)?;
        check_delta(source_delta)?;
        self.zcdp = Some(ZCDPRepresentation { rho, source_delta });
        Ok(self)
    }

    /// Attach a Gaussian-DP representation with parameter `mu`.
    #[cfg(feature = "idealized-numerics")]
    #[allow(non_snake_case)]
    pub fn with_gaussianDP(mut self, mu: f64) -> Fallible<Self> {
        check_mu(mu)?;
        self.gaussian = Some(GaussianDPRepresentation { mu });
        Ok(self)
    }

    /// Return the tightest delta bound, propagating representation errors.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        Ok(self.delta_batch(&[epsilon])?[0])
    }

    /// Return the tightest epsilon bound, propagating representation errors.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        Ok(self.epsilon_batch(&[delta])?[0])
    }

    /// Return an upward-conservative log-delta bound.
    pub fn log_delta(&self, epsilon: f64) -> Fallible<f64> {
        Ok(self.log_delta_batch(&[epsilon])?[0])
    }

    /// Invert a log-delta budget without narrowing it to ordinary delta.
    pub fn epsilon_from_log_delta(&self, log_delta: f64) -> Fallible<f64> {
        Ok(self.epsilon_from_log_delta_batch(&[log_delta])?[0])
    }

    /// Evaluate a nondecreasing batch of epsilon queries.
    pub fn delta_batch(&self, epsilon: &[f64]) -> Fallible<Vec<f64>> {
        let mut bounds = self.evaluate(
            epsilon,
            check_epsilon,
            |_| 1.0,
            ApproxDPRepr::delta,
            PrivacyProfile::delta,
        )?;
        if let Some(repr) = &self.renyi {
            let values = renyidp::delta_via_renyiDP_batch(repr.curve.as_ref(), repr.source_delta, epsilon)?;
            for (bound, value) in bounds.iter_mut().zip(values) {
                *bound = bound.min(value);
            }
        }
        if let Some(repr) = self.zcdp {
            for (bound, &query) in bounds.iter_mut().zip(epsilon) {
                *bound = bound.min(renyidp::delta_via_zCDP(repr.rho, repr.source_delta, query)?);
            }
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = self.gaussian {
            for (bound, &query) in bounds.iter_mut().zip(epsilon) {
                *bound = bound.min(gaussiandp::delta_via_gaussianDP(repr.mu, query)?);
            }
        }
        Ok(bounds)
    }

    /// Evaluate a nondecreasing batch of delta queries.
    pub fn epsilon_batch(&self, delta: &[f64]) -> Fallible<Vec<f64>> {
        let mut bounds = self.evaluate(
            delta,
            check_delta,
            |d| if d == 1.0 {
                0.0
            } else {
                f64::INFINITY
            },
            ApproxDPRepr::epsilon,
            PrivacyProfile::epsilon,
        )?;
        if let Some(repr) = &self.renyi {
            let values = renyidp::epsilon_via_renyiDP_batch(repr.curve.as_ref(), repr.source_delta, delta)?;
            for (bound, value) in bounds.iter_mut().zip(values) {
                *bound = bound.min(value);
            }
        }
        if let Some(repr) = self.zcdp {
            for (bound, &query) in bounds.iter_mut().zip(delta) {
                *bound = bound.min(renyidp::epsilon_via_zCDP(repr.rho, repr.source_delta, query)?);
            }
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = self.gaussian {
            for (bound, &query) in bounds.iter_mut().zip(delta) {
                *bound = bound.min(invert_decreasing_callback(|e| gaussiandp::delta_via_gaussianDP(repr.mu, e), query)?);
            }
        }
        Ok(bounds)
    }

    /// Evaluate a nondecreasing batch of epsilon queries in log-delta space.
    pub fn log_delta_batch(&self, epsilon: &[f64]) -> Fallible<Vec<f64>> {
        let mut bounds = self.evaluate(
            epsilon,
            check_epsilon,
            |_| 0.0,
            ApproxDPRepr::log_delta,
            PrivacyProfile::log_delta,
        )?;
        if let Some(repr) = &self.renyi {
            let values = repr.as_profile()?.log_delta(epsilon)?;
            for (bound, value) in bounds.iter_mut().zip(values) {
                *bound = bound.min(value);
            }
        }
        if let Some(repr) = self.zcdp {
            for (bound, &query) in bounds.iter_mut().zip(epsilon) {
                *bound = bound.min(delta_to_log_upper_unchecked(renyidp::delta_via_zCDP(repr.rho, repr.source_delta, query)?)?);
            }
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = self.gaussian {
            for (bound, &query) in bounds.iter_mut().zip(epsilon) {
                *bound = bound.min(delta_to_log_upper_unchecked(gaussiandp::delta_via_gaussianDP(repr.mu, query)?)?);
            }
        }
        Ok(bounds)
    }

    /// Invert a nondecreasing batch of log-delta budgets.
    pub fn epsilon_from_log_delta_batch(&self, log_delta: &[f64]) -> Fallible<Vec<f64>> {
        let mut bounds = self.evaluate(
            log_delta,
            check_log_delta,
            |d| if d == 0.0 {
                0.0
            } else {
                f64::INFINITY
            },
            ApproxDPRepr::epsilon_from_log_delta,
            PrivacyProfile::epsilon_from_log_delta,
        )?;
        if let Some(repr) = &self.renyi {
            let values = repr.as_profile()?.epsilon_from_log_delta(log_delta)?;
            for (bound, value) in bounds.iter_mut().zip(values) {
                *bound = bound.min(value);
            }
        }
        if let Some(repr) = self.zcdp {
            for (bound, &query) in bounds.iter_mut().zip(log_delta) {
                *bound = bound.min(renyidp::epsilon_via_zCDP(repr.rho, repr.source_delta, log_to_delta_lower_budget(query)?)?);
            }
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = self.gaussian {
            for (bound, &query) in bounds.iter_mut().zip(log_delta) {
                *bound = bound.min(invert_decreasing_callback(|e| gaussiandp::delta_via_gaussianDP(repr.mu, e), log_to_delta_lower_budget(query)?)?);
            }
        }
        Ok(bounds)
    }

    /// Return the strongest beta lower bounds for nondecreasing alpha inputs.
    /// Every attached representation is evaluated; its errors propagate.
    pub fn beta(&self, alphas: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(alphas, check_alpha)?;
        alphas
            .iter()
            .map(|&alpha| {
                if alpha == 1.0 {
                    return Ok(0.0);
                }
                combine_bounds(
                    [
                        self.approx_dp
                            .as_ref()
                            .map(|repr| {
                                approxdp::beta_via_approx_dp(repr.epsilon, repr.delta, alpha)
                            })
                            .transpose()?,
                        self.profile
                            .as_ref()
                            .map(|profile| profile_to_tradeoff::beta_via_profile(profile, alpha))
                            .transpose()?,
                        self.renyi
                            .as_ref()
                            .map(|repr| {
                                renyidp::beta_via_renyiDP(
                                    repr.curve.clone(),
                                    repr.source_delta,
                                    alpha,
                                )
                            })
                            .transpose()?,
                        self.zcdp
                            .as_ref()
                            .map(|repr| renyidp::beta_via_zCDP(repr.rho, repr.source_delta, alpha))
                            .transpose()?,
                        #[cfg(feature = "idealized-numerics")]
                        self.gaussian
                            .as_ref()
                            .map(|repr| gaussiandp::beta_via_gaussianDP(repr.mu, alpha))
                            .transpose()?,
                    ],
                    f64::max,
                )
            })
            .collect()
    }

    /// Return the strongest left-inverse lower bounds for nondecreasing beta inputs.
    /// Inversion evaluates the source representations at singleton candidates.
    pub fn alpha(&self, betas: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(betas, check_beta)?;
        betas.iter().map(|&beta| self.alpha_single(beta)).collect()
    }

    fn alpha_single(&self, beta: f64) -> Fallible<f64> {
        if beta == 1.0 {
            return Ok(0.0);
        }
        combine_bounds(
            [
                self.approx_dp
                    .as_ref()
                    .map(|repr| {
                        invert_beta_callback(
                            |alpha| approxdp::beta_via_approx_dp(repr.epsilon, repr.delta, alpha),
                            beta,
                        )
                    })
                    .transpose()?,
                self.profile
                    .as_ref()
                    .map(|profile| {
                        invert_beta_callback(
                            |alpha| profile_to_tradeoff::beta_via_profile(profile, alpha),
                            beta,
                        )
                    })
                    .transpose()?,
                self.renyi
                    .as_ref()
                    .map(|repr| {
                        invert_beta_callback(
                            |alpha| {
                                renyidp::beta_via_renyiDP(
                                    repr.curve.clone(),
                                    repr.source_delta,
                                    alpha,
                                )
                            },
                            beta,
                        )
                    })
                    .transpose()?,
                self.zcdp
                    .as_ref()
                    .map(|repr| {
                        invert_beta_callback(
                            |alpha| renyidp::beta_via_zCDP(repr.rho, repr.source_delta, alpha),
                            beta,
                        )
                    })
                    .transpose()?,
                #[cfg(feature = "idealized-numerics")]
                self.gaussian
                    .as_ref()
                    .map(|repr| {
                        invert_beta_callback(
                            |alpha| gaussiandp::beta_via_gaussianDP(repr.mu, alpha),
                            beta,
                        )
                    })
                    .transpose()?,
            ],
            f64::max,
        )
    }

    pub(crate) fn approx_dp(&self) -> Option<(f64, f64)> {
        self.approx_dp
            .as_ref()
            .map(|repr| (repr.epsilon, repr.delta))
    }

    pub(crate) fn profile(&self) -> Option<&PrivacyProfile> {
        self.profile.as_ref()
    }
}

impl std::fmt::Debug for PrivacyGuarantee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("PrivacyGuarantee");
        #[cfg(feature = "idealized-numerics")]
        debug.field("gaussian", &self.gaussian.is_some());
        debug
            .field("approx_dp", &self.approx_dp)
            .field("profile", &self.profile.is_some())
            .field("renyi", &self.renyi.is_some())
            .field("zcdp", &self.zcdp.is_some())
            .finish()
    }
}

type RenyiFn = dyn Fn(&[f64]) -> Fallible<Vec<f64>> + Send + Sync;

#[derive(Clone)]
struct RenyiRepresentation {
    curve: Arc<RenyiFn>,
    source_delta: f64,
}

impl RenyiRepresentation {
    fn as_profile(&self) -> Fallible<PrivacyProfile> {
        let curve = self.curve.clone();
        PrivacyProfile::new_log(move |epsilons| {
            crate::measures::rdp_to_approxdp::batch::rdp_log_delta_batch(curve.as_ref(), epsilons)
        }).with_delta_slack(self.source_delta)
    }
}

#[derive(Clone, Copy)]
struct ZCDPRepresentation {
    rho: f64,
    source_delta: f64,
}

#[cfg(feature = "idealized-numerics")]
#[derive(Clone, Copy)]
struct GaussianDPRepresentation {
    mu: f64,
}

fn combine_bounds<const N: usize>(
    bounds: [Option<f64>; N],
    combine: fn(f64, f64) -> f64,
) -> Fallible<f64> {
    bounds
        .into_iter()
        .flatten()
        .reduce(combine)
        .map_or(Ok(0.0), Ok)
}

impl PrivacyProfile {
    /// Construct a profile from a vector-valued, upward-conservative delta callback.
    ///
    /// Outputs must lie in `[0, 1]` and match the input length. Conversion to log
    /// space rounds upward, so ordinary-delta equality and plateaus may become
    /// looser. A returned zero is trusted as exact zero, not numerical underflow.
    pub fn new(delta: impl Fn(&[f64]) -> Fallible<Vec<f64>> + 'static + Send + Sync) -> Self {
        Self::new_log(move |epsilon| {
            let values = delta(epsilon)?;
            check_length(epsilon, &values)?;
            values
                .into_iter()
                .map(|value| {
                    check_delta(value)?;
                    delta_to_log_upper_unchecked(value)
                })
                .collect()
        })
    }

    pub(crate) fn new_log(
        log_delta: impl Fn(&[f64]) -> Fallible<Vec<f64>> + 'static + Send + Sync,
    ) -> Self {
        Self {
            log_delta: Arc::new(log_delta),
            delta_slack: 0.0,
        }
    }

    /// Replace the core delta callback.
    /// See [`Self::new`] for the vector callback contract and conversion rounding.
    #[cfg(feature = "honest-but-curious")]
    pub fn with_profile(
        mut self,
        delta: impl Fn(&[f64]) -> Fallible<Vec<f64>> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.log_delta = Self::new(delta).log_delta;
        Ok(self)
    }

    /// Replace the core with a vector-valued conservative log-delta callback.
    /// Outputs must match the input length, be at most zero and not NaN.
    /// Negative infinity certifies zero core delta.
    #[cfg(feature = "honest-but-curious")]
    pub fn with_log_profile(
        mut self,
        log_delta: impl Fn(&[f64]) -> Fallible<Vec<f64>> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.log_delta = Arc::new(log_delta);
        Ok(self)
    }

    /// Set this profile's additive delta slack, independently of its core.
    pub fn with_delta_slack(mut self, delta_slack: f64) -> Fallible<Self> {
        check_delta(delta_slack)?;
        self.delta_slack = delta_slack.max(0.0);
        Ok(self)
    }

    /// Evaluate total delta bounds for nondecreasing epsilons with one core call.
    /// Bounds round upward and are capped at one; finite logs never underflow to zero.
    pub fn delta(&self, epsilon: &[f64]) -> Fallible<Vec<f64>> {
        eval_log_profile(self.log_delta.as_ref(), epsilon)?
            .into_iter()
            .map(|core| {
                if core == f64::NEG_INFINITY {
                    return Ok(self.delta_slack);
                }
                if self.delta_slack == 0.0 {
                    return log_to_delta_upper(core);
                }
                (SInterval::point(self.delta_slack)?
                    + SInterval::point(core.max(-1024.0))?.exp()?)?
                .min(SInterval::point(1.0)?)
                .upper_f64()
            })
            .collect()
    }

    /// Evaluate total log-delta upper bounds with one core call for the whole batch.
    pub fn log_delta(&self, epsilon: &[f64]) -> Fallible<Vec<f64>> {
        eval_log_profile(self.log_delta.as_ref(), epsilon)?
            .into_iter()
            .map(|core| {
                if self.delta_slack == 0.0 {
                    return Ok(core);
                }
                logaddexp_upper(core, delta_to_log_upper_unchecked(self.delta_slack)?)
            })
            .collect()
    }

    /// Invert nondecreasing delta budgets, subtracting profile-specific slack.
    pub fn epsilon(&self, delta: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(delta, check_delta)?;
        let mut results = vec![f64::INFINITY; delta.len()];
        let mut targets = Vec::new();
        let mut indices = Vec::new();
        for (index, &delta) in delta.iter().enumerate() {
            if delta == 1.0 {
                results[index] = 0.0;
                continue;
            }
            if delta < self.delta_slack {
                continue;
            }
            targets.push(if delta == self.delta_slack {
                // Preserve exact source-space equality, even for subnormal slack.
                f64::NEG_INFINITY
            } else if self.delta_slack == 0.0 {
                delta_to_log_lower_unchecked(delta)?
            } else {
                (SInterval::point(delta)? - SInterval::point(self.delta_slack)?)?
                    .ln()?
                    .lower_f64()?
            });
            indices.push(index);
        }
        for (index, value) in indices.into_iter().zip(self.invert_core(&targets)?) {
            results[index] = value;
        }
        Ok(results)
    }

    /// Invert nondecreasing total log-delta budgets without narrowing to delta.
    /// Slack subtraction and logarithms round down before generic inversion.
    pub fn epsilon_from_log_delta(&self, log_delta: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(log_delta, check_log_delta)?;
        let mut results = vec![f64::INFINITY; log_delta.len()];
        let mut targets = Vec::new();
        let mut indices = Vec::new();
        for (index, &log_delta) in log_delta.iter().enumerate() {
            if log_delta == 0.0 {
                results[index] = 0.0;
                continue;
            }
            let target = if self.delta_slack == 0.0 {
                log_delta
            } else {
                if log_delta < delta_to_log_lower_unchecked(self.delta_slack)? {
                    continue;
                }
                let remaining =
                    (SInterval::point(log_delta)?.exp()? - SInterval::point(self.delta_slack)?)?;
                if !remaining.is_nonnegative() {
                    // No nonnegative remaining budget was certified.
                    continue;
                }
                if remaining.contains_zero() {
                    f64::NEG_INFINITY
                } else {
                    remaining.ln()?.lower_f64()?
                }
            };
            targets.push(target);
            indices.push(index);
        }
        for (index, value) in indices.into_iter().zip(self.invert_core(&targets)?) {
            results[index] = value;
        }
        Ok(results)
    }

    fn invert_core(&self, targets: &[f64]) -> Fallible<Vec<f64>> {
        targets
            .iter()
            .map(|&target| {
                invert_decreasing_callback(
                    |epsilon| Ok(eval_log_profile(self.log_delta.as_ref(), &[epsilon])?[0]),
                    target,
                )
            })
            .collect()
    }

    /// Derive a pure-DP endpoint by inversion; no support metadata is stored.
    pub fn pure_epsilon(&self) -> Fallible<Option<f64>> {
        let epsilon = self.epsilon(&[0.0])?[0];
        Ok(epsilon.is_finite().then_some(epsilon))
    }
}

fn check_inputs(values: &[f64], check: fn(f64) -> Fallible<()>) -> Fallible<()> {
    for &value in values {
        check(value)?;
    }
    if values.windows(2).any(|pair| pair[0] > pair[1]) {
        return fallible!(FailedMap, "privacy inputs must be nondecreasing");
    }
    Ok(())
}

fn check_length(inputs: &[f64], outputs: &[f64]) -> Fallible<()> {
    if outputs.len() != inputs.len() {
        return fallible!(
            FailedMap,
            "privacy callback output length ({}) must match input length ({})",
            outputs.len(),
            inputs.len()
        );
    }
    Ok(())
}

fn eval_log_profile(profile: &LogDeltaFn, epsilon: &[f64]) -> Fallible<Vec<f64>> {
    check_inputs(epsilon, check_epsilon)?;
    let values = profile(epsilon)?;
    check_length(epsilon, &values)?;
    for &value in &values {
        check_log_delta(value)?;
    }
    Ok(values)
}

/// Invert a nonincreasing callback while preserving the left edge of plateaus.
fn invert_decreasing_callback(
    callback: impl Fn(f64) -> Fallible<f64>,
    target: f64,
) -> Fallible<f64> {
    // Intermediate overflow is not evidence about the final comparison.
    let predicate = |epsilon: &f64| Ok(callback(*epsilon)? <= target);
    if predicate(&0.0)? {
        return Ok(0.0);
    }
    let mut lower = 0.0;
    // Every callback error, including Search, escapes discovery and bisection.
    for upper in f64::bands(0.0, 1).into_iter().skip(1) {
        if predicate(&upper)? {
            return fallible_binary_search(predicate, (lower, upper));
        }
        lower = upper;
    }
    Ok(f64::INFINITY)
}

/// Lower-bound the left generalized inverse with strict numerical witnesses.
fn invert_beta_callback(beta: impl Fn(f64) -> Fallible<f64>, target: f64) -> Fallible<f64> {
    let predicate = |alpha: &f64| {
        let value = beta(*alpha)?;
        check_beta(value)?;
        Ok(value > target)
    };
    if !predicate(&0.0)? {
        return Ok(0.0);
    }
    if predicate(&1.0)? {
        return Ok(1.0);
    }
    fallible_binary_search(predicate, (0.0, 1.0))
}

fn check_alpha(alpha: f64) -> Fallible<()> {
    check_unit_interval(alpha, "alpha")
}

fn check_beta(beta: f64) -> Fallible<()> {
    check_unit_interval(beta, "beta")
}

fn check_unit_interval(value: f64, name: &str) -> Fallible<()> {
    if !value.is_finite() || value < 0.0 || value > 1.0 {
        return fallible!(FailedMap, "{name} ({value}) must be between zero and one");
    }
    Ok(())
}

fn check_epsilon(epsilon: f64) -> Fallible<()> {
    if epsilon.is_nan() || epsilon < 0.0 {
        return fallible!(
            FailedMap,
            "epsilon ({epsilon}) must be non-negative and not NaN"
        );
    }
    Ok(())
}

fn check_rho(rho: f64) -> Fallible<()> {
    if rho.is_nan() || rho < 0.0 {
        return fallible!(FailedMap, "rho must be non-negative and not NaN");
    }
    Ok(())
}

fn log_to_delta_lower_budget(log_delta: f64) -> Fallible<f64> {
    if log_delta < -1024.0 { return Ok(0.0); }
    SInterval::point(log_delta)?.exp()?.lower_f64()
}

#[cfg(feature = "idealized-numerics")]
fn check_mu(mu: f64) -> Fallible<()> {
    if !mu.is_finite() || mu.is_sign_negative() {
        return fallible!(FailedMap, "mu must be finite and non-negative");
    }
    Ok(())
}

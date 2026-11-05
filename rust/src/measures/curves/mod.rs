use std::sync::Arc;

use crate::{
    error::Fallible,
    traits::SInterval,
    utilities::search::{Bands, fallible_binary_search},
};

mod approxdp;
pub(crate) mod logspace;
mod profile_to_tradeoff;
mod renyidp;
#[cfg(feature = "ffi")]
mod renyidp_ffi;

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

    /// Materialize a staircase only when a profile is needed.
    /// Source delta is additive slack, preserving exact plateau equality.
    fn into_profile(self) -> Fallible<PrivacyProfile> {
        PrivacyProfile::new_log(move |epsilons| {
            Ok(epsilons
                .iter()
                .map(|&epsilon| {
                    if epsilon < self.epsilon {
                        0.0
                    } else {
                        f64::NEG_INFINITY
                    }
                })
                .collect())
        })
        .with_delta_slack(self.delta)
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
}

impl PrivacyGuarantee {
    /// Construct an empty guarantee.
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

    /// Attach an RDP representation with OpenDP's additive source-delta semantics.
    #[cfg(feature = "honest-but-curious")]
    #[allow(non_snake_case)]
    pub fn with_renyiDP(
        mut self,
        curve: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        source_delta: f64,
    ) -> Fallible<Self> {
        check_delta(source_delta)?;
        self.renyi = Some(RenyiRepresentation {
            curve: Arc::new(curve),
            source_delta,
        });
        Ok(self)
    }

    #[allow(dead_code)]
    #[allow(non_snake_case)]
    pub(crate) fn with_renyiDP_trusted(
        mut self,
        curve: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        source_delta: f64,
    ) -> Fallible<Self> {
        check_delta(source_delta)?;
        self.renyi = Some(RenyiRepresentation {
            curve: Arc::new(curve),
            source_delta,
        });
        Ok(self)
    }

    /// Return the tightest delta bound, propagating representation errors.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;
        combine_bounds(
            [
                self.approx_dp.as_ref().map(|repr| {
                    if epsilon < repr.epsilon {
                        1.0
                    } else {
                        repr.delta
                    }
                }),
                self.profile
                    .as_ref()
                    .map(|profile| profile.delta(&[epsilon]).map(|v| v[0]))
                    .transpose()?,
                self.renyi
                    .as_ref()
                    .map(|repr| {
                        renyidp::delta_via_renyiDP(repr.curve.as_ref(), repr.source_delta, epsilon)
                    })
                    .transpose()?,
            ],
            f64::min,
        )
    }

    /// Return the tightest epsilon bound, propagating representation errors.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;
        combine_bounds(
            [
                self.approx_dp.as_ref().map(|repr| {
                    if delta == 1.0 {
                        0.0
                    } else if delta < repr.delta {
                        f64::INFINITY
                    } else {
                        repr.epsilon
                    }
                }),
                self.profile
                    .as_ref()
                    .map(|profile| profile.epsilon(&[delta]).map(|v| v[0]))
                    .transpose()?,
                self.renyi
                    .as_ref()
                    .map(|repr| {
                        invert_decreasing_callback(
                            |epsilon| {
                                renyidp::delta_via_renyiDP(
                                    repr.curve.as_ref(),
                                    repr.source_delta,
                                    epsilon,
                                )
                            },
                            delta,
                        )
                    })
                    .transpose()?,
            ],
            f64::min,
        )
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
        f.debug_struct("PrivacyGuarantee")
            .field("approx_dp", &self.approx_dp)
            .field("profile", &self.profile.is_some())
            .field("renyi", &self.renyi.is_some())
            .finish()
    }
}

type RenyiFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

#[derive(Clone)]
struct RenyiRepresentation {
    curve: Arc<RenyiFn>,
    source_delta: f64,
}

fn combine_bounds<const N: usize>(
    bounds: [Option<f64>; N],
    combine: fn(f64, f64) -> f64,
) -> Fallible<f64> {
    bounds
        .into_iter()
        .flatten()
        .reduce(combine)
        .ok_or_else(|| err!(FailedFunction, "PrivacyGuarantee has no representation"))
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
    if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
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

use std::sync::Arc;

use super::rdp_to_approxdp::linear::{zcdp_epsilon_batch, zcdp_log_delta_batch};

use crate::{
    error::Fallible, traits::SInterval, utilities::search::fallible_invert_decreasing_batch,
};

mod approxdp;
#[cfg(feature = "idealized-numerics")]
mod gaussiandp;
#[cfg(all(feature = "ffi", feature = "idealized-numerics"))]
mod gaussiandp_ffi;
pub(crate) mod logspace;
mod maximize;
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
/// Callbacks must be functionally pure and pointwise conservative, with
/// nonincreasing bounds within each batch. Bounds may depend on the whole
/// batch; the same epsilon need not receive the same bound in different batches.
/// Inputs must be nondecreasing, permitting duplicates. Only output ranges and
/// lengths are checked. Inversion returns a certified passing finite epsilon
/// when found, not necessarily the least possible epsilon.
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
        self.epsilon_from_complement((SInterval::point(1.0)? - SInterval::point(delta)?)?)
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
        let ratio = (complement / (one.clone() - SInterval::point(self.delta)?)?)?.min(one.clone());
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
    /// Output lengths must match; converters may request batches of orders.
    /// With positive `source_delta`, all orders must bound the same pair of
    /// conditioned distributions in the common-good-events definition of
    /// approximate RDP; the excluded mass is bounded by `source_delta`.
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
        check_inputs(epsilon, check_epsilon)?;
        let mut bounds: Vec<f64> = vec![1.0; epsilon.len()];
        if let Some(repr) = &self.approx_dp {
            for (bound, &query) in bounds.iter_mut().zip(epsilon) {
                *bound = bound.min(repr.delta(query)?);
            }
        }
        if let Some(profile) = &self.profile {
            min_assign(&mut bounds, profile.delta(epsilon)?);
        }
        if let Some(repr) = &self.renyi {
            min_assign(
                &mut bounds,
                renyidp::delta_via_renyiDP_batch(repr.curve.as_ref(), repr.source_delta, epsilon)?,
            );
        }
        if let Some(repr) = &self.zcdp {
            let values = zcdp_log_delta_batch(repr.rho, epsilon)?
                .into_iter()
                .map(|log_delta| log_to_delta_upper(log_delta, repr.source_delta))
                .collect::<Fallible<Vec<_>>>()?;
            min_assign(&mut bounds, values);
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = &self.gaussian {
            let values = gaussiandp::log_delta_via_gaussianDP_batch(repr.mu, epsilon)?
                .into_iter()
                .map(|log_delta| log_to_delta_upper(log_delta, 0.0))
                .collect::<Fallible<Vec<_>>>()?;
            min_assign(&mut bounds, values);
        }
        Ok(bounds)
    }

    /// Evaluate a nondecreasing batch of delta queries.
    pub fn epsilon_batch(&self, delta: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(delta, check_delta)?;
        let mut bounds: Vec<_> = delta
            .iter()
            .map(|&d| if d == 1.0 { 0.0 } else { f64::INFINITY })
            .collect();
        if let Some(repr) = &self.approx_dp {
            for (bound, &query) in bounds.iter_mut().zip(delta) {
                *bound = bound.min(repr.epsilon(query)?);
            }
        }
        if let Some(profile) = &self.profile {
            min_assign(&mut bounds, profile.epsilon(delta)?);
        }
        if let Some(repr) = &self.renyi {
            min_assign(
                &mut bounds,
                renyidp::epsilon_via_renyiDP_batch(repr.curve.as_ref(), repr.source_delta, delta)?,
            );
        }
        if let Some(repr) = &self.zcdp {
            let values = invert_delta_budgets(delta, repr.source_delta, false, |targets| {
                zcdp_epsilon_batch(repr.rho, targets)
            })?;
            min_assign(&mut bounds, values);
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = &self.gaussian {
            let values = invert_delta_budgets(delta, 0.0, false, |targets| {
                let evaluate = gaussiandp::log_delta_evaluator(repr.mu)?;
                fallible_invert_decreasing_batch(
                    |epsilons| epsilons.iter().copied().map(&evaluate).collect(),
                    targets,
                )
            })?;
            min_assign(&mut bounds, values);
        }
        Ok(bounds)
    }

    /// Evaluate a nondecreasing batch of epsilon queries in log-delta space.
    pub fn log_delta_batch(&self, epsilon: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(epsilon, check_epsilon)?;
        let mut bounds: Vec<f64> = vec![0.0; epsilon.len()];
        if let Some(repr) = &self.approx_dp {
            for (bound, &query) in bounds.iter_mut().zip(epsilon) {
                *bound = bound.min(repr.log_delta(query)?);
            }
        }
        if let Some(profile) = &self.profile {
            min_assign(&mut bounds, profile.log_delta(epsilon)?);
        }
        if let Some(repr) = &self.renyi {
            min_assign(
                &mut bounds,
                renyidp::log_delta_via_renyiDP_batch(
                    repr.curve.as_ref(),
                    repr.source_delta,
                    epsilon,
                )?,
            );
        }
        if let Some(repr) = &self.zcdp {
            let values = zcdp_log_delta_batch(repr.rho, epsilon)?
                .into_iter()
                .map(|log_delta| add_log_delta_slack(log_delta, repr.source_delta))
                .collect::<Fallible<Vec<_>>>()?;
            min_assign(&mut bounds, values);
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = &self.gaussian {
            let values = gaussiandp::log_delta_via_gaussianDP_batch(repr.mu, epsilon)?;
            min_assign(&mut bounds, values);
        }
        Ok(bounds)
    }

    /// Invert a nondecreasing batch of log-delta budgets.
    pub fn epsilon_from_log_delta_batch(&self, log_delta: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(log_delta, check_log_delta)?;
        let mut bounds: Vec<_> = log_delta
            .iter()
            .map(|&d| if d == 0.0 { 0.0 } else { f64::INFINITY })
            .collect();
        if let Some(repr) = &self.approx_dp {
            for (bound, &query) in bounds.iter_mut().zip(log_delta) {
                *bound = bound.min(repr.epsilon_from_log_delta(query)?);
            }
        }
        if let Some(profile) = &self.profile {
            min_assign(&mut bounds, profile.epsilon_from_log_delta(log_delta)?);
        }
        if let Some(repr) = &self.renyi {
            min_assign(
                &mut bounds,
                renyidp::epsilon_from_log_delta_via_renyiDP_batch(
                    repr.curve.as_ref(),
                    repr.source_delta,
                    log_delta,
                )?,
            );
        }
        if let Some(repr) = &self.zcdp {
            let values = invert_delta_budgets(log_delta, repr.source_delta, true, |targets| {
                zcdp_epsilon_batch(repr.rho, targets)
            })?;
            min_assign(&mut bounds, values);
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = &self.gaussian {
            let values = invert_delta_budgets(log_delta, 0.0, true, |targets| {
                let evaluate = gaussiandp::log_delta_evaluator(repr.mu)?;
                fallible_invert_decreasing_batch(
                    |epsilons| epsilons.iter().copied().map(&evaluate).collect(),
                    targets,
                )
            })?;
            min_assign(&mut bounds, values);
        }
        Ok(bounds)
    }

    /// Return the strongest beta lower bounds for nondecreasing alpha inputs.
    /// Every attached representation is evaluated; its errors propagate.
    pub fn beta(&self, alphas: &[f64]) -> Fallible<Vec<f64>> {
        check_inputs(alphas, check_alpha)?;
        let mut bounds = vec![0.0_f64; alphas.len()];
        if let Some(repr) = &self.approx_dp {
            for (bound, &alpha) in bounds.iter_mut().zip(alphas) {
                *bound = approxdp::beta_via_approx_dp(repr.epsilon, repr.delta, alpha)?;
            }
        }
        if let Some(profile) = &self.profile {
            max_assign(
                &mut bounds,
                profile_to_tradeoff::beta_via_profile_batch(profile, alphas)?,
            );
        }
        if let Some(repr) = &self.renyi {
            max_assign(
                &mut bounds,
                renyidp::beta_via_renyiDP_batch(repr.curve.clone(), repr.source_delta, alphas)?,
            );
        }
        if let Some(repr) = &self.zcdp {
            max_assign(
                &mut bounds,
                profile_to_tradeoff::beta_via_profile_batch(&repr.as_profile()?, alphas)?,
            );
        }
        #[cfg(feature = "idealized-numerics")]
        if let Some(repr) = &self.gaussian {
            max_assign(
                &mut bounds,
                gaussiandp::beta_via_gaussianDP_batch(repr.mu, alphas)?,
            );
        }
        Ok(bounds)
    }

    /// Return conservative alpha lower bounds by symmetry of neighboring datasets.
    pub fn alpha(&self, betas: &[f64]) -> Fallible<Vec<f64>> {
        self.beta(betas)
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

fn min_assign(bounds: &mut [f64], other: Vec<f64>) {
    for (bound, value) in bounds.iter_mut().zip(other) {
        *bound = bound.min(value);
    }
}

fn max_assign(bounds: &mut [f64], other: Vec<f64>) {
    for (bound, value) in bounds.iter_mut().zip(other) {
        *bound = bound.max(value);
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

impl ZCDPRepresentation {
    fn as_profile(&self) -> Fallible<PrivacyProfile> {
        let rho = self.rho;
        PrivacyProfile::new_log(move |epsilons| zcdp_log_delta_batch(rho, epsilons))
            .with_delta_slack(self.source_delta)
    }
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
            .map(|core| log_to_delta_upper(core, self.delta_slack))
            .collect()
    }

    /// Evaluate total log-delta upper bounds with one core call for the whole batch.
    pub fn log_delta(&self, epsilon: &[f64]) -> Fallible<Vec<f64>> {
        eval_log_profile(self.log_delta.as_ref(), epsilon)?
            .into_iter()
            .map(|core| add_log_delta_slack(core, self.delta_slack))
            .collect()
    }

    /// Invert nondecreasing delta budgets, subtracting profile-specific slack.
    pub fn epsilon(&self, delta: &[f64]) -> Fallible<Vec<f64>> {
        invert_delta_budgets(delta, self.delta_slack, false, |targets| {
            self.invert_core(targets)
        })
    }

    /// Invert nondecreasing total log-delta budgets without narrowing to delta.
    pub fn epsilon_from_log_delta(&self, log_delta: &[f64]) -> Fallible<Vec<f64>> {
        invert_delta_budgets(log_delta, self.delta_slack, true, |targets| {
            self.invert_core(targets)
        })
    }

    fn invert_core(&self, targets: &[f64]) -> Fallible<Vec<f64>> {
        fallible_invert_decreasing_batch(
            |epsilon| eval_log_profile(self.log_delta.as_ref(), epsilon),
            targets,
        )
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

fn add_log_delta_slack(core: f64, slack: f64) -> Fallible<f64> {
    if slack == 0.0 {
        return Ok(core);
    }
    logaddexp_upper(core, delta_to_log_upper_unchecked(slack)?)
}

// Invert total delta budgets using a core log-delta inverse. This is
// numerical slack handling, independent of the source representation.
fn invert_delta_budgets(
    budgets: &[f64],
    slack: f64,
    logarithmic: bool,
    invert: impl FnOnce(&[f64]) -> Fallible<Vec<f64>>,
) -> Fallible<Vec<f64>> {
    check_inputs(
        budgets,
        if logarithmic {
            check_log_delta
        } else {
            check_delta
        },
    )?;
    let mut results = vec![f64::INFINITY; budgets.len()];
    let mut targets = Vec::new();
    let mut indices = Vec::new();
    for (index, &budget) in budgets.iter().enumerate() {
        if budget == if logarithmic { 0.0 } else { 1.0 } {
            results[index] = 0.0;
            continue;
        }
        let target = if logarithmic {
            if slack == 0.0 {
                budget
            } else {
                if budget < delta_to_log_lower_unchecked(slack)? {
                    continue;
                }
                let remaining = (SInterval::point(budget)?.exp()? - SInterval::point(slack)?)?;
                if !remaining.is_nonnegative() {
                    continue;
                }
                if remaining.contains_zero() {
                    f64::NEG_INFINITY
                } else {
                    remaining.ln()?.lower_f64()?
                }
            }
        } else if budget < slack {
            continue;
        } else if budget == slack {
            f64::NEG_INFINITY
        } else if slack == 0.0 {
            delta_to_log_lower_unchecked(budget)?
        } else {
            (SInterval::point(budget)? - SInterval::point(slack)?)?
                .ln()?
                .lower_f64()?
        };
        targets.push(target);
        indices.push(index);
    }
    if !targets.is_empty() {
        for (index, value) in indices.into_iter().zip(invert(&targets)?) {
            results[index] = value;
        }
    }
    Ok(results)
}

fn check_alpha(alpha: f64) -> Fallible<()> {
    check_unit_interval(alpha, "alpha")
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

#[cfg(feature = "idealized-numerics")]
fn check_mu(mu: f64) -> Fallible<()> {
    if !mu.is_finite() || mu.is_sign_negative() {
        return fallible!(FailedMap, "mu must be finite and non-negative");
    }
    Ok(())
}

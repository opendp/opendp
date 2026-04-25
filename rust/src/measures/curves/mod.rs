use std::sync::Arc;

use crate::{
    measures::curves::gdp::{beta_via_gdp, delta_via_gdp},
    traits::InfCast,
    utilities::maximize_ternary,
};
use dashu::{
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
    rbig,
};

use crate::error::Fallible;

#[cfg(feature = "ffi")]
mod ffi;

mod gdp;

#[cfg(test)]
mod test;

/// A unified representation of privacy guarantees that can be queried as either
/// a privacy profile `delta(epsilon)` or an f-DP tradeoff curve `beta(alpha)`.
#[derive(Clone)]
pub struct PrivacyCurve(PrivacyCurveRepr);

#[deprecated(since = "0.15.0", note = "Use PrivacyCurve instead.")]
/// Compatibility alias while callers migrate to [`PrivacyCurve`].
pub type PrivacyProfile = PrivacyCurve;

#[derive(Clone)]
enum PrivacyCurveRepr {
    Profile { delta: Arc<ProfileFn> },
    Tradeoff { beta: Arc<TradeoffFn> },
    ApproxDP { points: Arc<Vec<ApproxDPPoint>> },
    GDP { mu: f64 },
}

type ProfileFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type TradeoffFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

#[derive(Clone, Debug)]
pub(crate) struct ApproxDPPoint {
    epsilon: f64,
    delta: f64,
    // allows to cache computations that are repeated many times in tradeoff evaluation
    one_minus_delta: RBig,
    exp_eps_up: RBig,
    exp_neg_eps_down: RBig,
}

const EPS_MAX_START: f64 = 32.0;
const EPS_MAX: f64 = 1e6;

impl PrivacyCurve {
    /// Construct a privacy curve from a callback mapping `epsilon -> delta`.
    pub(crate) fn new_profile(
        delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Self {
        Self(PrivacyCurveRepr::Profile {
            delta: Arc::new(delta),
        })
    }

    /// Construct a privacy curve from a callback mapping `alpha -> beta`.
    pub(crate) fn new_tradeoff(
        beta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Self {
        Self(PrivacyCurveRepr::Tradeoff {
            beta: Arc::new(beta),
        })
    }

    /// Invert the privacy profile by finding the smallest `epsilon`
    /// such that `delta(epsilon) <= delta`.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;
        if delta == 1.0 {
            return Ok(0.0);
        }
        let mut e_min: f64 = 0.0;
        let mut e_max: f64 = 2.0;
        while self.delta(e_max)? > delta {
            e_min = e_max;
            e_max *= e_max;
            if e_max.is_infinite() {
                return Ok(f64::INFINITY);
            }
        }

        let mut e_mid = e_min;
        loop {
            let new_mid = e_min + ((e_max - e_min) / 2.0);

            if new_mid == e_mid {
                return Ok(if delta == self.delta(e_min)? {
                    e_min
                } else {
                    e_max
                });
            }

            e_mid = new_mid;

            let d_mid: f64 = self.delta(e_mid)?;
            if d_mid > delta {
                e_min = e_mid
            } else {
                e_max = e_mid
            }
        }
    }

    /// Evaluate the privacy profile at `epsilon`.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;
        match &self.0 {
            PrivacyCurveRepr::Profile { delta } => delta(epsilon),
            PrivacyCurveRepr::ApproxDP { points } => delta_via_approxdp(points, epsilon),
            PrivacyCurveRepr::GDP { mu } => delta_via_gdp(*mu, epsilon),
            PrivacyCurveRepr::Tradeoff { beta } => delta_via_tradeoff(beta, epsilon),
        }
    }

    /// Evaluate the f-DP tradeoff curve at `alpha`.
    pub fn beta(&self, alpha: f64) -> Fallible<f64> {
        check_alpha(alpha)?;
        if alpha == 0.0 {
            return Ok(1.0);
        }
        if alpha == 1.0 {
            return Ok(0.0);
        }

        match &self.0 {
            PrivacyCurveRepr::Tradeoff { beta } => beta(alpha),
            PrivacyCurveRepr::GDP { mu } => beta_via_gdp(*mu, alpha),
            PrivacyCurveRepr::ApproxDP { points } => beta_via_approxdp(points, alpha),
            PrivacyCurveRepr::Profile { delta } => beta_via_profile(delta, alpha),
        }
    }

    pub fn new_approxdp(mut points: Vec<(f64, f64)>) -> Fallible<Self> {
        if points.is_empty() {
            return fallible!(
                MakeMeasurement,
                "privacy curve must be defined by at least one approximate-DP pair"
            );
        }

        // For duplicate epsilons, keep the largest delta conservatively.
        points.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| b.1.total_cmp(&a.1)));
        points.dedup_by(|a, b| a.0 == b.0);

        let mut min_delta = 1.0;
        for (_epsilon, delta) in &points {
            if *delta > min_delta {
                return fallible!(
                    MakeMeasurement,
                    "delta values must be monotonically nonincreasing as epsilon increases"
                );
            }
            min_delta = min_delta.min(*delta);
        }

        Ok(Self(PrivacyCurveRepr::ApproxDP {
            points: Arc::new(
                points
                    .into_iter()
                    .map(ApproxDPPoint::build)
                    .collect::<Fallible<_>>()?,
            ),
        }))
    }

    pub fn new_gdp(mu: f64) -> Fallible<Self> {
        if !mu.is_finite() || mu < 0.0 {
            return fallible!(
                MakeMeasurement,
                "mu ({mu}) must be a finite non-negative number"
            );
        }

        Ok(Self(PrivacyCurveRepr::GDP { mu }))
    }
}

fn beta_via_profile(profile: &Arc<ProfileFn>, alpha: f64) -> Fallible<f64> {
    let alpha = RBig::try_from(alpha)?;
    let mut eps_hi = EPS_MAX_START;
    let mut last_delta = profile(eps_hi)?;
    while last_delta > 1e-12 && eps_hi.is_finite() && eps_hi < EPS_MAX {
        eps_hi *= 2.0;
        let delta = profile(eps_hi)?;
        if delta >= last_delta {
            break;
        }
        last_delta = delta;
    }
    eps_hi = eps_hi.min(EPS_MAX);

    let best = maximize_ternary(0.0, eps_hi, |eps| {
        let point = ApproxDPPoint::build((eps, profile(eps)?))?;
        f64::neg_inf_cast(point.beta(&alpha))
    })?;

    Ok(best.next_down().clamp(0.0, 1.0))
}

fn delta_via_tradeoff(tradeoff: &Arc<TradeoffFn>, epsilon: f64) -> Fallible<f64> {
    let neg_epsilon_down = (-FBig::<Up>::try_from(epsilon)?).with_rounding::<Down>();
    let precision = neg_epsilon_down.precision().max(10);
    let neg_epsilon_down = neg_epsilon_down.with_precision(precision).value();

    let exp_neg_eps_down = RBig::try_from(neg_epsilon_down.exp())?;

    let best = maximize_ternary(0.0, 1.0, |alpha| {
        let beta = tradeoff(alpha)?;
        let alpha = RBig::try_from(alpha)?;
        let beta = RBig::try_from(beta)?;

        let c1 = RBig::ONE - &exp_neg_eps_down * &alpha - &beta;
        let c2 = RBig::ONE - &alpha - &exp_neg_eps_down * &beta;

        f64::inf_cast(c1.max(c2).max(RBig::ZERO))
    })?;

    Ok(best.next_up().clamp(0.0, 1.0))
}

impl ApproxDPPoint {
    pub fn build((epsilon, delta): (f64, f64)) -> Fallible<Self> {
        check_epsilon(epsilon)?;
        check_delta(delta)?;

        let epsilon_up = FBig::<Up>::try_from(epsilon)?;
        let precision = epsilon_up.precision().max(10);
        let epsilon_up = epsilon_up.with_precision(precision).value();

        let exp_eps_up = RBig::try_from(epsilon_up.clone().exp())?;

        let epsilon_down = (-epsilon_up).with_rounding::<Down>();
        let exp_neg_eps_down = RBig::try_from(epsilon_down.exp())?;

        Ok(Self {
            epsilon,
            delta,
            one_minus_delta: RBig::ONE - RBig::try_from(delta)?,
            exp_eps_up,
            exp_neg_eps_down,
        })
    }

    #[inline]
    pub fn beta(&self, alpha: &RBig) -> RBig {
        let t1 = &self.one_minus_delta - &self.exp_eps_up * alpha;
        let base = (&self.one_minus_delta - alpha).max(rbig!(0));
        let t2 = &self.exp_neg_eps_down * base;

        t1.max(t2).max(rbig!(0))
    }
}

fn delta_via_approxdp(points: &[ApproxDPPoint], epsilon: f64) -> Fallible<f64> {
    let idx = points.partition_point(|point| point.epsilon <= epsilon);
    Ok(if idx == 0 { 1.0 } else { points[idx - 1].delta })
}

fn beta_via_approxdp(points: &[ApproxDPPoint], alpha: f64) -> Fallible<f64> {
    let alpha = RBig::try_from(alpha)?;

    let best = points
        .iter()
        .map(|p| p.beta(&alpha))
        .max()
        .unwrap_or_default();

    Ok(f64::inf_cast(best)?.clamp(0.0, 1.0))
}

fn check_alpha(alpha: f64) -> Fallible<()> {
    if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
        return fallible!(
            FailedMap,
            "alpha ({alpha}) must be a finite number in [0, 1]"
        );
    }
    Ok(())
}

fn check_epsilon(epsilon: f64) -> Fallible<()> {
    if epsilon.is_nan() {
        return fallible!(FailedMap, "epsilon must not be nan");
    }
    if epsilon.is_sign_negative() {
        return fallible!(
            FailedMap,
            "epsilon ({epsilon}) must be a non-negative number"
        );
    }
    Ok(())
}

fn check_delta(delta: f64) -> Fallible<()> {
    if !delta.is_finite() {
        return fallible!(FailedMap, "delta ({delta}) must be finite");
    }
    if delta.is_sign_negative() || delta > 1.0 {
        return fallible!(FailedMap, "delta ({delta}) must be between zero and one");
    }
    Ok(())
}

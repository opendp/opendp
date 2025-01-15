//! Various definitions of Measures (and associated Distances).
//!
//! A Privacy Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    core::{Function, Measure},
    error::Fallible,
};

/// Privacy measure used to define $\epsilon$-pure differential privacy.
///
/// In the following proof definition, $d$ corresponds to $\epsilon$ when also quantified over all adjacent datasets.
/// That is, $\epsilon$ is the greatest possible $d$
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// ### `d`-closeness
///
/// For any two distributions $Y, Y'$ and any non-negative $d$,
/// $Y, Y'$ are $d$-close under the max divergence measure whenever
///
/// ```math
/// D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d.
/// ```
#[derive(Default, Clone, Debug, PartialEq)]
pub struct MaxDivergence;

impl Measure for MaxDivergence {
    type Distance = f64;
}

/// Privacy measure used to define $\delta(\epsilon)$-approximate differential privacy.
///
/// In the following proof definition, $d$ corresponds to a privacy profile when also quantified over all adjacent datasets.
/// That is, a privacy profile $\delta(\epsilon)$ is no smaller than $d(\epsilon)$ for all possible choices of $\epsilon$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// The distance $d$ is of type [`PrivacyProfile`], so it can be invoked with an $\epsilon$
/// to retrieve the corresponding $\delta$.
///
/// # Proof Definition
///
/// ### `d`-closeness
///
/// For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
/// $Y, Y'$ are $d$-close under the smoothed max divergence measure whenever,
/// for any choice of non-negative $\epsilon$, and $\delta = d(\epsilon)$,
///
/// ```math
/// D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon.
/// ```
///
/// Note that $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct SmoothedMaxDivergence;

impl Measure for SmoothedMaxDivergence {
    type Distance = PrivacyProfile;
}

/// A function mapping from $\epsilon$ to $\delta$
///
/// This is the distance type for [`SmoothedMaxDivergence`].
#[derive(Clone)]
pub struct PrivacyProfile(Arc<dyn Fn(f64) -> Fallible<f64> + Send + Sync>);

impl PrivacyProfile {
    pub fn new(delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        PrivacyProfile(Arc::new(delta))
    }

    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        // reject negative zero
        if delta.is_sign_negative() {
            return fallible!(FailedMap, "delta ({}) must not be negative", delta);
        }

        if !(0.0..=1.0).contains(&delta) {
            return fallible!(FailedMap, "delta ({}) must be between zero and one", delta);
        }

        if delta == 1.0 {
            return Ok(0.0);
        }

        self.epsilon_unchecked(delta)
    }

    pub(crate) fn epsilon_unchecked(&self, delta: f64) -> Fallible<f64> {
        let mut e_min: f64 = 0.0;
        let mut e_max: f64 = 2.0;
        while self.delta(e_max)? > delta {
            e_max *= e_max;
            if e_max.is_infinite() {
                // For mechanisms that are not pureDP, e_max will be infinity.
                // This loop can be very long running.
                return Ok(f64::INFINITY);
            }
        }

        // delta(e_max) <= delta <= delta(e_min) -> always holds
        // We always try to find the smallest e that minimizes |delta(e) - delta| and enforces delta(e) <= delta
        //           -> if delta == delta(e_min), we can pick e_min, otherwise we have to take e_max
        // same as   -> if e
        // For delta == 1.0, we find the largest e that gives delta(e) == 1.0
        // (so as not to create a discontinuity and go to zero.)
        let mut e_mid = e_min;
        loop {
            let new_mid = e_min + ((e_max - e_min) / 2.0);

            // converge when midpoint doesn't change
            if new_mid == e_mid {
                if delta == 1. {
                    return Ok(e_max);
                }

                return Ok(if delta == self.delta(e_min)? {
                    e_min
                } else {
                    e_max
                });
            }

            e_mid = new_mid;

            // get delta corresponding to e_mid
            let d_mid: f64 = self.delta(e_mid)?;
            match d_mid.partial_cmp(&delta) {
                Some(Ordering::Greater) => e_min = e_mid,
                Some(Ordering::Less) => e_max = e_mid,
                Some(Ordering::Equal) => {
                    if delta == 1. {
                        e_min = e_mid
                    } else {
                        e_max = e_mid
                    }
                }
                None => return fallible!(FailedMap, "not comparable"),
            }
        }
    }

    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        (self.0)(epsilon)
    }
}

/// Privacy measure used to define $\delta$-approximate PM-differential privacy.
///
/// In the following definition, $d$ corresponds to privacy parameters $(d', \delta)$
/// when also quantified over all adjacent datasets
/// ($d'$ is the privacy parameter corresponding to privacy measure PM).
/// That is, $(d', \delta)$ is no smaller than $d$ (by product ordering),
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two distributions $Y, Y'$ and 2-tuple $d = (d', \delta)$,
/// where $d'$ is the distance with respect to privacy measure PM,
/// $Y, Y'$ are $d$-close under the approximate PM measure whenever,
/// for any choice of $\delta \in [0, 1]$,
/// there exist events $E$ (depending on $Y$) and $E'$ (depending on $Y'$)
/// such that $\Pr[E] \ge 1 - \delta$, $\Pr[E'] \ge 1 - \delta$, and
///
/// ```math
/// D_{\mathrm{PM}}^\delta(Y|_E, Y'|_{E'}) = D_{\mathrm{PM}}(Y|_E, Y'|_{E'})
/// ```
///
/// where $Y|_E$ denotes the distribution of $Y$ conditioned on the event $E$.
///
/// Note that this $\delta$ is not privacy parameter $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Approximate<PM: Measure>(pub PM);

impl<M: Measure> Measure for Approximate<M> {
    type Distance = (M::Distance, f64);
}

/// Privacy measure used to define $\rho$-zero concentrated differential privacy.
///
/// In the following proof definition, $d$ corresponds to $\rho$ when also quantified over all adjacent datasets.
/// That is, $\rho$ is the greatest possible $d$
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// ### `d`-closeness
///
/// For any two distributions $Y, Y'$ and any non-negative $d$,
/// $Y, Y'$ are $d$-close under the zero-concentrated divergence measure if,
/// for every possible choice of $\alpha \in (1, \infty)$,
///
/// ```math
/// D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d \cdot \alpha.
/// ```
#[derive(Default, Clone, Debug, PartialEq)]
pub struct ZeroConcentratedDivergence;

impl Measure for ZeroConcentratedDivergence {
    type Distance = f64;
}

/// Privacy measure used to define $\epsilon(\alpha)$-Rényi differential privacy.
///
/// In the following proof definition, $d$ corresponds to an RDP curve when also quantified over all adjacent datasets.
/// That is, an RDP curve $\epsilon(\alpha)$ is no smaller than $d(\alpha)$ for any possible choices of $\alpha$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two distributions $Y, Y'$ and any curve $d$,
/// $Y, Y'$ are $d$-close under the Rényi divergence measure if,
/// for any given $\alpha \in (1, \infty)$,
///
/// ```math
/// D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d(\alpha).
/// ```
///
/// Note that this $\epsilon$ and $\alpha$ are not privacy parameters $\epsilon$ and $\alpha$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct RenyiDivergence;

impl Measure for RenyiDivergence {
    type Distance = Function<f64, f64>;
}

//! Various definitions of Measures (and associated Distances).
//!
//! A Privacy Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

pub(crate) mod curves;
pub use curves::*;

use std::fmt::Debug;

use crate::core::{Function, Measure};

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
/// ## `d`-closeness
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

/// Privacy measure used to define privacy guarantees represented by a [`PrivacyCurve`].
///
/// There is a dual interpretation of the privacy curve.
/// The curve can be evaluated as a privacy profile via [`PrivacyCurve::delta`]
/// or as an f-DP tradeoff curve via [`PrivacyCurve::beta`].
///
/// Under the privacy profile interpretation,
/// $d$ corresponds to a privacy profile when also quantified over all adjacent datasets.
/// That is, a privacy profile $\delta(\epsilon)$ is no smaller than $d(\epsilon)$ for all possible choices of $\epsilon$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// Under the tradeoff curve interpretation,
/// In one sense, $d$ corresponds to an $f$-DP tradeoff curve
/// when also quantified over all adjacent datasets.
/// That is, a tradeoff curve $\beta(\alpha)$ is no smaller than $d(\alpha)$
/// for all possible choices of $\alpha$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// The distance $d$ is of type [`TradeoffCurve`], so it can be invoked with an $\alpha$
/// to retrieve the corresponding $\beta$.
///
/// # Proof Definition
///
/// ## `d`-closeness ($f$-DP)
/// For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
/// we say that $Y, Y'$ are $d$-close under f-DP
/// whenever, for every $\alpha \in [0, 1]$,
/// with $\beta = d(\alpha)$,
///
/// ```math
/// T(Y, Y')(\alpha) \ge \beta,
/// ```
///
/// where $T(Y, Y')$ is the hypothesis-testing tradeoff function between $Y$ and $Y'$.
///
/// Note that this $\alpha$ and $\beta$ are not privacy parameters
/// until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
///
/// ## `d`-closeness (profile-DP)
///
/// For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
/// we say that $Y, Y'$ are $d$-close under the smoothed max divergence measure
/// whenever, for every non-negative $\epsilon$, with $\delta = d(\epsilon)$,
/// and for every event $S \subseteq \mathrm{Supp}(Y)$,
///
/// ```math
/// \Pr[Y \in S] \le e^\epsilon \Pr[Y' \in S] + \delta.
/// ```
///
/// Note that $\epsilon$ and $\delta$ are not privacy parameters
/// until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct PrivacyCurveDP;

impl Measure for PrivacyCurveDP {
    type Distance = PrivacyCurve;
}

#[deprecated(since = "0.15.0", note = "Use `PrivacyCurveDP` instead.")]
pub type SmoothedMaxDivergence = PrivacyCurveDP;

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
/// ## `d`-closeness
/// For any two distributions $Y, Y'$ and 2-tuple $d = (d', \delta)$,
/// where $d'$ is the distance with respect to privacy measure PM,
/// we say that $Y, Y'$ are $d$-close under the approximate PM measure
/// whenever they satisfy the privacy guarantee of PM with parameter $d'$,
/// up to slack $\delta$.
///
/// The exact interpretation of the slack depends on the underlying privacy
/// measure PM.
///
/// ### Special case: `PM = MaxDivergence`
/// When $d = (\epsilon, \delta)$ and `PM = MaxDivergence`,
/// this is exactly fixed $(\epsilon, \delta)$-approximate differential privacy:
///
/// ```math
/// \Pr[Y \in S] \le e^\epsilon \Pr[Y' \in S] + \delta
/// \quad\text{for every event } S \subseteq \mathrm{Supp}(Y).
/// ```
///
/// The profile form of this notion, where $\delta$ is a function of $\epsilon$,
/// is represented by [`PrivacyCurveDP`].
///
/// Note that $d'$ and $\delta$ are not privacy parameters until quantified over
/// all adjacent datasets, as is done in the definition of a measurement.
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
/// ## `d`-closeness
///
/// For any two distributions $Y, Y'$ and any non-negative $d$,
/// we say that $Y, Y'$ are $d$-close under the zero-concentrated divergence measure
/// whenever, for every $\alpha \in (1, \infty)$,
///
/// ```math
/// D_\alpha(Y, Y') = \frac{1}{\alpha - 1}
/// \ln \mathbb{E}_{x \sim Y'} \left[ \left(
/// \dfrac{\Pr[Y = x]}{\Pr[Y' = x]}
/// \right)^\alpha \right] \le d \cdot \alpha.
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
/// ## `d`-closeness
/// For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
/// we say that $Y, Y'$ are $d$-close under the Rényi divergence measure
/// whenever, for every $\alpha \in (1, \infty)$,
///
/// ```math
/// D_\alpha(Y, Y') = \frac{1}{\alpha - 1}
/// \ln \mathbb{E}_{x \sim Y'} \left[ \left(
/// \dfrac{\Pr[Y = x]}{\Pr[Y' = x]}
/// \right)^\alpha \right] \le d(\alpha).
/// ```
///
/// Note that this $\epsilon$ and $\alpha$ are not privacy parameters
/// until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct RenyiDivergence;

impl Measure for RenyiDivergence {
    type Distance = Function<f64, f64>;
}

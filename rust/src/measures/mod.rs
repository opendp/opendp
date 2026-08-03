//! Various definitions of Measures (and associated Distances).
//!
//! A Privacy Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

pub(crate) mod rdp_to_approxdp;
pub(crate) mod zcdp;
pub(crate) use zcdp::{zcdp_delta, zcdp_epsilon, zcdp_log_delta};

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
/// $Y, Y'$ are $d$-close under the pure-DP privacy measure whenever
///
/// ```math
/// D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d.
/// ```
#[derive(Default, Clone, Debug, PartialEq)]
pub struct PureDP;

#[deprecated(since = "1.15.0", note = "Use `PureDP` instead.")]
pub type MaxDivergence = PureDP;

impl Measure for PureDP {
    type Distance = f64;
}

/// `MultiDP` is a privacy measure whose distance is a [`PrivacyGuarantee`],
/// allowing a measurement to retain multiple valid DP representations
/// simultaneously.
///
/// A `PrivacyGuarantee` contains multiple simultaneously valid privacy
/// representations for the same mechanism and neighboring relation. Every
/// stored representation holds conjunctively; representations may differ in
/// strength and in their closure properties under later operations. The
/// guarantee can be queried as a privacy profile via [`PrivacyGuarantee::delta`]
/// or as an f-DP tradeoff function via [`PrivacyGuarantee::beta`].
///
/// Under the privacy profile interpretation,
/// $d$ corresponds to a privacy profile when also quantified over all adjacent datasets.
/// That is, a privacy profile $\delta(\epsilon)$ is no smaller than $d(\epsilon)$ for all possible choices of $\epsilon$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// Under the tradeoff-function interpretation,
/// $d$ corresponds to an $f$-DP tradeoff function when also quantified over all
/// adjacent datasets. That is, a tradeoff function $\beta(\alpha)$ is no smaller
/// than $d(\alpha)$ for all $\alpha$ and all adjacent datasets.
///
/// The distance $d$ is a [`PrivacyGuarantee`] and can be queried in either form.
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
/// we say that $Y, Y'$ are $d$-close under the profile-DP privacy measure
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
pub struct MultiDP;

impl Measure for MultiDP {
    type Distance = PrivacyGuarantee;
}

#[deprecated(since = "0.15.0", note = "Use `MultiDP` instead.")]
pub type SmoothedMaxDivergence = MultiDP;

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
/// ### Special case: `PM = PureDP`
/// When $d = (\epsilon, \delta)$ and `PM = PureDP`,
/// this is exactly fixed $(\epsilon, \delta)$-approximate differential privacy:
///
/// ```math
/// \Pr[Y \in S] \le e^\epsilon \Pr[Y' \in S] + \delta
/// \quad\text{for every event } S \subseteq \mathrm{Supp}(Y).
/// ```
///
/// The profile form of this notion, where $\delta$ is a function of $\epsilon$,
/// can be retained as one representation in [`MultiDP`].
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
/// we say that $Y, Y'$ are $d$-close under the zCDP privacy measure
/// whenever, for every $\alpha \in (1, \infty)$,
///
/// ```math
/// D_\alpha(Y, Y') = \frac{1}{\alpha - 1}
/// \ln \mathbb{E}_{x \sim Y'} \left[ \left(
/// \dfrac{\Pr[Y = x]}{\Pr[Y' = x]}
/// \right)^\alpha \right] \le d \cdot \alpha.
/// ```
#[derive(Default, Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub struct zCDP;

#[deprecated(since = "1.15.0", note = "Use `zCDP` instead.")]
pub type ZeroConcentratedDivergence = zCDP;

impl Measure for zCDP {
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
/// we say that $Y, Y'$ are $d$-close under the Rényi-DP privacy measure
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
pub struct RenyiDP;

#[deprecated(since = "1.15.0", note = "Use `RenyiDP` instead.")]
pub type RenyiDivergence = RenyiDP;

impl Measure for RenyiDP {
    type Distance = Function<f64, f64>;
}

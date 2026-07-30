//! Various definitions of Measures (and associated Distances).
//!
//! A Privacy Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

pub(crate) mod rdp_to_approxdp;
pub(crate) mod zcdp;

pub(crate) mod curves;
pub use curves::*;

#[cfg(test)]
mod test;

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
/// ## `d`-closeness
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
pub struct SmoothedMaxDivergence;

impl Measure for SmoothedMaxDivergence {
    type Distance = PrivacyProfile;
}

/// Purity level of a guaranteed privacy representation.
///
/// `Approximate` permits the representation family's nonzero source slack,
/// while `Pure` guarantees its zero-slack / pure form according to that
/// representation family's semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Purity {
    Approximate,
    Pure,
}

/// Construction-time lower bounds on the representations a [`MultiDP`]
/// privacy map guarantees to return.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PrivacyCapabilities {
    profile: Option<Purity>,
    tradeoff: bool,
    gaussian: bool,
    renyi: Option<Purity>,
    zcdp: Option<Purity>,
}

impl PrivacyCapabilities {
    pub(crate) fn with_profile(purity: Purity) -> Self {
        Self {
            profile: Some(purity),
            tradeoff: false,
            gaussian: false,
            renyi: None,
            zcdp: None,
        }
    }

    pub(crate) fn profile(&self) -> Option<Purity> {
        self.profile
    }

    pub(crate) fn with_tradeoff(mut self) -> Self {
        self.tradeoff = true;
        self
    }

    pub(crate) fn tradeoff(&self) -> bool {
        self.tradeoff
    }

    pub(crate) fn with_gaussian(mut self) -> Self {
        self.gaussian = true;
        self
    }

    pub(crate) fn gaussian(&self) -> bool {
        self.gaussian
    }

    pub(crate) fn with_renyi(mut self, purity: Purity) -> Self {
        self.renyi = Some(purity);
        self
    }

    pub(crate) fn renyi(&self) -> Option<Purity> {
        self.renyi
    }

    pub(crate) fn with_zcdp(mut self, purity: Purity) -> Self {
        self.zcdp = Some(purity);
        self
    }

    pub(crate) fn zcdp(&self) -> Option<Purity> {
        self.zcdp
    }
}

/// Privacy measure whose distance contains all privacy facts returned by a
/// privacy map.
///
/// Capabilities describe only the representations that every successful map
/// invocation is guaranteed to provide. They do not alter the mathematical
/// privacy relation, so they are intentionally ignored by equality.
#[derive(Clone, Debug)]
pub struct MultiDP {
    capabilities: PrivacyCapabilities,
}

impl MultiDP {
    pub(crate) fn new(capabilities: PrivacyCapabilities) -> Self {
        Self { capabilities }
    }

    pub(crate) fn with_profile(purity: Purity) -> Self {
        Self::new(PrivacyCapabilities::with_profile(purity))
    }

    pub(crate) fn with_tradeoff() -> Self {
        Self::new(PrivacyCapabilities::default().with_tradeoff())
    }

    pub(crate) fn with_renyi(purity: Purity) -> Self {
        Self::new(PrivacyCapabilities::default().with_renyi(purity))
    }

    pub(crate) fn with_gaussian() -> Self {
        Self::new(PrivacyCapabilities::default().with_gaussian())
    }

    pub(crate) fn with_zcdp(purity: Purity) -> Self {
        Self::new(PrivacyCapabilities::default().with_zcdp(purity))
    }

    pub(crate) fn capabilities(&self) -> &PrivacyCapabilities {
        &self.capabilities
    }
}

impl Default for MultiDP {
    fn default() -> Self {
        Self::new(PrivacyCapabilities::default())
    }
}

impl PartialEq for MultiDP {
    fn eq(&self, _other: &Self) -> bool {
        // Capability richness describes metadata about maps, not a different
        // mathematical relation.
        true
    }
}

impl Measure for MultiDP {
    type Distance = PrivacyGuarantee;
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
/// is represented by [`SmoothedMaxDivergence`].
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

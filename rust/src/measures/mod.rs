//! Various definitions of Measures (and associated Distances).
//!
//! A Privacy Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

use std::{fmt::Debug, sync::Arc};

use crate::{core::Measure, error::Fallible};

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

/// Privacy measure used to define $\epsilon(\delta)$-approximate differential privacy.
///
/// In the following proof definition, $d$ corresponds to a privacy profile when also quantified over all adjacent datasets.
/// That is, a privacy profile $\epsilon(\delta)$ is no smaller than $d(\delta)$ for all possible choices of $\delta$,
/// and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// Privacy profiles are represented by the type [`SMDCurve`].
/// This curve can be evaluated with a $\delta$ to retrieve a corresponding $\epsilon$.
///
/// # Proof Definition
///
/// ### `d`-closeness
///
/// For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
/// $Y, Y'$ are $d$-close under the smoothed max divergence measure whenever,
/// for any choice of $\delta \in [0, 1]$,
///
/// ```math
/// D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq d(\delta).
/// ```
///
/// Note that this $\delta$ is not privacy parameter $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct SmoothedMaxDivergence;

impl Measure for SmoothedMaxDivergence {
    type Distance = SMDCurve;
}

/// A function mapping from $\delta$ to $\epsilon$
///
/// SMD stands for "Smoothed Max Divergence".
/// This is the distance type for [`SmoothedMaxDivergence`].
pub struct SMDCurve(Arc<dyn Fn(&f64) -> Fallible<f64> + Send + Sync>);

impl Clone for SMDCurve {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl SMDCurve {
    pub fn new(epsilon: impl Fn(&f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        SMDCurve(Arc::new(epsilon))
    }

    // these functions allow direct invocation as a method, making parens unnecessary
    pub fn epsilon(&self, delta: &f64) -> Fallible<f64> {
        (self.0)(delta)
    }
}

/// Privacy measure used to define $(\epsilon, \delta)$-approximate differential privacy.
///
/// In the following definition, $d$ corresponds to $(\epsilon, \delta)$ when also quantified over all adjacent datasets.
/// That is, $(\epsilon, \delta)$ is no smaller than $d$ (by product ordering),
/// over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
/// $M(\cdot)$ is a measurement (commonly known as a mechanism).
/// The measurement's input metric defines the notion of adjacency,
/// and the measurement's input domain defines the set of possible datasets.
///
/// # Proof Definition
///
/// ### `d`-closeness
///
/// For any two distributions $Y, Y'$ and any 2-tuple $d$ of non-negative numbers $\epsilon$ and $\delta$,
/// $Y, Y'$ are $d$-close under the fixed smoothed max divergence measure whenever
///
/// ```math
/// D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon.
/// ```
///
/// Note that this $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
/// as is done in the definition of a measurement.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct FixedSmoothedMaxDivergence;

impl Measure for FixedSmoothedMaxDivergence {
    type Distance = (f64, f64);
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

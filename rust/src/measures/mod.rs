//! Various implementations of Measures (and associated Distance).
//!
//! A Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

use std::{fmt::Debug, sync::Arc};

use crate::{core::Measure, error::Fallible};

/// $\epsilon$-pure differential privacy.
///
/// The greatest divergence between any randomly selected subset of the support.
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two vectors $u, v \in \texttt{D}$ and any $d$ of generic type $\texttt{Q}$,
/// we say that $M(u), M(v)$ are $d$-close under the max divergence measure (abbreviated as $D_{\infty}$) whenever
///
/// ```math
/// D_{\infty}(M(u) \| M(v)) = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[M(u) \in S]}{\Pr[M(v) \in S]} \Big] \leq d.
/// ```
#[derive(Default, Clone, Debug, PartialEq)]
pub struct MaxDivergence;

impl Measure for MaxDivergence {
    type Distance = f64;
}

/// $\epsilon(\delta)$-approximate differential privacy.
///
/// The greatest divergence between any randomly selected subset of the support,
/// with an additive tolerance for error.
///
/// The distance $d$ is of type [`SMDCurve`], so it can be invoked with a $\delta$
/// to retrieve the tightest corresponding $\epsilon$.
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two vectors $u, v \in \texttt{D}$
/// and any choice of $\epsilon, \delta$ such that $\epsilon \ge d(\delta)$,
/// we say that $M(u), M(v)$ are $d$-close under the smoothed max divergence measure (abbreviated as $D_{S\infty}$) whenever
///
/// ```math
/// D_{S\infty}(M(u) \| M(v)) = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[M(u) \in S] + \delta}{\Pr[M(v) \in S]} \Big] \leq \epsilon.
/// ```
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

/// $(\epsilon, \delta)$-approximate differential privacy.
///
/// The greatest divergence between any randomly selected subset of the support,
/// with an additive tolerance for error.
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two vectors $u, v \in \texttt{D}$ and any $d$ of type $(\texttt{Q}, \texttt{Q})$,
/// where $d = (\epsilon, \delta)$,
/// we say that $M(u), M(v)$ are $d$-close under the smoothed max divergence measure (abbreviated as $D_{S\infty}$) whenever
///
/// ```math
/// D_{S\infty}(M(u) \| M(v)) = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[M(u) \in S] + \delta}{\Pr[M(v) \in S]} \Big] \leq \epsilon.
/// ```
#[derive(Clone, PartialEq, Debug, Default)]
pub struct FixedSmoothedMaxDivergence;

impl Measure for FixedSmoothedMaxDivergence {
    type Distance = (f64, f64);
}

/// $\rho$-zero concentrated differential privacy.
///
/// The greatest zero-concentrated divergence between any randomly selected subset of the support.
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two vectors $u, v \in \texttt{D}$ and any $d$ of generic type $\texttt{Q}$,
/// define $P$ and $Q$ to be the distributions of $M(u)$ and $M(v)$.
/// We say that $u, v$ are $d$-close under the alpha-Renyi divergence measure (abbreviated as $D_{\alpha}$) whenever
///
/// ```math
/// D_{\alpha}(P \| Q) = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Q} \Big[\ln \left( \dfrac{P(x)}{Q(x)} \right)^\alpha \Big] \leq d \alpha.
/// ```
/// for all possible choices of $\alpha \in (1, \infty)$.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct ZeroConcentratedDivergence;

impl Measure for ZeroConcentratedDivergence {
    type Distance = f64;
}

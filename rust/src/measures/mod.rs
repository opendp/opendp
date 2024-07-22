//! Various implementations of Measures (and associated Distance).
//!
//! A Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

mod f_dp;

use std::{
    cmp::Ordering, fmt::{Debug, Formatter}, sync::Arc
};

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
pub struct MaxDivergence;
impl Default for MaxDivergence {
    fn default() -> Self {
        MaxDivergence
    }
}

impl Clone for MaxDivergence {
    fn clone(&self) -> Self {
        MaxDivergence
    }
}

impl PartialEq for MaxDivergence {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Debug for MaxDivergence {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "MaxDivergence()")
    }
}

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
pub struct SmoothedMaxDivergence;

impl Default for SmoothedMaxDivergence {
    fn default() -> Self {
        SmoothedMaxDivergence
    }
}
impl Clone for SmoothedMaxDivergence {
    fn clone(&self) -> Self {
        SmoothedMaxDivergence
    }
}

impl PartialEq for SmoothedMaxDivergence {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Debug for SmoothedMaxDivergence {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SmoothedMaxDivergence()")
    }
}

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

/// $\delta$-approximate differential privacy for any compatible privacy measure $MO$.
///
/// # Proof Definition
///
/// ### `d`-closeness
/// For any two data sets $x, x' \in \texttt{D}$ and any $d$ of type $(\texttt{MO::Distance}, \texttt{MO::Distance})$,
/// where $d = (d', \delta)$,
/// we say that $M(x), M(x)$ are $d$-close under the $\delta$-approximate $MO$-DP (abbreviated as $D_{MO}^{\delta}$)
/// when there exist events $E$ (depending on $M(x)$) and $E'$ (depending on $M(x')$)
/// such that $\Pr[E] \ge 1 - \delta$, $\Pr[E'] \ge 1 - \delta$, and
///
/// ```math
/// D_{MO}^{\delta}(M(x)|_E \|\| M(x')|_{E'}) = D_{MO}(M(x)|_E \|\| M(x')|_{E'})
/// ```
///
/// where $(M(x)|_E) denotes the distribution of $M(x)$ conditioned on the event $E$.
///
pub struct Approximate<M: Measure>(pub M);

impl<M: Measure + Default> Default for Approximate<M> {
    fn default() -> Self {
        Approximate(M::default())
    }
}
impl<M: Measure> Clone for Approximate<M> {
    fn clone(&self) -> Self {
        Approximate(self.0.clone())
    }
}

impl<M: Measure> PartialEq for Approximate<M> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<M: Measure> Debug for Approximate<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Approximate({:?})", self.0)
    }
}

impl<M: Measure> Measure for Approximate<M> {
    type Distance = (M::Distance, f64);
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
pub struct ZeroConcentratedDivergence;
impl Default for ZeroConcentratedDivergence {
    fn default() -> Self {
        ZeroConcentratedDivergence
    }
}
impl Clone for ZeroConcentratedDivergence {
    fn clone(&self) -> Self {
        ZeroConcentratedDivergence
    }
}

impl PartialEq for ZeroConcentratedDivergence {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Debug for ZeroConcentratedDivergence {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ZeroConcentratedDivergence()")
    }
}

impl Measure for ZeroConcentratedDivergence {
    type Distance = f64;
}

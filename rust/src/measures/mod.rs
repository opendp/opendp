//! Various implementations of Measures (and associated Distance).
//!
//! A Measure is used to measure the distance between distributions.
//! The distance is expressed in terms of an **associated type**.
//!
//! # Example
//! `MaxDivergence<Q>` has an associated distance type of `Q`.
//! This means that the symmetric distance between vectors is expressed in terms of the type `Q`.
//! In this context Q is usually [`f32`] or [`f64`].

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    sync::Arc,
};

use crate::{core::Measure, domains::type_name, error::Fallible};

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
pub struct MaxDivergence<Q>(PhantomData<fn() -> Q>);
impl<Q> Default for MaxDivergence<Q> {
    fn default() -> Self {
        MaxDivergence(PhantomData)
    }
}

impl<Q> Clone for MaxDivergence<Q> {
    fn clone(&self) -> Self {
        MaxDivergence(PhantomData)
    }
}

impl<Q> PartialEq for MaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<Q> Debug for MaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "MaxDivergence({})", type_name!(Q))
    }
}

impl<Q> Measure for MaxDivergence<Q> {
    type Distance = Q;
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
pub struct SmoothedMaxDivergence<Q>(PhantomData<fn() -> Q>);

impl<Q> Default for SmoothedMaxDivergence<Q> {
    fn default() -> Self {
        SmoothedMaxDivergence(PhantomData)
    }
}
impl<Q> Clone for SmoothedMaxDivergence<Q> {
    fn clone(&self) -> Self {
        SmoothedMaxDivergence(PhantomData)
    }
}

impl<Q> PartialEq for SmoothedMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<Q> Debug for SmoothedMaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SmoothedMaxDivergence({})", type_name!(Q))
    }
}

impl<Q> Measure for SmoothedMaxDivergence<Q> {
    type Distance = SMDCurve<Q>;
}

/// A function mapping from $\delta$ to $\epsilon$
///
/// SMD stands for "Smoothed Max Divergence".
/// This is the distance type for [`SmoothedMaxDivergence`].
pub struct SMDCurve<Q>(Arc<dyn Fn(&Q) -> Fallible<Q> + Send + Sync>);

impl<Q> Clone for SMDCurve<Q> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Q> SMDCurve<Q> {
    pub fn new(epsilon: impl Fn(&Q) -> Fallible<Q> + 'static + Send + Sync) -> Self {
        SMDCurve(Arc::new(epsilon))
    }

    // these functions allow direct invocation as a method, making parens unnecessary
    pub fn epsilon(&self, delta: &Q) -> Fallible<Q> {
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
pub struct FixedSmoothedMaxDivergence<Q>(PhantomData<fn() -> Q>);

impl<Q> Default for FixedSmoothedMaxDivergence<Q> {
    fn default() -> Self {
        FixedSmoothedMaxDivergence(PhantomData)
    }
}
impl<Q> Clone for FixedSmoothedMaxDivergence<Q> {
    fn clone(&self) -> Self {
        FixedSmoothedMaxDivergence(PhantomData)
    }
}

impl<Q> PartialEq for FixedSmoothedMaxDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<Q> Debug for FixedSmoothedMaxDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "FixedSmoothedMaxDivergence({})", type_name!(Q))
    }
}

impl<Q> Measure for FixedSmoothedMaxDivergence<Q> {
    type Distance = (Q, Q);
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
pub struct ZeroConcentratedDivergence<Q>(PhantomData<fn() -> Q>);
impl<Q> Default for ZeroConcentratedDivergence<Q> {
    fn default() -> Self {
        ZeroConcentratedDivergence(PhantomData)
    }
}
impl<Q> Clone for ZeroConcentratedDivergence<Q> {
    fn clone(&self) -> Self {
        ZeroConcentratedDivergence(PhantomData)
    }
}

impl<Q> PartialEq for ZeroConcentratedDivergence<Q> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<Q> Debug for ZeroConcentratedDivergence<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ZeroConcentratedDivergence({})", type_name!(Q))
    }
}

impl<Q> Measure for ZeroConcentratedDivergence<Q> {
    type Distance = Q;
}

//! Various implementations of Metrics (and associated Distance).
//! 
//! A Metric is used to measure the distance between data. 
//! Metrics are paired with a **domain** on which the metric can measure distance.
//! The distance is expressed in terms of an **associated type**.
//! 
//! # Example
//! 
//! [`SymmetricDistance`] can be paired with a domain: `VectorDomain(AllDomain(T))`. 
//! In this context, the `SymmetricDistance` is used to measure the distance between any two vectors of elements of type `T`. 
//! The `SymmetricDistance` has an associated distance type of [`u32`].
//! This means that the symmetric distance between vectors is expressed in terms of a [`u32`].

use std::{marker::PhantomData};

use crate::{core::Metric, domains::type_name};
use std::fmt::{Debug, Formatter};

/// The type that represents the distance between datasets.
/// It is used as the associated [`Metric`]::Distance type for e.g. [`SymmetricDistance`], [`InsertDeleteDistance`], etc.
pub type IntDistance = u32;


/// The smallest number of additions or removals to make two datasets equivalent.
/// 
/// This metric is not sensitive to data ordering.
/// Because this metric counts additions and removals, 
/// it is an unbounded metric (for unbounded DP).
/// 
/// # Proof Definition
/// 
/// ### `d`-closeness
/// For any two vectors $u, v \in \texttt{D}$ and any $d$ of type [`IntDistance`], 
/// we say that $u, v$ are $d$-close under the symmetric distance metric 
/// (abbreviated as $d_{Sym}$) whenever 
/// 
/// ```math
/// d_{Sym}(u, v) = |u \Delta v| \leq d
/// ```
/// # Note
/// The distance type is hard-coded as [`IntDistance`], 
/// so this metric is not generic over the distance type like many other metrics.
/// 
/// # Compatible Domains
/// 
/// * `VectorDomain<D>` for any valid `D`
/// * `SizedDomain<VectorDomain<D>>` for any valid `D`
/// 
/// When this metric is paired with a `VectorDomain`, we instead consider the multisets corresponding to $u, v \in \texttt{D}$.
#[derive(Clone)]
pub struct SymmetricDistance;

impl Default for SymmetricDistance {
    fn default() -> Self { SymmetricDistance }
}

impl PartialEq for SymmetricDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for SymmetricDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SymmetricDistance()")
    }
}
impl Metric for SymmetricDistance {
    type Distance = IntDistance;
}

/// The smallest number of insertions or deletions to make two datasets equivalent.
/// 
/// An *insertion* to a dataset is an addition of an element at a specific index,
/// and a *deletion* is the removal of an element at a specific index.
/// 
/// Therefore, this metric is sensitive to data ordering.
/// Because this metric counts insertions and deletions, 
/// it is an unbounded metric (for unbounded DP).
/// 
/// # Proof Definition
/// 
/// ### `d`-closeness
/// For any two vectors $u, v \in \texttt{D}$ and any $d$ of type [`IntDistance`], 
/// we say that $u, v$ are $d$-close under the insert-delete distance metric 
/// (abbreviated as $d_{ID}$) whenever 
/// 
/// ```math
/// d_{ID}(u, v) \leq d
/// ```
/// 
/// # Note
/// The distance type is hard-coded as [`IntDistance`], 
/// so this metric is not generic over the distance type like many other metrics.
/// 
/// # Compatible Domains
/// 
/// * `VectorDomain<D>` for any valid `D`
/// * `SizedDomain<VectorDomain<D>>` for any valid `D`
#[derive(Clone)]
pub struct InsertDeleteDistance;

impl Default for InsertDeleteDistance {
    fn default() -> Self { InsertDeleteDistance }
}

impl PartialEq for InsertDeleteDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for InsertDeleteDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "InsertDeleteDistance()")
    }
}
impl Metric for InsertDeleteDistance {
    type Distance = IntDistance;
}


/// The smallest number of changes to make two equal-length datasets equivalent.
/// 
/// This metric is not sensitive to data ordering.
/// Since this metric counts the number of changed rows, 
/// it is a bounded metric (for bounded DP).
/// 
/// Since this metric is bounded, the dataset size must be fixed.
/// Thus we only consider neighboring datasets with the same fixed size: [`crate::domains::SizedDomain`].
/// 
/// # Proof Definition
/// 
/// ### `d`-closeness
/// For any two datasets $u, v \in \texttt{D}$ and any $d$ of type [`IntDistance`], 
/// we say that $u, v$ are $d$-close under the change-one distance metric (abbreviated as $d_{CO}$) whenever
/// 
/// ```math
/// d_{CO}(u, v) = d_{Sym}(u, v) / 2 \leq d
/// ```
/// $d_{Sym}$ is in reference to the [`SymmetricDistance`].
/// 
/// # Note
/// Since the dataset size is fixed, 
/// there are always just as many additions as there are removals to reach an adjacent dataset.
/// Consider an edit as one addition and one removal, 
/// therefore the symmetric distance is always even.
/// 
/// The distance type is hard-coded as [`IntDistance`], 
/// so this metric is not generic over the distance type like many other metrics.
/// 
/// WLOG, most OpenDP interfaces need only consider unbounded metrics.
/// Use [`crate::transformations::make_metric_unbounded`] and [`crate::transformations::make_metric_bounded`] 
/// to convert to/from the symmetric distance.
/// 
/// # Compatible Domains
/// 
/// * `SizedDomain<D>` for any valid `D`
#[derive(Clone)]
pub struct ChangeOneDistance;

impl Default for ChangeOneDistance {
    fn default() -> Self { ChangeOneDistance }
}

impl PartialEq for ChangeOneDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for ChangeOneDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ChangeOneDistance()")
    }
}
impl Metric for ChangeOneDistance {
    type Distance = IntDistance;
}

/// The number of elements that differ between two equal-length datasets.
/// 
/// This metric is sensitive to data ordering.
/// Since this metric counts the number of changed rows, 
/// it is a bounded metric (for bounded DP).
/// 
/// Since this metric is bounded, the dataset size must be fixed.
/// Thus we only consider neighboring datasets with the same fixed size: [`crate::domains::SizedDomain`].
/// 
/// # Proof Definition
/// 
/// ### `d`-closeness
/// For any two datasets $u, v \in \texttt{D}$ and any $d$ of type [`IntDistance`], 
/// we say that $u, v$ are $d$-close under the Hamming distance metric (abbreviated as $d_{Ham}$) whenever
/// 
/// ```math
/// d_{Ham}(u, v) = \#\{i: u_i \neq v_i\} \leq d
/// ```
/// 
/// # Note
/// 
/// The distance type is hard-coded as [`IntDistance`], 
/// so this metric is not generic over the distance type like many other metrics.
/// 
/// WLOG, most OpenDP interfaces need only consider unbounded metrics.
/// Use [`crate::transformations::make_metric_unbounded`] and [`crate::transformations::make_metric_bounded`] 
/// to convert to/from the symmetric distance.
/// 
/// # Compatible Domains
/// 
/// * `SizedDomain<D>` for any valid `D`
#[derive(Clone)]
pub struct HammingDistance;

impl Default for HammingDistance {
    fn default() -> Self { HammingDistance }
}

impl PartialEq for HammingDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for HammingDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "HammingDistance()")
    }
}
impl Metric for HammingDistance {
    type Distance = IntDistance;
}

/// The $L_p$ distance between two vector-valued aggregates.
/// 
/// # Proof Definition
/// 
/// ### $d$-closeness
/// For any two vectors $u, v \in \texttt{D}$ and $d$ of generic type $\texttt{Q}$, 
/// we say that $u, v$ are $d$-close under the the $L_p$ distance metric (abbreviated as $d_{LP}$) whenever
/// 
/// ```math
/// d_{LP}(u, v) = \|u_i - v_i\|_p \leq d
/// ```
/// 
/// If $u$ and $v$ are different lengths, then
/// ```math
/// d_{LP}(u, v) = \infty
/// ```
/// 
/// # Compatible Domains
/// 
/// * `VectorDomain<D>` for any valid `D`
/// * `SizedDomain<VectorDomain<D>>` for any valid `D`
/// * `MapDomain<D>` for any valid `D`
/// * `SizedDomain<MapDomain<D>>` for any valid `D`
pub struct LpDistance<const P: usize, Q>(PhantomData<Q>);
impl<const P: usize, Q> Default for LpDistance<P, Q> {
    fn default() -> Self { LpDistance(PhantomData) }
}

impl<const P: usize, Q> Clone for LpDistance<P, Q> {
    fn clone(&self) -> Self { Self::default() }
}
impl<const P: usize, Q> PartialEq for LpDistance<P, Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<const P: usize, Q> Debug for LpDistance<P, Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "L{}Distance({})", P, type_name!(Q))
    }
}
impl<const P: usize, Q> Metric for LpDistance<P, Q> {
    type Distance = Q;
}

/// The $L_1$ distance between two vector-valued aggregates.
/// 
/// Refer to [`LpDistance`] for details.
pub type L1Distance<Q> = LpDistance<1, Q>;

/// The $L_2$ distance between two vector-valued aggregates.
/// 
/// Refer to [`LpDistance`] for details.
pub type L2Distance<Q> = LpDistance<2, Q>;

/// The absolute distance between two scalar-valued aggregates.
/// 
/// # Proof Definition
/// 
/// ### `d`-closeness
/// For any two scalars $u, v \in \texttt{D}$ and $d$ of generic type $\texttt{Q}$, 
/// we say that $u, v$ are $d$-close under the the the absolute distance metric (abbreviated as $d_{Abs}$) whenever
/// 
/// ```math
/// d_{Abs}(u, v) = |u - v| \leq d
/// ```
/// 
/// # Compatible Domains
/// 
/// * `AllDomain<T>` for any valid `T`
pub struct AbsoluteDistance<Q>(PhantomData<Q>);
impl<Q> Default for AbsoluteDistance<Q> {
    fn default() -> Self { AbsoluteDistance(PhantomData) }
}

impl<Q> Clone for AbsoluteDistance<Q> {
    fn clone(&self) -> Self { Self::default() }
}
impl<Q> PartialEq for AbsoluteDistance<Q> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<Q> Debug for AbsoluteDistance<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "AbsoluteDistance({})", type_name!(Q))
    }
}
impl<Q> Metric for AbsoluteDistance<Q> {
    type Distance = Q;
}

/// Indicates if two elements are equal to each other.
/// 
/// This is used in the context of randomized response, 
/// to capture the distance between adjacent inputs (they are either equivalent or not).
/// 
/// # Proof Definition
/// 
/// ### `d`-closeness
/// For any two datasets $u, v \in$ `AllDomain<T>` and any $d$ of type [`IntDistance`], 
/// we say that $u, v$ are $d$-close under the discrete metric (abbreviated as $d_{Eq}$) whenever
/// 
/// ```math
/// d_{Eq}(u, v) = \mathbb{1}[u = v] \leq d
/// ```
/// 
/// # Notes
/// Clearly, `d` is bounded above by 1.
/// 1 is the expected argument on measurements that use this distance.
/// 
/// # Compatible Domains
/// * AllDomain<T> for any valid `T`.
#[derive(Clone)]
pub struct DiscreteDistance;

impl Default for DiscreteDistance {
    fn default() -> Self { DiscreteDistance }
}

impl PartialEq for DiscreteDistance {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Debug for DiscreteDistance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "DiscreteDistance()")
    }
}
impl Metric for DiscreteDistance {
    type Distance = IntDistance;
}

/// A dummy to fill the metric position in postprocessors.
/// 
/// Postprocessors don't necessarily care about matching metrics.
#[derive(Clone, Default, PartialEq)]
pub struct AgnosticMetric;

impl Debug for AgnosticMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "AgnosticMetric()")
    }
}
impl Metric for AgnosticMetric {
    type Distance = ();
}
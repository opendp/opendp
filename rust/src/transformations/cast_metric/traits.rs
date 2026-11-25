use crate::{
    core::Metric,
    metrics::{
        ChangeOneDistance, EventLevelMetric, HammingDistance, InsertDeleteDistance,
        MicrodataMetric, SymmetricDistance,
    },
};

#[cfg(feature = "polars")]
use crate::metrics::{ChangeOneIdDistance, SymmetricIdDistance};

/// UnorderedMetric is implemented for metrics that compute the distance between datasets in a way that is insensitive to row ordering.
///
/// # Proof Definition
/// Permuting the rows of a dataset does not change its distance to any dataset.
pub trait UnorderedMetric: EventLevelMetric {
    /// OrderedMetric denotes the associated metric that is sensitive to row ordering.
    ///
    /// # Proof Definition
    /// For any datasets $x,x'$ at distance at most $d$ under `Self`,
    /// there exists a coupling of their uniformly random row permutations $X,X'$
    /// such that $d_{\mathrm{ordered}}(X,X')\leq d$ almost surely.
    type OrderedMetric: Metric<Distance = Self::Distance> + Default;
}
impl UnorderedMetric for SymmetricDistance {
    type OrderedMetric = InsertDeleteDistance;
}
impl UnorderedMetric for ChangeOneDistance {
    type OrderedMetric = HammingDistance;
}

/// OrderedMetric is implemented for metrics that compute the distance between datasets in a way that is sensitive to row ordering.
///
/// # Proof Definition
/// Unlike an unordered metric, the distance can depend on row ordering.
pub trait OrderedMetric: EventLevelMetric {
    /// UnorderedMetric denotes the associated metric that is insensitive to row ordering.
    ///
    /// # Proof Definition
    /// For any datasets $x,x'$ at distance at most $d$ under `Self`,
    /// their distance under the associated unordered metric is at most $d$.
    type UnorderedMetric: Metric<Distance = Self::Distance> + Default;
}
impl OrderedMetric for InsertDeleteDistance {
    type UnorderedMetric = SymmetricDistance;
}
impl OrderedMetric for HammingDistance {
    type UnorderedMetric = ChangeOneDistance;
}

/// BoundedMetric is implemented for metrics that compute the distance between datasets via replacements of units (records or identifiers).
///
/// # Proof Definition
/// Replacing one unit produces datasets at distance at most one.
pub trait BoundedMetric: MicrodataMetric {
    /// UnboundedMetric denotes the associated metric that measures additions and removals.
    ///
    /// # Proof Definition
    /// For any datasets $x,x'$ at distance at most $d$ under `Self`,
    /// their distance under `self.to_unbounded()` is at most $2d$.
    type UnboundedMetric: UnboundedMetric;
    fn to_unbounded(&self) -> Self::UnboundedMetric;
}
impl BoundedMetric for ChangeOneDistance {
    type UnboundedMetric = SymmetricDistance;
    fn to_unbounded(&self) -> Self::UnboundedMetric {
        SymmetricDistance
    }
}
impl BoundedMetric for HammingDistance {
    type UnboundedMetric = InsertDeleteDistance;
    fn to_unbounded(&self) -> Self::UnboundedMetric {
        InsertDeleteDistance
    }
}
#[cfg(feature = "polars")]
impl BoundedMetric for ChangeOneIdDistance {
    type UnboundedMetric = SymmetricIdDistance;
    fn to_unbounded(&self) -> Self::UnboundedMetric {
        SymmetricIdDistance {
            identifier: self.identifier.clone(),
        }
    }
}

/// UnboundedMetric is implemented for metrics that compute the distance between datasets via additions and removals of units (records or identifiers).
///
/// # Proof Definition
/// Adding or removing one unit produces datasets at distance at most one.
pub trait UnboundedMetric: MicrodataMetric {}
impl UnboundedMetric for SymmetricDistance {}
impl UnboundedMetric for InsertDeleteDistance {}
#[cfg(feature = "polars")]
impl UnboundedMetric for SymmetricIdDistance {}

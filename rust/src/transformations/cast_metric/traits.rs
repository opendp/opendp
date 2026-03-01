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
/// For any choice of x and x' where x' = shuffle(x), then $d(x, x') = 0$.
pub trait UnorderedMetric: EventLevelMetric {
    /// OrderedMetric denotes the associated metric that is sensitive to row ordering.
    ///
    /// # Proof Definition
    /// For any choice of datasets x and x' where d(x, x') > 0 for an OrderedMetric d, and x and x' only differ by the reordering of rows,
    /// then d'(x, x') = 0 for a corresponding UnorderedMetric d'.
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
/// For any choice of x and x' where x' only differs by the reordering of rows, then $d(x, x') > 0$.
pub trait OrderedMetric: EventLevelMetric {
    /// UnorderedMetric denotes the associated metric that is insensitive to row ordering.
    ///
    /// # Proof Definition
    /// For any choice of datasets x, x' where d(x, x') > 0 for an OrderedMetric d and only differs by the reordering of rows,
    /// then d'(x, x') = 0 for a corresponding UnorderedMetric d'.
    type UnorderedMetric: Metric<Distance = Self::Distance> + Default;
}
impl OrderedMetric for InsertDeleteDistance {
    type UnorderedMetric = SymmetricDistance;
}
impl OrderedMetric for HammingDistance {
    type UnorderedMetric = ChangeOneDistance;
}

/// BoundedMetric is implemented for metrics that compute the distance between datasets via record edits.
///
/// # Proof Definition
/// For any choice of x and x' where x' differs solely by the edit of one record, then $d(x, x') = 1$.
pub trait BoundedMetric: MicrodataMetric {
    /// UnboundedMetric denotes the associated metric that measures edit distance.
    ///
    /// # Proof Definition
    /// For any choice of datasets x, x' where d(x, x') = 1 for a BoundedMetric d, then d'(x, x') = 2 for the corresponding UnboundedMetric d'.
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

/// UnboundedMetric is implemented for metrics that compute the distance between datasets via record additions and removals.
///
/// # Proof Definition
/// For any choice of x and x' where x' differs solely by the addition or removal of one record, then $d(x, x') = 1$.
pub trait UnboundedMetric: MicrodataMetric {
    /// BoundedMetric denotes the associated metric that measures edit distance.
    ///
    /// # Proof Definition
    /// For any choice of datasets x, x' where d(x, x') = 1 for a BoundedMetric d, then d'(x, x') = 2 for a corresponding UnboundedMetric d'.
    type BoundedMetric: BoundedMetric;
    fn to_bounded(&self) -> Self::BoundedMetric;
}
impl UnboundedMetric for SymmetricDistance {
    type BoundedMetric = ChangeOneDistance;
    fn to_bounded(&self) -> Self::BoundedMetric {
        ChangeOneDistance
    }
}
impl UnboundedMetric for InsertDeleteDistance {
    type BoundedMetric = HammingDistance;
    fn to_bounded(&self) -> Self::BoundedMetric {
        HammingDistance
    }
}
#[cfg(feature = "polars")]
impl UnboundedMetric for SymmetricIdDistance {
    type BoundedMetric = ChangeOneIdDistance;
    fn to_bounded(&self) -> Self::BoundedMetric {
        ChangeOneIdDistance {
            identifier: self.identifier.clone(),
        }
    }
}

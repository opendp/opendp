use crate::{
    core::Metric,
    metrics::{
        ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
    },
    transformations::DatasetMetric,
};

/// UnorderedMetric is implemented for metrics that compute the distance between datasets in a way that is insensitive to row ordering.
///
/// # Proof Definition
/// For any choice of x and x' where x' = shuffle(x), then $d(x, x') = 0$.
pub trait UnorderedMetric: DatasetMetric<Distance = IntDistance> {
    /// OrderedMetric denotes the associated metric that is sensitive to row ordering.
    ///
    /// # Proof Definition
    /// For any choice of datasets x and x' where d(x, x') > 0 for an OrderedMetric d, and x and x' only differ by the reordering of rows,
    /// then d'(x, x') = 0 for a corresponding UnorderedMetric d'.
    type OrderedMetric: Metric<Distance = Self::Distance>;
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
pub trait OrderedMetric: DatasetMetric<Distance = IntDistance> {
    /// UnorderedMetric denotes the associated metric that is insensitive to row ordering.
    ///
    /// # Proof Definition
    /// For any choice of datasets x, x' where d(x, x') > 0 for an OrderedMetric d and only differs by the reordering of rows,
    /// then d'(x, x') = 0 for a corresponding UnorderedMetric d'.
    type UnorderedMetric: Metric<Distance = Self::Distance>;
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
pub trait BoundedMetric: DatasetMetric<Distance = IntDistance> {
    /// UnboundedMetric denotes the associated metric that measures edit distance.
    ///
    /// # Proof Definition
    /// For any choice of datasets x, x' where d(x, x') = 1 for a BoundedMetric d, then d'(x, x') = 2 for the corresponding UnboundedMetric d'.
    type UnboundedMetric: Metric<Distance = Self::Distance>;
}
impl BoundedMetric for ChangeOneDistance {
    type UnboundedMetric = SymmetricDistance;
}
impl BoundedMetric for HammingDistance {
    type UnboundedMetric = InsertDeleteDistance;
}

/// UnboundedMetric is implemented for metrics that compute the distance between datasets via record additions and removals.
///
/// # Proof Definition
/// For any choice of x and x' where x' differs solely by the addition or removal of one record, then $d(x, x') = 1$.
pub trait UnboundedMetric: DatasetMetric<Distance = IntDistance> {
    /// BoundedMetric denotes the associated metric that measures edit distance.
    ///
    /// # Proof Definition
    /// For any choice of datasets x, x' where d(x, x') = 1 for a BoundedMetric d, then d'(x, x') = 2 for a corresponding UnboundedMetric d'.
    type BoundedMetric: Metric<Distance = Self::Distance>;
}
impl UnboundedMetric for SymmetricDistance {
    type BoundedMetric = ChangeOneDistance;
}
impl UnboundedMetric for InsertDeleteDistance {
    type BoundedMetric = HammingDistance;
}

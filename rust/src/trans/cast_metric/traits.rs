use crate::{
    core::Metric,
    core::{ChangeOneDistance, HammingDistance, InsertDeleteDistance, SymmetricDistance},
};

pub trait UnorderedMetric: Metric {
    type OrderedMetric: Metric<Distance = Self::Distance>;
}
impl UnorderedMetric for SymmetricDistance {
    type OrderedMetric = InsertDeleteDistance;
}
impl UnorderedMetric for ChangeOneDistance {
    type OrderedMetric = HammingDistance;
}

pub trait OrderedMetric: Metric {
    type UnorderedMetric: Metric<Distance = Self::Distance>;
}
impl OrderedMetric for InsertDeleteDistance {
    type UnorderedMetric = SymmetricDistance;
}
impl OrderedMetric for HammingDistance {
    type UnorderedMetric = ChangeOneDistance;
}

pub trait BoundedMetric: Metric {
    type UnboundedMetric: Metric<Distance = Self::Distance>;
}
impl BoundedMetric for ChangeOneDistance {
    type UnboundedMetric = SymmetricDistance;
}
impl BoundedMetric for HammingDistance {
    type UnboundedMetric = InsertDeleteDistance;
}

pub trait UnboundedMetric: Metric {
    type BoundedMetric: Metric<Distance = Self::Distance>;
}
impl UnboundedMetric for SymmetricDistance {
    type BoundedMetric = ChangeOneDistance;
}
impl UnboundedMetric for InsertDeleteDistance {
    type BoundedMetric = HammingDistance;
}

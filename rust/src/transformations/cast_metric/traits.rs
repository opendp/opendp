use crate::{
    core::Metric,
    metrics::{
        ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
    },
    transformations::DatasetMetric,
};

pub trait UnorderedMetric: DatasetMetric<Distance = IntDistance> {
    type OrderedMetric: Metric<Distance = Self::Distance>;
}
impl UnorderedMetric for SymmetricDistance {
    type OrderedMetric = InsertDeleteDistance;
}
impl UnorderedMetric for ChangeOneDistance {
    type OrderedMetric = HammingDistance;
}

pub trait OrderedMetric: DatasetMetric<Distance = IntDistance> {
    type UnorderedMetric: Metric<Distance = Self::Distance>;
}
impl OrderedMetric for InsertDeleteDistance {
    type UnorderedMetric = SymmetricDistance;
}
impl OrderedMetric for HammingDistance {
    type UnorderedMetric = ChangeOneDistance;
}

pub trait BoundedMetric: DatasetMetric<Distance = IntDistance> {
    type UnboundedMetric: Metric<Distance = Self::Distance>;
}
impl BoundedMetric for ChangeOneDistance {
    type UnboundedMetric = SymmetricDistance;
}
impl BoundedMetric for HammingDistance {
    type UnboundedMetric = InsertDeleteDistance;
}

pub trait UnboundedMetric: DatasetMetric<Distance = IntDistance> {
    type BoundedMetric: Metric<Distance = Self::Distance>;
}
impl UnboundedMetric for SymmetricDistance {
    type BoundedMetric = ChangeOneDistance;
}
impl UnboundedMetric for InsertDeleteDistance {
    type BoundedMetric = HammingDistance;
}

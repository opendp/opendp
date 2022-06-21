use crate::{core::{Domain, Metric}, error::Fallible, samplers::Shuffle, dist::{SymmetricDistance, InsertDeleteDistance, ChangeOneDistance, HammingDistance}};

pub trait ShuffleableDomain: Domain {
    fn shuffle(val: Self::Carrier) -> Fallible<Self::Carrier>;
}
impl<D: Domain> ShuffleableDomain for D
where
    D::Carrier: Shuffle,
{
    fn shuffle(mut val: Self::Carrier) -> Fallible<Self::Carrier> {
        val.shuffle()?;
        Ok(val)
    }
}

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

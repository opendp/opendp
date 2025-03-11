use crate::{
    core::Metric,
    metrics::{
        ChangeOneDistance, EventLevelMetric, HammingDistance, InsertDeleteDistance,
        MicrodataMetric, SymmetricDistance,
    },
};

#[cfg(feature = "polars")]
use crate::metrics::{ChangeOneIdDistance, SymmetricIdDistance};

pub trait UnorderedMetric: EventLevelMetric {
    type OrderedMetric: Metric<Distance = Self::Distance> + Default;
}
impl UnorderedMetric for SymmetricDistance {
    type OrderedMetric = InsertDeleteDistance;
}
impl UnorderedMetric for ChangeOneDistance {
    type OrderedMetric = HammingDistance;
}

pub trait OrderedMetric: EventLevelMetric {
    type UnorderedMetric: Metric<Distance = Self::Distance> + Default;
}
impl OrderedMetric for InsertDeleteDistance {
    type UnorderedMetric = SymmetricDistance;
}
impl OrderedMetric for HammingDistance {
    type UnorderedMetric = ChangeOneDistance;
}

pub trait BoundedMetric: MicrodataMetric {
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

pub trait UnboundedMetric: MicrodataMetric {
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

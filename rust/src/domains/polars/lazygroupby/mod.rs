use crate::core::{Domain, MetricSpace};
use crate::domains::LazyFrameDomain;
use crate::error::Fallible;
use crate::metrics::L1;
use crate::transformations::DatasetMetric;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

#[derive(Clone, PartialEq)]
pub struct LazyGroupByDomain {
    pub lazy_frame_domain: LazyFrameDomain,
    pub grouping_columns: Vec<String>,
}

impl<D: DatasetMetric> MetricSpace for (LazyGroupByDomain, L1<D>)
where
    (LazyFrameDomain, D): MetricSpace,
{
    fn check_space(&self) -> Fallible<()> {
        (self.0.lazy_frame_domain.clone(), self.1 .0.clone()).check_space()
    }
}

impl Debug for LazyGroupByDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyGroupByDomain")
            .field("lazy_frame_domain", &self.lazy_frame_domain)
            .field("grouping_columns", &self.grouping_columns)
            .finish()
    }
}

impl Domain for LazyGroupByDomain {
    type Carrier = LazyGroupBy;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let lazy_frame = val.clone().agg([all()]);
        self.lazy_frame_domain.member(&lazy_frame)
    }
}

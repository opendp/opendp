use crate::core::{Domain, MetricSpace};
use crate::domains::LazyFrameDomain;
use crate::error::Fallible;
use crate::metrics::{L1Distance, L1};
use crate::traits::TotalOrd;
use crate::transformations::{item, DatasetMetric};
use polars::prelude::*;

use std::collections::BTreeSet;
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

impl<Q: TotalOrd> MetricSpace for (LazyGroupByDomain, L1Distance<Q>) {
    fn check_space(&self) -> Fallible<()> {
        let margin = (self.0.lazy_frame_domain.margins)
            .get(&BTreeSet::from_iter(self.0.grouping_columns.clone()))
            .ok_or_else(|| err!(MetricSpace, "absolute distance must know dataframe margin"))?;

        let any_neq_1 =
            (margin.data.clone()).select([col(margin.get_count_column_name()?.as_str())
                .neq(lit(1))
                .any(true)]);

        if item::<bool>(any_neq_1)? {
            return fallible!(MetricSpace, "all groups must be of size 1");
        }

        Ok(())
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

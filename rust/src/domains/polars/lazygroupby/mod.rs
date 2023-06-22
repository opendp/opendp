use crate::core::{Domain, MetricSpace};
use crate::domains::{DatasetMetric, LazyFrameDomain};
use crate::error::Fallible;
use crate::metrics::{AbsoluteDistance, Lp, L1};
use polars::prelude::*;
use std::collections::BTreeSet;
use std::fmt::{Debug, Formatter};

#[derive(Clone, PartialEq)]
pub struct LazyGroupByDomain {
    pub lazy_frame_domain: LazyFrameDomain,
    pub grouping_columns: Vec<String>,
}

impl<D: DatasetMetric> MetricSpace for (LazyGroupByDomain, L1<D>) {
    fn check(&self) -> bool {
        true
    }
}

impl<const P: usize, Q> MetricSpace for (LazyGroupByDomain, Lp<P, AbsoluteDistance<Q>>) {
    fn check(&self) -> bool {
        // make sure there is a margin that says the counts are one in each partition
        let lgb_domain = self.0.clone();
        let Some(margin) = lgb_domain.lazy_frame_domain.margins.get(&BTreeSet::from_iter(lgb_domain.grouping_columns)) else {panic!("No margins for grouping columns provided")};
        let Ok(count_name) = margin.get_count_column_name() else {panic!("No count columns name specified")};

        let temp = |lf: LazyFrame| -> Fallible<Vec<u32>> {
            lf.collect()?.get_columns()[0]
                .u32()?
                .to_vec()
                .into_iter()
                .map(|c| c.ok_or_else(|| err!(FailedFunction)))
                .collect()
        };
        let lf = margin.clone().data.select([col(count_name.as_str())]);
        dbg!(lf.clone().collect().unwrap());

        let Ok(counts) = temp(lf) else {panic!("Could not access counts in counts column")};
        counts.into_iter().all(|c| c == 1)
    }
}

impl<Q> MetricSpace for (LazyFrameDomain, AbsoluteDistance<Q>) {
    fn check(&self) -> bool {
        let temp = |lf: LazyFrame| -> Fallible<u32> {
            (lf.collect()?.get_columns()[0].u32()?.get(0)).ok_or_else(|| err!(FailedFunction))
        };

        if let Some(margin) = self.0.margins.get(&BTreeSet::default()) {
            let Ok(count_name) = margin.get_count_column_name() else { panic!("No count columns name specified")};
            let lf = margin.clone().data.select([col(count_name.as_str())]);

            let Ok(count) = temp(lf) else { panic!("Could not access counts in counts column")};
            count == 1
        } else {
            panic!("No counts for empty columns set")
        }
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

use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::collections::BTreeSet;
use std::fmt::{Debug, Formatter};

use crate::core::{Metric, MetricSpace};
use crate::metrics::{
    AbsoluteDistance, ChangeOneDistance, HammingDistance, InsertDeleteDistance, LInfDistance,
    LpDistance, PartitionDistance, SymmetricDistance,
};
use crate::traits::ProductOrd;
use crate::transformations::DatasetMetric;
use crate::{core::Domain, error::Fallible};

use super::{Frame, FrameDomain, LogicalPlanDomain, NumericDataType, SeriesDomain};

#[cfg(feature = "ffi")]
mod ffi;

// TODO: remove this allow marker later
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum ExprContext {
    // select
    Select,
    // filter
    Filter,
    // with_columns
    WithColumns,
    // group_by/agg
    Aggregate(BTreeSet<String>),
}

impl ExprContext {
    fn get_plan(&self, val: &(LogicalPlan, Expr)) -> LogicalPlan {
        let (lp, expr) = val.clone();
        let frame = LazyFrame::from(lp);
        match self {
            ExprContext::Select => frame.select([expr]),
            ExprContext::Filter => frame.filter(expr),
            ExprContext::WithColumns => frame.with_columns([expr]),
            ExprContext::Aggregate(columns) => frame
                .group_by(&columns.iter().map(|n| col(n)).collect::<Vec<_>>())
                .agg([expr]),
        }
        .logical_plan
    }

    pub fn break_alignment(&self) -> Fallible<()> {
        if matches!(self, ExprContext::Filter | ExprContext::WithColumns) {
            return fallible!(
                MakeMeasurement,
                "record alignment can only be broken in a selection or aggregation"
            );
        }
        Ok(())
    }

    pub fn grouping_columns(&self) -> BTreeSet<String> {
        match self {
            ExprContext::Aggregate(columns) => columns.clone(),
            _ => BTreeSet::new(),
        }
    }
}

/// # Proof Definition
/// `ExprDomain` is the domain of data frames that can be constructed by applying a given expression to a given data frame.
///
/// # Example
/// ```
/// use polars::prelude::*;
/// use opendp::domains::{AtomDomain, SeriesDomain, LazyFrameDomain, ExprDomain, ExprContext};
/// let lf_domain = LazyFrameDomain::new(vec![
///     SeriesDomain::new("A", AtomDomain::<i32>::default()),
///     SeriesDomain::new("B", AtomDomain::<f64>::default()),
/// ])?;
///
/// let expr_domain = ExprDomain::new(lf_domain, ExprContext::RowByRow);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq)]
pub struct ExprDomain {
    /// The domain that materialized data frames are a member of.
    pub frame_domain: LogicalPlanDomain,
    /// Denotes an expression must be applied to materialize a member of the domain.
    pub context: ExprContext,
}

impl ExprDomain {
    pub fn new<F: Frame>(frame_domain: FrameDomain<F>, context: ExprContext) -> ExprDomain {
        Self {
            frame_domain: frame_domain.cast_carrier(),
            context,
        }
    }

    pub fn active_series(&self) -> Fallible<&SeriesDomain> {
        self.check_one_column()?;
        Ok(&self.frame_domain.series_domains[0])
    }

    pub fn active_series_mut(&mut self) -> Fallible<&mut SeriesDomain> {
        self.check_one_column()?;
        Ok(&mut self.frame_domain.series_domains[0])
    }

    fn check_one_column(&self) -> Fallible<()> {
        let series_domains = &self.frame_domain.series_domains;
        if series_domains.len() != 1 {
            return fallible!(
                FailedFunction,
                "expression must span exactly one column, but expression spans {} columns",
                series_domains.len()
            );
        }
        Ok(())
    }
}

impl Domain for ExprDomain {
    type Carrier = (LogicalPlan, Expr);

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let frame = self.context.get_plan(val);
        self.frame_domain.member(&frame)
    }
}

impl Debug for ExprDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprDomain")
            .field("lazy_frame_domain", &self.frame_domain)
            .finish()
    }
}

pub trait OuterMetric: 'static + Metric + Send + Sync {
    type InnerMetric: Metric + Send + Sync;
    fn inner_metric(&self) -> Self::InnerMetric;
}

macro_rules! impl_expr_metric_select {
    ($($ty:ty)+) => {$(
        impl OuterMetric for $ty {
            type InnerMetric = Self;

            fn inner_metric(&self) -> Self::InnerMetric {
                self.clone()
            }
        })+
    }
}
impl_expr_metric_select!(InsertDeleteDistance SymmetricDistance HammingDistance ChangeOneDistance);

impl<const P: usize, Q: 'static + Send + Sync> OuterMetric for LpDistance<P, Q> {
    type InnerMetric = Self;
    fn inner_metric(&self) -> Self::InnerMetric {
        self.clone()
    }
}
impl<Q: 'static + Send + Sync> OuterMetric for LInfDistance<Q> {
    type InnerMetric = Self;
    fn inner_metric(&self) -> Self::InnerMetric {
        self.clone()
    }
}
impl<M: 'static + Metric> OuterMetric for PartitionDistance<M> {
    type InnerMetric = M;
    fn inner_metric(&self) -> Self::InnerMetric {
        self.0.clone()
    }
}

impl<M: DatasetMetric> MetricSpace for (ExprDomain, M) {
    fn check_space(&self) -> Fallible<()> {
        (self.0.frame_domain.clone(), self.1.clone()).check_space()
    }
}

impl<Q: ProductOrd, const P: usize> MetricSpace for (ExprDomain, LpDistance<P, Q>) {
    fn check_space(&self) -> Fallible<()> {
        if ![1, 2].contains(&P) {
            return fallible!(MetricSpace, "P must be 1 or 2");
        }
        Ok(())
    }
}

impl<Q: ProductOrd> MetricSpace for (ExprDomain, LInfDistance<Q>) {
    fn check_space(&self) -> Fallible<()> {
        Ok(())
    }
}

impl<M: DatasetMetric> MetricSpace for (ExprDomain, PartitionDistance<M>) {
    fn check_space(&self) -> Fallible<()> {
        (self.0.frame_domain.clone(), self.1 .0.clone()).check_space()
    }
}

impl<Q: ProductOrd + NumericDataType> MetricSpace for (ExprDomain, AbsoluteDistance<Q>) {
    fn check_space(&self) -> Fallible<()> {
        if self.0.active_series()?.field.dtype != Q::dtype() {
            return fallible!(
                MetricSpace,
                "selected column must be of type {}",
                Q::dtype()
            );
        }
        Ok(())
    }
}

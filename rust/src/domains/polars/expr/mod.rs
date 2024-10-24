use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::collections::BTreeSet;
use std::fmt::{Debug, Formatter};

use crate::core::{Metric, MetricSpace};
use crate::metrics::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, LInfDistance, LpDistance, Parallel,
    PartitionDistance, SymmetricDistance,
};
use crate::traits::ProductOrd;
use crate::transformations::DatasetMetric;
use crate::{core::Domain, error::Fallible};

use super::{DslPlanDomain, Frame, FrameDomain, Margin, SeriesDomain};

#[cfg(feature = "ffi")]
mod ffi;

/// The expression context describes how an expression will be applied to a data frame.
///
/// Expressions used in the Polars API fall into four categories:
///
/// 1. Not useful on their own for DP (shift)
/// 2. Must be leaf nodes, like only col or lit (impute, group by or join keys, explode)
/// 3. Must be row-by-row (sorting by, filter, with column, top/bottom k)
/// 4. Aggregates (select, aggregate)
///
/// Specifying the expression context is not necessary for categories one or two, leaving only row-by-row and aggregates.
#[derive(Clone, PartialEq, Debug)]
pub enum ExprContext {
    /// Requires that the expression applied to the data frame is row-by-row, i.e. the expression is applied to each row independently.
    ///
    /// Rows cannot be added or removed, and the order of rows cannot be changed.
    RowByRow,
    /// Allows for aggregation operations that break row alignment, such as `group_by/agg` and `select`.
    ///
    /// `.agg(exprs)` is the general case where there are grouping columns.
    /// `.select(exprs)` is the special case where there are no grouping columns.
    Aggregate { grouping_columns: BTreeSet<String> },
}

impl ExprContext {
    /// # Proof Definition
    /// Given a context (`self`), logical plan (`lp`) and expression,
    /// returns a new logical plan where the expression has been applied to the logical plan by means specified by the context.
    fn get_plan(&self, (lp, expr): &(DslPlan, Expr)) -> DslPlan {
        let frame = LazyFrame::from(lp.clone());
        match self {
            ExprContext::RowByRow => frame.select([expr.clone()]),
            ExprContext::Aggregate { grouping_columns } => frame
                .group_by(
                    &grouping_columns
                        .iter()
                        .map(AsRef::as_ref)
                        .map(col)
                        .collect::<Vec<_>>(),
                )
                .agg([expr.clone()]),
        }
        .logical_plan
    }

    /// # Proof Definition
    /// Return the grouping columns specified by `self` if in an aggregation context,
    /// otherwise return an error.
    pub fn grouping_columns(&self) -> Fallible<BTreeSet<String>> {
        match self {
            // ExprContext::Aggregate serves both `select` and `group_by/agg`
            ExprContext::Aggregate { grouping_columns } => Ok(grouping_columns.clone()),
            ExprContext::RowByRow => {
                fallible!(FailedFunction, "RowByRow context has no grouping columns")
            }
        }
    }

    /// # Proof Definition
    /// Only return without an error if the expression context `self`
    /// allows for row ordering or the number of rows to be changed.
    ///
    /// This is used to disallow operations like sorting, shuffling or filtering
    /// when row alignment must not change,
    /// like when augmenting a new column via `with_column`.
    pub fn break_alignment(&self) -> Fallible<()> {
        if !matches!(self, ExprContext::Aggregate { .. }) {
            return fallible!(
                MakeMeasurement,
                "record alignment can only be broken in a selection or aggregation"
            );
        }
        Ok(())
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
    pub frame_domain: DslPlanDomain,
    /// Denotes how an expression must be applied to materialize a member of the domain.
    pub context: ExprContext,
}

impl ExprDomain {
    /// # Proof Definition
    /// Returns an [`ExprDomain`] with members matching those of `frame_domain`.
    pub fn new<F: Frame>(frame_domain: FrameDomain<F>, context: ExprContext) -> ExprDomain {
        Self {
            frame_domain: frame_domain.cast_carrier(),
            context,
        }
    }

    /// # Proof Definition
    /// If there is only one series in `self`,
    /// returns the series domain for that column,
    /// otherwise an error.
    pub fn active_series(&self) -> Fallible<&SeriesDomain> {
        self.check_one_column()?;
        Ok(&self.frame_domain.series_domains[0])
    }

    /// # Proof Definition
    /// If there is only one series in `self`,
    /// returns a mutable reference to the series domain for that column,
    /// otherwise returns an error.
    pub fn active_series_mut(&mut self) -> Fallible<&mut SeriesDomain> {
        self.check_one_column()?;
        Ok(&mut self.frame_domain.series_domains[0])
    }

    /// # Proof Definition
    /// Returns the margin corresponding to the grouping columns in the context,
    /// otherwise returns an error.
    pub fn active_margin(&self) -> Fallible<&Margin> {
        let grouping_columns = self.context.grouping_columns()?;
        self.frame_domain
            .margins
            .get(&grouping_columns)
            .ok_or_else(|| err!(FailedFunction, "No known margin for {:?}", grouping_columns))
    }

    /// # Proof Definition
    /// Returns Ok if members of `self` have one column, otherwise an error.
    pub fn check_one_column(&self) -> Fallible<()> {
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

#[cfg(test)]
impl<F: Frame> FrameDomain<F> {
    pub fn row_by_row(&self) -> ExprDomain {
        ExprDomain::new(self.clone(), ExprContext::RowByRow)
    }
    pub fn aggregate<E: AsRef<[IE]>, IE: AsRef<str>>(&self, by: E) -> ExprDomain {
        let by = BTreeSet::from_iter(by.as_ref().iter().map(|s| s.as_ref().to_string()));
        ExprDomain::new(
            self.clone(),
            ExprContext::Aggregate {
                grouping_columns: by,
            },
        )
    }
    pub fn select(&self) -> ExprDomain {
        self.aggregate::<_, &str>([])
    }
}

impl Domain for ExprDomain {
    type Carrier = (DslPlan, Expr);

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

/// OuterMetric encodes the relationship between
/// the metric on data that may be grouped vs the metric on individual groups.
pub trait OuterMetric: 'static + Metric + Send + Sync {
    /// # Proof Definition
    /// Type of metric used to measure distances between each group.
    type InnerMetric: Metric + Send + Sync;

    /// # Proof Definition
    /// Returns the inner metric of `self`.
    ///
    /// This is the metric used to measure distances between non-grouped datasets.
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

impl<M: 'static + Metric> OuterMetric for PartitionDistance<M> {
    type InnerMetric = M;
    fn inner_metric(&self) -> Self::InnerMetric {
        self.0.clone()
    }
}

impl<M: DatasetMetric> MetricSpace for (ExprDomain, M) {
    fn check_space(&self) -> Fallible<()> {
        let (expr_domain, metric) = self;
        (expr_domain.frame_domain.clone(), metric.clone()).check_space()
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

impl<Q: ProductOrd> MetricSpace for (ExprDomain, Parallel<LInfDistance<Q>>) {
    fn check_space(&self) -> Fallible<()> {
        let (expr_domain, Parallel(inner_metric)) = self;
        (expr_domain.clone(), inner_metric.clone()).check_space()
    }
}

impl<M: DatasetMetric> MetricSpace for (ExprDomain, PartitionDistance<M>) {
    fn check_space(&self) -> Fallible<()> {
        let (expr_domain, PartitionDistance(inner_metric)) = self;
        (expr_domain.frame_domain.clone(), inner_metric.clone()).check_space()
    }
}

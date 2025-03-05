use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::core::{Metric, MetricSpace};
use crate::metrics::{
    AbsoluteDistance, ChangeOneDistance, HammingDistance, InsertDeleteDistance, LInfDistance,
    LpDistance, Parallel, PartitionDistance, SymmetricDistance,
};
use crate::traits::ProductOrd;
use crate::transformations::DatasetMetric;
use crate::{core::Domain, error::Fallible};

use super::{Frame, FrameDomain, LazyFrameDomain, Margin, SeriesDomain};

#[cfg(feature = "ffi")]
mod ffi;

/// The expression context describes how an expression will be applied to a data frame.
///
/// Expressions used in the Polars API fall into four categories:
///
/// 1. Not useful on their own for DP (shift)
/// 2. Leaf nodes, like only col or lit (impute, group by or join keys, explode)
/// 3. Row-by-row (sorting by, filter, with column, top/bottom k)
/// 4. Grouping (select, aggregate)
///
/// Specifying the expression context is not necessary for categories one or two, leaving only row-by-row and aggregates.
#[derive(Clone, PartialEq, Debug)]
pub enum Context {
    /// Requires that the expression applied to the data frame is row-by-row, i.e. the expression is applied to each row independently.
    ///
    /// Rows cannot be added or removed, and the order of rows cannot be changed.
    RowByRow,
    /// Allows for aggregation operations that break row alignment, such as `agg` and `select`.
    ///
    /// `.agg(exprs)` is the general case where there are grouping columns.
    /// `.select(exprs)` is the special case where there are no grouping columns.
    Aggregation { margin: Margin },
}

impl Context {
    /// # Proof Definition
    /// Return the grouping columns and margin specified by `self` if in an aggregation context,
    /// otherwise return an error.
    pub fn aggregation(&self, operation: &str) -> Fallible<Margin> {
        match self {
            Context::RowByRow { .. } => fallible!(
                MakeDomain,
                "{} is not allowed in a row-by-row context",
                operation
            ),
            Context::Aggregation { margin } => Ok(margin.clone()),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WildExprDomain {
    /// Domains for each column.
    pub columns: Vec<SeriesDomain>,
    /// The context in which a frame resides.
    pub context: Context,
}

impl Domain for WildExprDomain {
    type Carrier = DslPlan;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.clone()
            .to_frame_domain()?
            .member(&LazyFrame::from(val.clone()))
    }
}

impl WildExprDomain {
    pub fn as_row_by_row(&self) -> Self {
        Self {
            columns: self.columns.clone(),
            context: Context::RowByRow,
        }
    }

    fn to_frame_domain<F: Frame>(self) -> Fallible<FrameDomain<F>> {
        FrameDomain::new_with_margins(
            self.columns,
            match self.context {
                Context::RowByRow => Vec::new(),
                Context::Aggregation { margin } => {
                    vec![margin]
                }
            },
        )
    }
}

/// # Proof Definition
/// `ExprDomain` is the domain of series that can be constructed by applying an expression to a data frame.
#[derive(Clone, PartialEq, Debug)]
pub struct ExprDomain {
    /// The domain that materialized data frames are a member of.
    pub column: SeriesDomain,
    /// Context-specific descriptors.
    pub context: Context,
}

impl LazyFrameDomain {
    pub fn select(self) -> WildExprDomain {
        self.aggregate::<Expr, 0>([])
    }

    pub fn aggregate<S: Into<Expr>, const P: usize>(self, by: [S; P]) -> WildExprDomain {
        let by = by.map(|s| s.into()).into();
        let margin = self.get_margin(&by);
        WildExprDomain {
            columns: self.series_domains,
            context: Context::Aggregation { margin },
        }
    }

    pub fn row_by_row(self) -> WildExprDomain {
        WildExprDomain {
            columns: self.series_domains,
            context: Context::RowByRow,
        }
    }
}

#[derive(Clone)]
pub struct ExprPlan {
    pub plan: DslPlan,
    pub expr: Expr,
    pub fill: Option<Expr>,
}

impl Debug for ExprPlan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprPlan")
            .field("expr", &self.expr)
            .field("default", &self.fill.is_some())
            .finish()
    }
}

impl ExprPlan {
    /// # Proof Definition
    /// Return a compute plan where the expression and fill expression in `self` are extended by `function`.
    pub fn then(&self, function: impl Fn(Expr) -> Expr) -> Self {
        Self {
            plan: self.plan.clone(),
            expr: function(self.expr.clone()),
            fill: self.fill.clone().map(function),
        }
    }
}

impl From<DslPlan> for ExprPlan {
    fn from(value: DslPlan) -> Self {
        ExprPlan {
            plan: value,
            expr: all(),
            fill: None,
        }
    }
}

impl From<LazyFrame> for ExprPlan {
    fn from(value: LazyFrame) -> Self {
        ExprPlan::from(value.logical_plan)
    }
}

impl Domain for ExprDomain {
    type Carrier = ExprPlan;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let (plan, expr) = (LazyFrame::from(val.plan.clone()), val.expr.clone());
        let frame = match &self.context {
            Context::RowByRow { .. } => plan.select([expr]),
            Context::Aggregation { margin } => plan
                .group_by(margin.by.iter().cloned().collect::<Vec<_>>())
                .agg([expr.clone()]),
        }
        .collect()?;

        let series = frame.column(&self.column.name)?.as_materialized_series();
        if !(self.column).member(series)? {
            return Ok(false);
        }

        match &self.context {
            Context::RowByRow => (),
            Context::Aggregation { margin } => {
                if !margin.member(frame.lazy().group_by(&Vec::from_iter(margin.by.clone())))? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
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

impl<M: 'static + Metric> OuterMetric for Parallel<M> {
    type InnerMetric = M;

    fn inner_metric(&self) -> Self::InnerMetric {
        self.0.clone()
    }
}

impl<const P: usize, Q: 'static> OuterMetric for LpDistance<P, Q> {
    type InnerMetric = AbsoluteDistance<Q>;

    fn inner_metric(&self) -> Self::InnerMetric {
        AbsoluteDistance::default()
    }
}

impl<M: DatasetMetric> MetricSpace for (WildExprDomain, M) {
    fn check_space(&self) -> Fallible<()> {
        let (expr_domain, metric) = self;
        (
            expr_domain.clone().to_frame_domain::<DslPlan>()?,
            metric.clone(),
        )
            .check_space()
    }
}

impl<M: DatasetMetric> MetricSpace for (WildExprDomain, PartitionDistance<M>) {
    fn check_space(&self) -> Fallible<()> {
        let (expr_domain, PartitionDistance(inner_metric)) = self;
        (
            expr_domain.clone().to_frame_domain::<DslPlan>()?,
            inner_metric.clone(),
        )
            .check_space()
    }
}

impl<M: DatasetMetric> MetricSpace for (ExprDomain, M) {
    fn check_space(&self) -> Fallible<()> {
        Ok(())
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
        Ok(())
    }
}

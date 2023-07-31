use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::core::{Metric, MetricSpace};
use crate::domains::{DatasetMetric, LazyFrameDomain};
use crate::metrics::{L1Distance, Lp, AbsoluteDistance};
use crate::traits::TotalOrd;
use crate::{core::Domain, error::Fallible};

use super::SeriesDomain;

// TODO: remove this allow marker later
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum LazyFrameContext {
    Select,
    Filter,
    WithColumns,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LazyGroupByContext {
    pub columns: Vec<String>,
}

pub trait Context: PartialEq + Clone {
    const GROUPBY: bool;
    type Value: Clone;
    fn get_frame(&self, val: &(Self::Value, Expr)) -> LazyFrame;
    fn grouping_columns(&self) -> Vec<String>;
}

impl Context for LazyFrameContext {
    const GROUPBY: bool = false;
    type Value = LazyFrame;
    fn get_frame(&self, val: &(Self::Value, Expr)) -> LazyFrame {
        let (frame, expr) = val.clone();
        match self {
            LazyFrameContext::Select => frame.select([expr]),
            LazyFrameContext::Filter => frame.filter(expr),
            LazyFrameContext::WithColumns => frame.with_columns([expr]),
        }
    }
    fn grouping_columns(&self) -> Vec<String> {
        vec![]
    }
}
impl Context for LazyGroupByContext {
    const GROUPBY: bool = true;
    type Value = LazyGroupBy;
    fn get_frame(&self, val: &(Self::Value, Expr)) -> LazyFrame {
        let (grouped, expr) = val.clone();
        grouped.agg([expr])
    }
    fn grouping_columns(&self) -> Vec<String> {
        self.columns.clone()
    }
}

/// # Proof Definition
/// `ExprDomain(C)` is the domain of all polars expressions that can be applied to a `LazyFrame` where:
/// * `lazy_frame_domain` - `LazyFrameDomain`.
/// * `context` - Context in which expression is to be applied.
/// * `active_column` - Column to which expression is to be applied.
/// * `aligned` - `true` if the expression preserves order and number of rows, `false` otherwise.
///
/// ## Generics
/// * `C` - Context: `LazyFrameContext`, `LazyGroupByContext`.
///
/// # Example
/// ```
/// use polars::prelude::*;
/// use opendp::domains::{AtomDomain, SeriesDomain, LazyFrameDomain, ExprDomain, LazyFrameContext};
/// let lazy_frame_domain = LazyFrameDomain::new(vec![
///             SeriesDomain::new("A", AtomDomain::<i32>::default()),
///             SeriesDomain::new("B", AtomDomain::<f64>::default()),
/// ])?;
///
/// let expr_domain = ExprDomain::new(lazy_frame_domain, LazyFrameContext::Select, Some("A".to_string()), true);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq)]
pub struct ExprDomain<C: Context> {
    pub lazy_frame_domain: LazyFrameDomain,
    pub context: C,
    pub active_column: Option<String>,
    pub aligned: bool,
}

impl<C: Context> ExprDomain<C> {
    pub fn new(
        lazy_frame_domain: LazyFrameDomain,
        context: C,
        active_column: Option<String>,
        aligned: bool,
    ) -> ExprDomain<C> {
        Self {
            lazy_frame_domain,
            context,
            active_column,
            aligned,
        }
    }

    pub fn active_series(&self) -> Fallible<&SeriesDomain> {
        match &self.active_column {
            Some(column) => self.lazy_frame_domain.try_column(column),
            None => fallible!(FailedFunction, "no active column"),
        }
    }

    pub fn active_column(&self) -> Fallible<String> {
        return self
            .active_column
            .clone()
            .ok_or_else(|| err!(FailedFunction, "active column not set"));
    }
}

impl<C: Context> Domain for ExprDomain<C> {
    type Carrier = (C::Value, Expr);

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let frame = self.context.get_frame(val);
        self.lazy_frame_domain.member(&frame)
    }
}

impl<C: Context> Debug for ExprDomain<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprDomain")
            .field("lazy_frame_domain", &self.lazy_frame_domain)
            .finish()
    }
}

pub trait ExprMetric<C>: Metric {
    type InnerMetric: Metric<Distance = Self::Distance>;
    fn inner_metric(&self) -> Self::InnerMetric;
}

impl<M: Metric> ExprMetric<LazyFrameContext> for M {
    type InnerMetric = Self;

    fn inner_metric(&self) -> Self::InnerMetric {
        self.clone()
    }
}

impl<M: Metric> ExprMetric<LazyGroupByContext> for Lp<1, M> {
    type InnerMetric = M;

    fn inner_metric(&self) -> Self::InnerMetric {
        self.0.clone()
    }
}

impl<M: DatasetMetric> MetricSpace for (ExprDomain<LazyFrameContext>, M) {
    fn check(&self) -> bool {
        if M::BOUNDED {
            return (self.0.lazy_frame_domain.margins.iter())
                .find(|(_, margin)| margin.counts_index.is_some())
                .is_some();
        };

        true
    }
}

impl<M: DatasetMetric, const P: usize> MetricSpace for (ExprDomain<LazyGroupByContext>, Lp<P, M>) {
    fn check(&self) -> bool {
        if M::BOUNDED {
            return (self.0.lazy_frame_domain.margins.iter())
                .find(|(_, margin)| margin.counts_index.is_some())
                .is_some();
        };

        true
    }
}

impl<T: TotalOrd> MetricSpace for (ExprDomain<LazyGroupByContext>, L1Distance<T>) {
    fn check(&self) -> bool {
        true
    }
}

impl<T: TotalOrd> MetricSpace for (ExprDomain<LazyFrameContext>, AbsoluteDistance<T>) {
    fn check(&self) -> bool {
        true
    }
}

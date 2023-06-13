use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::core::{Metric, MetricSpace};
use crate::domains::{NumericDataType, DatasetMetric, LazyFrameDomain};
use crate::metrics::{AbsoluteDistance, L1Distance, Lp, InsertDeleteDistance, SymmetricDistance, HammingDistance, ChangeOneDistance};
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

pub trait Context: 'static + PartialEq + Clone + Send + Sync {
    const GROUPBY: bool;
    type Value: 'static + Clone + Send + Sync;
    fn get_frame(&self, val: &(Arc<Self::Value>, Expr)) -> LazyFrame;
    fn grouping_columns(&self) -> Vec<String>;
}

impl Context for LazyFrameContext {
    const GROUPBY: bool = false;
    type Value = LazyFrame;
    fn get_frame(&self, val: &(Arc<Self::Value>, Expr)) -> LazyFrame {
        let frame = (*val.0).clone();
        let expr = val.1.clone();
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
    fn get_frame(&self, val: &(Arc<Self::Value>, Expr)) -> LazyFrame {
        let (grouped, expr) = val.clone();
        (*grouped).clone().agg([expr])
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
    pub row_by_row: bool,
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
            row_by_row: aligned,
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
    type Carrier = (Arc<C::Value>, Expr);

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

pub trait ExprMetric: 'static + Metric + Send + Sync {
    type InnerMetric: Metric<Distance = Self::Distance>;
    type Context: Context;
    fn inner_metric(&self) -> Self::InnerMetric;
}



macro_rules! impl_expr_metric_select {
    ($($ty:ty)+) => {$(
        impl ExprMetric for $ty {
            type InnerMetric = Self;
            type Context = LazyFrameContext;
        
            fn inner_metric(&self) -> Self::InnerMetric {
                self.clone()
            }
        })+
    }
}
impl_expr_metric_select!(InsertDeleteDistance SymmetricDistance HammingDistance ChangeOneDistance);

impl<Q: 'static + Send + Sync> ExprMetric for AbsoluteDistance<Q> {
    type InnerMetric = Self;
    type Context = LazyFrameContext;

    fn inner_metric(&self) -> Self::InnerMetric {
        self.clone()
    }
}
impl<M: 'static + Metric + Send + Sync> ExprMetric for Lp<1, M> {
    type InnerMetric = M;
    type Context = LazyGroupByContext;

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

// it is valid to pair L1Distance with ExprDomain when the selected column is the same type as T
impl<T: TotalOrd + NumericDataType, C: Context> MetricSpace for (ExprDomain<C>, L1Distance<T>) {
    fn check(&self) -> bool {
        let active_column = if let Some(active_column) = &self.0.active_column {
            active_column
        } else {
            return true;
        };

        let series_domain =
            if let Some(series_domain) = self.0.lazy_frame_domain.column(active_column) {
                series_domain
            } else {
                return false;
            };

        series_domain.field.dtype == T::numeric_dtype()
    }
}

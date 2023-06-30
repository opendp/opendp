use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::core::{Metric, MetricSpace};
use crate::domains::{DatasetMetric, LazyFrameDomain, NumericDataType};
use crate::metrics::{
    AbsoluteDistance, ChangeOneDistance, HammingDistance, InsertDeleteDistance, L1Distance, Lp,
    SymmetricDistance, LInfDiffDistance, L1,
};
use crate::traits::TotalOrd;
use crate::{core::Domain, error::Fallible};

use super::{SeriesDomain, LazyGroupByDomain};

#[cfg(feature = "ffi")]
mod ffi;

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
    fn break_alignment(&self) -> Fallible<()>;
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
    fn break_alignment(&self) -> Fallible<()> {
        match self {
            LazyFrameContext::Select => Ok(()),
            LazyFrameContext::Filter => fallible!(
                MakeTransformation,
                "cannot break alignment in filter context"
            ),
            LazyFrameContext::WithColumns => fallible!(
                MakeTransformation,
                "cannot break alignment in select context"
            ),
        }
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
    fn break_alignment(&self) -> Fallible<()> {
        Ok(())
    }
}

/// # Proof Definition
/// `ExprDomain(C)` is the domain of all polars expressions that can be applied to a `LazyFrame` where:
/// * `lazy_frame_domain` - `LazyFrameDomain`.
/// * `context` - Context in which expression is to be applied.
/// * `active_column` - Column to which expression is to be applied.
/// * `row_by_row` - `true` if the expression preserves order and number of rows, `false` otherwise.
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
/// let expr_domain = ExprDomain::<LazyFrameDomain>::new(lazy_frame_domain, LazyFrameContext::Select, Some("A".to_string()));
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq)]
pub struct ExprDomain<D: LazyDomain> {
    pub lazy_frame_domain: LazyFrameDomain,
    pub context: D::Context,
    pub active_column: Option<String>,
}

pub trait LazyDomain: Domain + Send + Sync {
    type Context: Context<Value = Self::Carrier>;
}

impl LazyDomain for LazyFrameDomain {
    type Context = LazyFrameContext;
}

impl LazyDomain for LazyGroupByDomain {
    type Context = LazyGroupByContext;
}

impl<D: LazyDomain> ExprDomain<D> {
    pub fn new(
        lazy_frame_domain: LazyFrameDomain,
        context: D::Context,
        active_column: Option<String>,
    ) -> ExprDomain<D> {
        Self {
            lazy_frame_domain,
            context,
            active_column,
        }
    }

    pub fn active_series(&self) -> Fallible<&SeriesDomain> {
        self.lazy_frame_domain.try_column(self.active_column()?)
    }

    pub fn active_column(&self) -> Fallible<String> {
        return self
            .active_column
            .clone()
            .ok_or_else(|| err!(FailedFunction, "active column not set. Use `make_col(col_name)` first."))
    }
}

impl<D: LazyDomain> Domain for ExprDomain<D> {
    type Carrier = (Arc<D::Carrier>, Expr);

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let frame = self.context.get_frame(val);
        self.lazy_frame_domain.member(&frame)
    }
}

impl<D: LazyDomain> Debug for ExprDomain<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprDomain")
            .field("lazy_frame_domain", &self.lazy_frame_domain)
            .finish()
    }
}

pub trait OuterMetric: 'static + Metric + Send + Sync {
    type InnerMetric: Metric<Distance = Self::Distance> + Send + Sync;
    type LazyDomain: LazyDomain;
    fn inner_metric(&self) -> Self::InnerMetric;
}

macro_rules! impl_expr_metric_select {
    ($($ty:ty)+) => {$(
        impl OuterMetric for $ty {
            type InnerMetric = Self;
            type LazyDomain = LazyFrameDomain;

            fn inner_metric(&self) -> Self::InnerMetric {
                self.clone()
            }
        })+
    }
}
impl_expr_metric_select!(InsertDeleteDistance SymmetricDistance HammingDistance ChangeOneDistance);

impl<Q: 'static + Send + Sync> OuterMetric for AbsoluteDistance<Q> {
    type InnerMetric = Self;
    type LazyDomain = LazyFrameDomain;
    fn inner_metric(&self) -> Self::InnerMetric {
        self.clone()
    }
}

impl<Q: 'static + Send + Sync> OuterMetric for LInfDiffDistance<Q> {
    type InnerMetric = Self;
    type LazyDomain = LazyFrameDomain;
    fn inner_metric(&self) -> Self::InnerMetric {
        self.clone()
    }
}

impl<M: 'static + Metric + Send + Sync> OuterMetric for Lp<1, M> {
    type InnerMetric = M;
    type LazyDomain = LazyGroupByDomain;
    fn inner_metric(&self) -> Self::InnerMetric {
        self.0.clone()
    }
}

impl<M: DatasetMetric> MetricSpace for (ExprDomain<LazyFrameDomain>, M) {
    fn check(&self) -> bool {
        if M::BOUNDED {
            return (self.0.lazy_frame_domain.margins.iter())
                .find(|(_, margin)| margin.counts_index.is_some())
                .is_some();
        };

        true
    }
}

impl<M: DatasetMetric, const P: usize> MetricSpace for (ExprDomain<LazyGroupByDomain>, Lp<P, M>) {
    fn check(&self) -> bool {
        if M::BOUNDED {
            return (self.0.lazy_frame_domain.margins.iter())
                .find(|(_, margin)| margin.counts_index.is_some())
                .is_some();
        };

        true
    }
}


impl<Q: TotalOrd> MetricSpace for (ExprDomain<LazyFrameDomain>, AbsoluteDistance<Q>) {
    fn check(&self) -> bool {
        (self.0.lazy_frame_domain.clone(), self.1.clone()).check()
    }
}
impl<Q> MetricSpace for (ExprDomain<LazyGroupByDomain>, L1<LInfDiffDistance<Q>>) 
    where (LazyGroupByDomain, L1<LInfDiffDistance<Q>>): MetricSpace {
    fn check(&self) -> bool {
        let lgb_domain = LazyGroupByDomain {
            lazy_frame_domain: self.0.lazy_frame_domain.clone(),
            grouping_columns: self.0.context.columns.clone()
        };

        (lgb_domain, self.1.clone()).check()
    }
}

impl<Q> MetricSpace for (ExprDomain<LazyFrameDomain>, LInfDiffDistance<Q>)
    where (LazyFrameDomain, LInfDiffDistance<Q>): MetricSpace {
    fn check(&self) -> bool {
        (self.0.lazy_frame_domain.clone(), self.1.clone()).check()
    }
}

impl<Q> MetricSpace for (LazyFrameDomain, LInfDiffDistance<Q>) {
    fn check(&self) -> bool {
        true
    }
}

impl<Q> MetricSpace for (LazyGroupByDomain, L1<LInfDiffDistance<Q>>) {
    fn check(&self) -> bool {
        true
    }
}

// it is valid to pair L1Distance with ExprDomain when the selected column is the same type as T
impl<T: TotalOrd + NumericDataType> MetricSpace for (ExprDomain<LazyGroupByDomain>, L1Distance<T>) {
    fn check(&self) -> bool {
        (self.0.active_series())
            .map(|s| s.field.dtype == T::dtype())
            .unwrap_or(false)
    }
}

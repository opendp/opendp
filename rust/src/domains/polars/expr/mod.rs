use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::core::{Metric, MetricSpace};
use crate::domains::{DatasetMetric, LazyFrameDomain};
use crate::metrics::Lp;
use crate::{core::Domain, error::Fallible};

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
}
impl Context for LazyGroupByContext {
    const GROUPBY: bool = true;
    type Value = LazyGroupBy;
    fn get_frame(&self, val: &(Self::Value, Expr)) -> LazyFrame {
        let (grouped, expr) = val.clone();
        grouped.agg([expr])
    }
}

#[derive(Clone, PartialEq)]
pub struct ExprDomain<C: Context> {
    pub lazy_frame_domain: LazyFrameDomain,
    pub context: C,
    pub active_column: Option<String>,
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
    type OuterMetric: Metric<Distance = Self::Distance>;
}

impl<M: Metric> ExprMetric<LazyFrameContext> for M {
    type OuterMetric = Self;
}

impl<M: Metric> ExprMetric<LazyGroupByContext> for M {
    type OuterMetric = Lp<1, Self>;
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

use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::core::MetricSpace;
use crate::domains::{DatasetMetric, LazyFrameDomain};
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
    columns: Vec<String>,
}

pub trait Context: PartialEq + Clone {
    type Value;
    fn get_frame(&self, val: &(Self::Value, Expr)) -> LazyFrame;
}

impl Context for LazyFrameContext {
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

impl<D: DatasetMetric, C: Context> MetricSpace for (ExprDomain<C>, D) {
    fn check(&self) -> bool {
        if D::BOUNDED {
            let margins = self.0.lazy_frame_domain.margins.clone();
            return if margins.is_empty() {
                false
            } else {
                let margins_with_counts: Vec<_> = margins
                    .iter()
                    .filter(|(_, margin)| margin.get_count_column_name().is_ok())
                    .collect();
                !margins_with_counts.is_empty()
            };
        }
        return true;
    }
}

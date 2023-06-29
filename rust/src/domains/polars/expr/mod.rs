use polars::lazy::dsl::Expr;
use polars::prelude::*;
use std::fmt::{Debug, Formatter};

use crate::core::MetricSpace;
use crate::domains::{DatasetMetric, LazyFrameDomain};
use crate::{core::Domain, error::Fallible};

#[derive(Clone, PartialEq)]
pub enum LazyFrameContext {
    Select,
    Filter,
    WithColumns,
}

#[derive(Clone, PartialEq)]
pub struct LazyGroupByContext {
    columns: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub struct ExprDomain<T> {
    pub lazy_frame_domain: LazyFrameDomain,
    pub context: T,
    pub active_column: Option<String>,
}

impl Domain for ExprDomain<LazyFrameContext> {
    type Carrier = (LazyFrame, Expr);

    fn member(&self, (val_lazy_frame, val_expr): &Self::Carrier) -> Fallible<bool> {
        let expr = val_expr.clone();
        let lazy_frame = val_lazy_frame.clone();

        let result_frame = match self.context {
            LazyFrameContext::Select => lazy_frame.select([expr]),
            LazyFrameContext::Filter => lazy_frame.filter(expr),
            LazyFrameContext::WithColumns => lazy_frame.with_columns([expr]),
        };
        self.lazy_frame_domain.member(&result_frame)
    }
}

impl Domain for ExprDomain<LazyGroupByContext> {
    type Carrier = (LazyGroupBy, Expr);

    fn member(&self, (val_lazy_groupby, val_expr): &Self::Carrier) -> Fallible<bool> {
        let expr = val_expr.clone();
        let result_frame = val_lazy_groupby.clone().agg([expr]);
        self.lazy_frame_domain.member(&result_frame)
    }
}

impl<T> Debug for ExprDomain<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprDomain")
            .field("lazy_frame_domain", &self.lazy_frame_domain)
            .finish()
    }
}

impl<D: DatasetMetric, T> MetricSpace for (ExprDomain<T>, D) {
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

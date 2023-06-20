use polars::lazy::dsl::Expr;
use polars::prelude::{col, LazyFrame};

use crate::core::MetricSpace;
use crate::domains::{DatasetMetric, LazyFrameDomain};
use crate::{core::Domain, error::Fallible};

#[derive(Clone, PartialEq)]
pub enum Context {
    Select,
    Agg { columns: Vec<String> },
    Filter,
    WithColumns,
}

#[derive(Clone, PartialEq)]
pub struct ExprDomain {
    pub lazy_frame_domain: LazyFrameDomain,
    pub context: Context,
    pub active_column: Option<String>,
}

impl Domain for ExprDomain {
    type Carrier = (LazyFrame, Expr);

    fn member(&self, (val_lazy_frame, val_expr): &Self::Carrier) -> Fallible<bool> {
        let expr = val_expr.clone();
        let lazy_frame = val_lazy_frame.clone();

        let result_frame = match self.context {
            Context::Select => lazy_frame.select([expr]),
            Context::Agg { ref columns } => {
                let column_exprs: Vec<_> = columns.iter().map(|c| col(c.as_ref())).collect();
                lazy_frame.groupby_stable(column_exprs).agg([expr])
            }
            Context::Filter => lazy_frame.filter(expr),
            Context::WithColumns => lazy_frame.with_columns([expr]),
        };
        self.lazy_frame_domain.member(&result_frame)
    }
}

impl std::fmt::Debug for ExprDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprDomain")
            .field("lazy_frame_domain", &self.lazy_frame_domain)
            .finish()
    }
}

impl<D: DatasetMetric> MetricSpace for (ExprDomain, D) {
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

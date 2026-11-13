//! A backend-neutral, bound computation. Plans never contain private rows.
//!
//! FrameDomain<VLazyFrame<S>> constrains the *realized collection*, exactly
//! like FrameDomain<VTable>. A source binding must denote one immutable logical
//! snapshot, even when the backend defers actually reading the source.

use polars::prelude::PlSmallStr;

use crate::{
    core::Domain,
    domains::{ColumnDomain, Frame, FrameDomain},
    error::*,
    verified_query::{self as verified, VExpr, VPlan, VTable},
};

use super::verified_frame::VColumn;

/// A source handle that denotes one stable table state. Production handles
/// can be opaque: only explicit realization/membership asks for its contents.
/// Implementations must not silently rebind the handle to changing data.
pub(crate) trait VSource: Clone + Send + Sync + 'static {
    fn snapshot(&self) -> Fallible<VTable>;
}

/// Reference source used in Verus-backed tests. The table is held *outside*
/// the plan, and no copy is made while constructing lazy operators.
impl VSource for VTable {
    fn snapshot(&self) -> Fallible<VTable> {
        Ok(self.clone())
    }
}

#[derive(Clone)]
pub(crate) struct VLazyFrame<S: VSource> {
    pub plan: VPlan,
    pub source: S,
}

impl<S: VSource> VLazyFrame<S> {
    pub fn new(source: S) -> Self {
        Self { plan: VPlan::Input, source }
    }

    /// Explicit reference realization. Transformation construction and
    /// invocation never call this; an execution backend can instead lower
    /// the same plan and source handle without materializing them here.
    pub fn collect(&self) -> Fallible<VTable> {
        let input = self.source.snapshot()?;
        if !verified::check_table(&input) {
            return fallible!(FailedFunction, "malformed verified source table");
        }
        let schema = verified::check_plan(&self.plan, &input.schema)
            .ok_or_else(|| err!(FailedFunction, "ill-typed verified plan"))?;
        let rows = verified::evaluate(&self.plan, &input)
            .map_err(|_| err!(FailedFunction, "verified plan evaluation failed"))?;
        Ok(VTable { schema, rows })
    }
}

impl<S: VSource> Frame for VLazyFrame<S> {
    type Column = VColumn;
    type Expr = VExpr;

    fn root_names(
        expr: &VExpr,
        columns: &[ColumnDomain<VColumn>],
    ) -> Fallible<Vec<PlSmallStr>> {
        <VTable as Frame>::root_names(expr, columns)
    }

    fn frame_member(&self, domain: &FrameDomain<Self>) -> Fallible<bool> {
        // Describe the realized table, never the plan's syntactic shape.
        let realized = self.collect()?;
        domain.cast_carrier::<VTable>().member(&realized)
    }
}

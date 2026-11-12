//! Polars backend: lower the normalized DSL to a native Polars plan.
use super::verified;
use crate::error::*;
use polars::prelude::*;
use polars_plan::prelude::ProjectionOptions;
use std::sync::Arc;

/// Produce Polars expressions only from the normalized AST.
fn render_expr(expr: &verified::VExpr, columns: &[PlSmallStr]) -> Fallible<Expr> {
    use verified::{VDType, VValue};
    Ok(match expr {
        verified::VExpr::Column(index) => {
            let column = columns.get(*index).ok_or_else(||
                err!(MakeTransformation, "verified column index outside schema")
            )?;
            col(column.as_str())
        },
        verified::VExpr::Alias { expr, name } =>
            render_expr(expr, columns)?.alias(name.as_str()),
        verified::VExpr::Literal(VValue::Int(v)) => lit(*v),
        verified::VExpr::Literal(VValue::Bool(v)) => lit(*v),
        verified::VExpr::Null(VDType::Int64) => lit(NULL).cast(DataType::Int64),
        verified::VExpr::Null(VDType::Boolean) => lit(NULL).cast(DataType::Boolean),
        verified::VExpr::Binary { left, op, right } => {
            let lhs = render_expr(left, columns)?;
            let rhs = render_expr(right, columns)?;
            match op {
                verified::VBinaryOp::Lt => lhs.lt(rhs),
                verified::VBinaryOp::Gt => lhs.gt(rhs),
                verified::VBinaryOp::Eq => lhs.eq(rhs),
                verified::VBinaryOp::And => lhs.and(rhs),
            }
        },
        verified::VExpr::Clip { expr, lower, upper } => {
            let value = render_expr(expr, columns)?;
            let low = lower.as_ref().map(|expr| render_expr(expr, columns)).transpose()?;
            let high = upper.as_ref().map(|expr| render_expr(expr, columns)).transpose()?;
            // Implement the DSL's total clip semantics. Polars' native clip can
            // panic for reversed bounds, whereas the lower bound wins here.
            match (low, high) {
                (Some(low), Some(high)) =>
                    when(value.clone().lt(low.clone())).then(low)
                        .when(value.clone().gt(high.clone())).then(high)
                        .otherwise(value),
                (Some(low), None) =>
                    when(value.clone().lt(low.clone())).then(low).otherwise(value),
                (None, Some(high)) =>
                    when(value.clone().gt(high.clone())).then(high).otherwise(value),
                (None, None) => value,
            }
        },
    })
}

/// Reconstruct the supported query against the caller's data source.
pub(super) fn render_plan(
    plan: &verified::VPlan, source: DslPlan, source_columns: &[PlSmallStr],
) -> Fallible<(DslPlan, Vec<PlSmallStr>)> {
    match plan {
        verified::VPlan::Input => Ok((source, source_columns.to_vec())),
        verified::VPlan::Project { input, exprs } => {
            let (child, input_columns) = render_plan(input, source, source_columns)?;
            let mut expressions = Vec::with_capacity(exprs.len());
            let mut names = Vec::with_capacity(exprs.len());
            for node in exprs {
                let verified::VExpr::Alias { name, .. } = node else {
                    return fallible!(MakeTransformation, "verified projection requires named expressions");
                };
                expressions.push(render_expr(node, &input_columns)?);
                names.push(PlSmallStr::from(name.as_str()));
            }
            Ok((DslPlan::Select {
                input: Arc::new(child),
                expr: expressions,
                options: ProjectionOptions {
                    should_broadcast: true,
                    ..ProjectionOptions::default()
                },
            }, names))
        },
        verified::VPlan::Filter { input, predicate } => {
            let (child, columns) = render_plan(input, source, source_columns)?;
            Ok((DslPlan::Filter {
                input: Arc::new(child),
                predicate: render_expr(predicate, &columns)?,
            }, columns))
        },
    }
}


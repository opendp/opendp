//! Polars frontend: resolve supported Polars queries into the backend-neutral DSL.
use super::verified;
use crate::domains::verified_frame::kernel_schema;
use crate::{domains::{LazyFrameDomain, SeriesDomain}, error::*};
use polars::prelude::*;
use polars_plan::{dsl::{FunctionExpr, Operator}, utils::expr_output_name};

fn broad_column(dtype: DataType, nullable: bool) -> Fallible<SeriesDomain> {
    let mut domain = SeriesDomain::new_from_field(Field::new("_".into(), dtype))?;
    domain.nullable = nullable;
    Ok(domain)
}

fn inferred_field(expr: &verified::VExpr, columns: &[SeriesDomain])
    -> Fallible<verified::VField>
{
    verified::check_expr(expr, &kernel_schema(columns)?)
        .ok_or_else(|| err!(MakeTransformation, "ill-typed verified expression"))
}

fn output_column(
    expr: &verified::VExpr, field: verified::VField, columns: &[SeriesDomain],
) -> Fallible<SeriesDomain> {
    match expr {
        verified::VExpr::Alias { expr, .. } => return output_column(expr, field, columns),
        verified::VExpr::Column(index) => return Ok(columns[*index].clone()),
        _ => {},
    }
    let dtype = match field.dtype {
        verified::VDType::Int64 => DataType::Int64,
        verified::VDType::Boolean => DataType::Boolean,
    };
    broad_column(dtype, field.nullable)
}

fn normalize_expr(expr: &Expr, columns: &[SeriesDomain]) -> Fallible<verified::VExpr> {
    use verified::VValue;
    match expr {
        Expr::Alias(inner, name) => Ok(verified::VExpr::Alias {
            expr: Box::new(normalize_expr(inner, columns)?),
            name: name.to_string(),
        }),
        Expr::Column(name) => {
            let index = columns.iter().position(|c| c.name == *name)
                .ok_or_else(|| err!(MakeTransformation, "unknown column: {}", name))?;
            Ok(verified::VExpr::Column(index))
        },
        Expr::Literal(literal) => {
            let value = literal.to_any_value().ok_or_else(||
                err!(MakeTransformation, "unsupported literal in verified query")
            )?;
            Ok(verified::VExpr::Literal(match value {
                AnyValue::Int64(v) => VValue::Int(v),
                // Small dynamic integer literals materialize as Int32 in Polars.
                AnyValue::Int32(v)
                    if matches!(literal.get_datatype(), DataType::Unknown(_)) =>
                        VValue::Int(v.into()),
                AnyValue::Boolean(v) => VValue::Bool(v),
                _ => return fallible!(MakeTransformation,
                    "verified queries require Int64 or Boolean literals"),
            }))
        },
        Expr::BinaryExpr { left, op, right } => {
            let op = match op {
                Operator::Lt => verified::VBinaryOp::Lt,
                Operator::Gt => verified::VBinaryOp::Gt,
                Operator::Eq => verified::VBinaryOp::Eq,
                Operator::And | Operator::LogicalAnd => verified::VBinaryOp::And,
                _ => return fallible!(MakeTransformation,
                    "unsupported verified binary operation: {:?}", op),
            };
            Ok(verified::VExpr::Binary {
                left: Box::new(normalize_expr(left, columns)?),
                op,
                right: Box::new(normalize_expr(right, columns)?),
            })
        },
        Expr::Function {
            input,
            function: FunctionExpr::Clip { has_min: true, has_max: true },
            ..
        } if input.len() == 3 => Ok(verified::VExpr::Clip {
            expr: Box::new(normalize_expr(&input[0], columns)?),
            lower: Some(Box::new(normalize_expr(&input[1], columns)?)),
            upper: Some(Box::new(normalize_expr(&input[2], columns)?)),
        }),
        _ => fallible!(MakeTransformation,
            "unsupported or non-row-local expression in verified query"),
    }
}

fn reads_column(expr: &verified::VExpr) -> bool {
    match expr {
        verified::VExpr::Column(_) => true,
        verified::VExpr::Alias { expr, .. } => reads_column(expr),
        verified::VExpr::Literal(_) | verified::VExpr::Null(_) => false,
        verified::VExpr::Binary { left, right, .. } =>
            reads_column(left) || reads_column(right),
        verified::VExpr::Clip { expr, lower, upper } =>
            reads_column(expr)
                || lower.as_ref().is_some_and(|e| reads_column(e))
                || upper.as_ref().is_some_and(|e| reads_column(e)),
    }
}

/// Validate every relational node without reading source data.
pub(super) fn normalize_plan(
    plan: &DslPlan, input_columns: &[SeriesDomain],
) -> Fallible<(verified::VPlan, Vec<SeriesDomain>)> {
    match plan {
        DslPlan::IR { dsl, .. } => normalize_plan(dsl, input_columns),
        DslPlan::DataFrameScan { .. } => {
            kernel_schema(input_columns)?;
            Ok((verified::VPlan::Input, input_columns.to_vec()))
        },
        DslPlan::Select { input, expr, options } => {
            let (child, columns) = normalize_plan(input, input_columns)?;
            if !options.should_broadcast {
                return fallible!(MakeTransformation,
                    "verified select requires scalar broadcasting");
            }
            let mut nodes = Vec::new();
            let mut output_columns = Vec::new();
            let mut row_dependent = false;
            for item in expr {
                let name = expr_output_name(item)?;
                let inner = normalize_expr(item, &columns)?;
                let node = match inner {
                    verified::VExpr::Alias { .. } => inner,
                    _ => verified::VExpr::Alias {
                        expr: Box::new(inner), name: name.to_string(),
                    },
                };
                let field = inferred_field(&node, &columns)?;
                let mut domain = output_column(&node, field, &columns)?;
                row_dependent |= reads_column(&node);
                domain.name = name;
                nodes.push(node);
                output_columns.push(domain);
            }
            if !row_dependent {
                return fallible!(MakeTransformation,
                    "verified select requires a row-dependent expression");
            }
            LazyFrameDomain::new(output_columns.clone())?;
            Ok((verified::VPlan::Project {
                input: Box::new(child), exprs: nodes,
            }, output_columns))
        },
        DslPlan::Filter { input, predicate } => {
            let (child, columns) = normalize_plan(input, input_columns)?;
            let node = normalize_expr(predicate, &columns)?;
            if inferred_field(&node, &columns)?.dtype != verified::VDType::Boolean {
                return fallible!(MakeTransformation, "filter predicate must be Boolean");
            }
            Ok((verified::VPlan::Filter {
                input: Box::new(child), predicate: node,
            }, columns))
        },
        _ => fallible!(MakeTransformation, "unsupported verified relational operation"),
    }
}


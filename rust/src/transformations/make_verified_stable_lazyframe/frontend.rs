//! Polars frontend: resolve supported Polars queries into the backend-neutral DSL.
use super::verified;
use crate::{domains::{LazyFrameDomain, SeriesDomain}, error::*};
use polars::prelude::*;
use polars_plan::{dsl::{FunctionExpr, Operator}, utils::expr_output_name};

fn broad_column(dtype: DataType, nullable: bool) -> Fallible<SeriesDomain> {
    let mut domain = SeriesDomain::new_from_field(Field::new("_".into(), dtype))?;
    domain.nullable = nullable;
    Ok(domain)
}

pub(super) fn kernel_schema(columns: &[SeriesDomain]) -> Fallible<Vec<verified::Field>> {
    columns.iter().map(|column| {
        let dtype = match column.dtype() {
            DataType::Int64 => verified::DType::Int64,
            DataType::Boolean => verified::DType::Boolean,
            dtype => return fallible!(MakeTransformation,
                "unsupported verified column dtype: {:?}", dtype),
        };
        Ok(verified::Field { dtype, nullable: column.nullable })
    }).collect()
}

fn inferred_field(expr: &verified::Expr, columns: &[SeriesDomain])
    -> Fallible<verified::Field>
{
    verified::check_expr(expr, &kernel_schema(columns)?)
        .ok_or_else(|| err!(MakeTransformation, "ill-typed verified expression"))
}

fn output_column(
    expr: &verified::Expr, field: verified::Field, columns: &[SeriesDomain],
) -> Fallible<SeriesDomain> {
    match expr {
        verified::Expr::Alias { expr, .. } => return output_column(expr, field, columns),
        verified::Expr::Column(index) => return Ok(columns[*index].clone()),
        _ => {},
    }
    let dtype = match field.dtype {
        verified::DType::Int64 => DataType::Int64,
        verified::DType::Boolean => DataType::Boolean,
    };
    broad_column(dtype, field.nullable)
}

fn normalize_expr(expr: &Expr, columns: &[SeriesDomain]) -> Fallible<verified::Expr> {
    use verified::Value;
    match expr {
        Expr::Alias(inner, name) => Ok(verified::Expr::Alias {
            expr: Box::new(normalize_expr(inner, columns)?),
            name: name.to_string(),
        }),
        Expr::Column(name) => {
            let index = columns.iter().position(|c| c.name == *name)
                .ok_or_else(|| err!(MakeTransformation, "unknown column: {}", name))?;
            Ok(verified::Expr::Column(index))
        },
        Expr::Literal(literal) => {
            let value = literal.to_any_value().ok_or_else(||
                err!(MakeTransformation, "unsupported literal in verified query")
            )?;
            Ok(verified::Expr::Literal(match value {
                AnyValue::Int64(v) => Value::Int(v),
                // Small dynamic integer literals materialize as Int32 in Polars.
                AnyValue::Int32(v)
                    if matches!(literal.get_datatype(), DataType::Unknown(_)) =>
                        Value::Int(v.into()),
                AnyValue::Boolean(v) => Value::Bool(v),
                _ => return fallible!(MakeTransformation,
                    "verified queries require Int64 or Boolean literals"),
            }))
        },
        Expr::BinaryExpr { left, op, right } => {
            let op = match op {
                Operator::Lt => verified::BinaryOp::Lt,
                Operator::Gt => verified::BinaryOp::Gt,
                Operator::Eq => verified::BinaryOp::Eq,
                Operator::And | Operator::LogicalAnd => verified::BinaryOp::And,
                _ => return fallible!(MakeTransformation,
                    "unsupported verified binary operation: {:?}", op),
            };
            Ok(verified::Expr::Binary {
                left: Box::new(normalize_expr(left, columns)?),
                op,
                right: Box::new(normalize_expr(right, columns)?),
            })
        },
        Expr::Function {
            input,
            function: FunctionExpr::Clip { has_min: true, has_max: true },
            ..
        } if input.len() == 3 => Ok(verified::Expr::Clip {
            expr: Box::new(normalize_expr(&input[0], columns)?),
            lower: Some(Box::new(normalize_expr(&input[1], columns)?)),
            upper: Some(Box::new(normalize_expr(&input[2], columns)?)),
        }),
        _ => fallible!(MakeTransformation,
            "unsupported or non-row-local expression in verified query"),
    }
}

fn reads_column(expr: &verified::Expr) -> bool {
    match expr {
        verified::Expr::Column(_) => true,
        verified::Expr::Alias { expr, .. } => reads_column(expr),
        verified::Expr::Literal(_) | verified::Expr::Null(_) => false,
        verified::Expr::Binary { left, right, .. } =>
            reads_column(left) || reads_column(right),
        verified::Expr::Clip { expr, lower, upper } =>
            reads_column(expr)
                || lower.as_ref().is_some_and(|e| reads_column(e))
                || upper.as_ref().is_some_and(|e| reads_column(e)),
    }
}

/// Validate every relational node without reading source data.
pub(super) fn normalize_plan(
    plan: &DslPlan, input_columns: &[SeriesDomain],
) -> Fallible<(verified::Plan, Vec<SeriesDomain>)> {
    match plan {
        DslPlan::IR { dsl, .. } => normalize_plan(dsl, input_columns),
        DslPlan::DataFrameScan { .. } => {
            kernel_schema(input_columns)?;
            Ok((verified::Plan::Input, input_columns.to_vec()))
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
                    verified::Expr::Alias { .. } => inner,
                    _ => verified::Expr::Alias {
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
            Ok((verified::Plan::Project {
                input: Box::new(child), exprs: nodes,
            }, output_columns))
        },
        DslPlan::Filter { input, predicate } => {
            let (child, columns) = normalize_plan(input, input_columns)?;
            let node = normalize_expr(predicate, &columns)?;
            if inferred_field(&node, &columns)?.dtype != verified::DType::Boolean {
                return fallible!(MakeTransformation, "filter predicate must be Boolean");
            }
            Ok((verified::Plan::Filter {
                input: Box::new(child), predicate: node,
            }, columns))
        },
        _ => fallible!(MakeTransformation, "unsupported verified relational operation"),
    }
}


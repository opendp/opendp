//! Private bridge from supported Polars queries to the verified row kernel.
//! The Verus proofs cover the normalized IR, not Polars execution itself.

use std::{collections::HashSet, sync::Arc};

#[path = "../../../query_kernel/src/row.rs"]
mod verified;
use polars::prelude::*;
use polars_plan::{dsl::{FunctionExpr, Operator}, prelude::ProjectionOptions, utils::expr_output_name};

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{LazyFrameDomain, SeriesDomain},
    error::*,
    metrics::{Bounds, FrameDistance, SymmetricDistance},
};

/// Metadata not present in the verified, name-free AST.
enum Step {
    Project {
        input_columns: Vec<SeriesDomain>,
        names: Vec<PlSmallStr>,
        options: ProjectionOptions,
    },
    Filter { input_columns: Vec<SeriesDomain> },
}

fn broad_column(dtype: DataType, nullable: bool) -> Fallible<SeriesDomain> {
    let mut domain = SeriesDomain::new_from_field(Field::new("_".into(), dtype))?;
    domain.nullable = nullable;
    Ok(domain)
}

fn normalize_expr(
    expr: &Expr, columns: &[SeriesDomain],
) -> Fallible<(verified::Expr, SeriesDomain)> {
    use verified::Value;
    match expr {
        Expr::Alias(inner, name) => {
            let (node, mut domain) = normalize_expr(inner, columns)?;
            domain.name = name.clone();
            Ok((node, domain))
        },
        Expr::Column(name) => {
            let index = columns.iter().position(|c| c.name == *name)
                .ok_or_else(|| err!(MakeTransformation, "unknown column: {}", name))?;
            Ok((verified::Expr::Column(index), columns[index].clone()))
        },
        Expr::Literal(literal) => {
            let value = literal.to_any_value().ok_or_else(||
                err!(MakeTransformation, "unsupported literal in verified query")
            )?;
            match value {
                AnyValue::Int64(v) => Ok((
                    verified::Expr::Literal(Value::Int(v)),
                    broad_column(DataType::Int64, false)?,
                )),
                AnyValue::Boolean(v) => Ok((
                    verified::Expr::Literal(Value::Bool(v)),
                    broad_column(DataType::Boolean, false)?,
                )),
                _ => fallible!(MakeTransformation,
                    "verified queries require Int64 or Boolean literals"),
            }
        },
        Expr::BinaryExpr { left, op, right } => {
            let (lhs, left_domain) = normalize_expr(left, columns)?;
            let (rhs, right_domain) = normalize_expr(right, columns)?;
            let op = match op {
                Operator::Lt => verified::BinaryOp::Lt,
                Operator::Gt => verified::BinaryOp::Gt,
                Operator::Eq => verified::BinaryOp::Eq,
                Operator::And | Operator::LogicalAnd => verified::BinaryOp::And,
                _ => return fallible!(MakeTransformation,
                    "unsupported verified binary operation: {:?}", op),
            };
            let left_dtype = left_domain.dtype();
            let right_dtype = right_domain.dtype();
            let valid = match op {
                verified::BinaryOp::Lt | verified::BinaryOp::Gt =>
                    left_dtype == DataType::Int64 && right_dtype == DataType::Int64,
                verified::BinaryOp::Eq => left_dtype == right_dtype,
                verified::BinaryOp::And =>
                    left_dtype == DataType::Boolean && right_dtype == DataType::Boolean,
            };
            if !valid {
                return fallible!(MakeTransformation,
                    "incompatible verified operand dtypes: {:?}, {:?}",
                    left_dtype, right_dtype);
            }
            Ok((
                verified::Expr::Binary {
                    left: Box::new(lhs), op, right: Box::new(rhs),
                },
                broad_column(
                    DataType::Boolean, left_domain.nullable || right_domain.nullable
                )?,
            ))
        },
        Expr::Function {
            input,
            function: FunctionExpr::Clip { has_min: true, has_max: true },
            ..
        } if input.len() == 3 => {
            let (value, value_domain) = normalize_expr(&input[0], columns)?;
            let (lower, lower_domain) = normalize_expr(&input[1], columns)?;
            let (upper, upper_domain) = normalize_expr(&input[2], columns)?;
            if value_domain.dtype() != DataType::Int64
                || lower_domain.dtype() != DataType::Int64
                || upper_domain.dtype() != DataType::Int64
            {
                return fallible!(MakeTransformation, "verified clip requires Int64");
            }
            Ok((
                verified::Expr::Clip {
                    expr: Box::new(value),
                    lower: Some(Box::new(lower)),
                    upper: Some(Box::new(upper)),
                },
                broad_column(DataType::Int64, value_domain.nullable)?,
            ))
        },
        _ => fallible!(MakeTransformation,
            "unsupported or non-row-local expression in verified query"),
    }
}

fn reads_column(expr: &verified::Expr) -> bool {
    match expr {
        verified::Expr::Column(_) => true,
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
fn normalize_plan(
    plan: &DslPlan, input_columns: &[SeriesDomain],
) -> Fallible<(verified::Plan, Vec<Step>, Vec<SeriesDomain>)> {
    match plan {
        DslPlan::IR { dsl, .. } => normalize_plan(dsl, input_columns),
        DslPlan::DataFrameScan { .. } =>
            Ok((verified::Plan::Input, Vec::new(), input_columns.to_vec())),
        DslPlan::Select { input, expr, options } => {
            let (child, mut steps, columns) = normalize_plan(input, input_columns)?;
            let mut nodes = Vec::new();
            let mut output_columns = Vec::new();
            let mut names = Vec::new();
            let mut row_dependent = false;
            for item in expr {
                let (node, mut domain) = normalize_expr(item, &columns)?;
                row_dependent |= reads_column(&node);
                let name = expr_output_name(item)?;
                domain.name = name.clone();
                nodes.push(node);
                names.push(name);
                output_columns.push(domain);
            }
            if !row_dependent {
                return fallible!(MakeTransformation,
                    "verified select requires a row-dependent expression");
            }
            LazyFrameDomain::new(output_columns.clone())?;
            steps.push(Step::Project {
                input_columns: columns,
                names,
                options: options.clone(),
            });
            Ok((verified::Plan::Project {
                input: Box::new(child), exprs: nodes,
            }, steps, output_columns))
        },
        DslPlan::Filter { input, predicate } => {
            let (child, mut steps, columns) = normalize_plan(input, input_columns)?;
            let (node, domain) = normalize_expr(predicate, &columns)?;
            if domain.dtype() != DataType::Boolean {
                return fallible!(MakeTransformation, "filter predicate must be Boolean");
            }
            steps.push(Step::Filter { input_columns: columns.clone() });
            Ok((verified::Plan::Filter {
                input: Box::new(child), predicate: node,
            }, steps, columns))
        },
        _ => fallible!(MakeTransformation, "unsupported verified relational operation"),
    }
}

/// Produce Polars expressions only from the normalized AST.
fn render_expr(expr: &verified::Expr, columns: &[SeriesDomain]) -> Fallible<Expr> {
    use verified::{DType, Value};
    Ok(match expr {
        verified::Expr::Column(index) => {
            let column = columns.get(*index).ok_or_else(||
                err!(MakeTransformation, "verified column index outside schema")
            )?;
            col(column.name.as_str())
        },
        verified::Expr::Literal(Value::Int(v)) => lit(*v),
        verified::Expr::Literal(Value::Bool(v)) => lit(*v),
        verified::Expr::Null(DType::Int64) => lit(NULL).cast(DataType::Int64),
        verified::Expr::Null(DType::Boolean) => lit(NULL).cast(DataType::Boolean),
        verified::Expr::Binary { left, op, right } => {
            let lhs = render_expr(left, columns)?;
            let rhs = render_expr(right, columns)?;
            match op {
                verified::BinaryOp::Lt => lhs.lt(rhs),
                verified::BinaryOp::Gt => lhs.gt(rhs),
                verified::BinaryOp::Eq => lhs.eq(rhs),
                verified::BinaryOp::And => lhs.and(rhs),
            }
        },
        verified::Expr::Clip { expr, lower, upper } => {
            let Some(lo) = lower else {
                return fallible!(MakeTransformation, "lower clip bound required");
            };
            let Some(hi) = upper else {
                return fallible!(MakeTransformation, "upper clip bound required");
            };
            render_expr(expr, columns)?.clip(
                render_expr(lo, columns)?,
                render_expr(hi, columns)?,
            )
        },
    })
}

/// Substitute the caller's data source instead of capturing the original one.
fn render_plan(
    plan: &verified::Plan,
    source: DslPlan,
    steps: &[Step],
    cursor: &mut usize,
) -> Fallible<DslPlan> {
    match plan {
        verified::Plan::Input => Ok(source),
        verified::Plan::Project { input, exprs } => {
            let child = render_plan(input, source, steps, cursor)?;
            let Some(Step::Project { input_columns, names, options }) = steps.get(*cursor) else {
                return fallible!(MakeTransformation, "verified plan metadata mismatch");
            };
            *cursor += 1;
            if names.len() != exprs.len() {
                return fallible!(MakeTransformation, "verified projection width mismatch");
            }
            let expr = exprs.iter().zip(names)
                .map(|(expr, name)| render_expr(expr, input_columns)
                    .map(|expr| expr.alias(name.as_str())))
                .collect::<Fallible<Vec<_>>>()?;
            Ok(DslPlan::Select {
                input: Arc::new(child),
                expr,
                options: options.clone(),
            })
        },
        verified::Plan::Filter { input, predicate } => {
            let child = render_plan(input, source, steps, cursor)?;
            let Some(Step::Filter { input_columns }) = steps.get(*cursor) else {
                return fallible!(MakeTransformation, "verified plan metadata mismatch");
            };
            *cursor += 1;
            Ok(DslPlan::Filter {
                input: Arc::new(child),
                predicate: render_expr(predicate, input_columns)?,
            })
        },
    }
}

/// Experimental private constructor. Unsupported plans fail closed.
pub(crate) fn make_verified_stable_lazyframe(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance<SymmetricDistance>,
    lazyframe: LazyFrame,
) -> Fallible<Transformation<
    LazyFrameDomain, FrameDistance<SymmetricDistance>,
    LazyFrameDomain, FrameDistance<SymmetricDistance>,
>> {
    let (plan, steps, columns) =
        normalize_plan(&lazyframe.logical_plan, &input_domain.series_domains)?;
    // Conservatively discard margins and grouped distance bounds.
    let output_domain = LazyFrameDomain::new(columns)?;
    Transformation::new(
        input_domain,
        input_metric.clone(),
        output_domain,
        input_metric,
        Function::new_fallible(move |input: &LazyFrame| {
            let mut cursor = 0;
            let output = render_plan(
                &plan, input.logical_plan.clone(), &steps, &mut cursor)?;
            if cursor != steps.len() {
                return fallible!(MakeTransformation, "verified plan metadata mismatch");
            }
            Ok(LazyFrame::from(output)
                .with_optimizations(input.get_current_optimizations()))
        }),
        StabilityMap::new(|bounds: &Bounds| {
            Bounds(vec![bounds.get_bound(&HashSet::new())])
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::{AtomDomain, OptionDomain};

    fn domain() -> Fallible<LazyFrameDomain> {
        LazyFrameDomain::new(vec![
            SeriesDomain::new("x", OptionDomain::new(AtomDomain::<i64>::default())),
            SeriesDomain::new("lower", AtomDomain::<i64>::default()),
            SeriesDomain::new("upper", AtomDomain::<i64>::default()),
            SeriesDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])
    }

    #[test]
    fn projects_and_filters_a_different_input() -> Fallible<()> {
        let first = df!(
            "x" => [Some(2i64), None],
            "lower" => [0i64, 0],
            "upper" => [5i64, 5],
            "flag" => [Some(true), None],
        )?.lazy();
        let query = first.clone()
            .filter(col("flag").eq(lit(true)))
            .select([
                col("x").clip(col("lower"), col("upper")).alias("clipped"),
                col("flag"),
            ]);
        let trans = make_verified_stable_lazyframe(
            domain()?, FrameDistance(SymmetricDistance), query)?;
        let second = df!(
            "x" => [Some(-2i64), Some(8), None],
            "lower" => [0i64, 1, 0],
            "upper" => [5i64, 6, 5],
            "flag" => [Some(true), Some(true), None],
        )?.lazy();
        let actual = trans.invoke(&second)?.collect()?;
        let expected = second
            .filter(col("flag").eq(lit(true)))
            .select([
                col("x").clip(col("lower"), col("upper")).alias("clipped"),
                col("flag"),
            ]).collect()?;
        assert_eq!(actual, expected);
        assert_eq!(trans.output_domain.series_domains.len(), 2);
        assert_eq!(trans.output_domain.series_domains[0].name.as_str(), "clipped");
        assert_eq!(
            trans.map(&Bounds::from(2))?.get_bound(&HashSet::new()).per_group,
            Some(2),
        );
        Ok(())
    }

    #[test]
    fn rejects_unsupported_plans() -> Fallible<()> {
        let lf = df!(
            "x" => [Some(2i64), Some(3i64)],
            "lower" => [0i64, 0],
            "upper" => [5i64, 5],
            "flag" => [Some(true), Some(false)],
        )?.lazy();
        let metric = FrameDistance(SymmetricDistance);
        assert!(make_verified_stable_lazyframe(
            domain()?, metric.clone(), lf.clone().select([lit(5i64)]),
        ).is_err());
        assert!(make_verified_stable_lazyframe(
            domain()?, metric.clone(), lf.clone().select([col("x").sum()]),
        ).is_err());
        assert!(make_verified_stable_lazyframe(
            domain()?, metric.clone(), lf.clone().filter(col("x")),
        ).is_err());
        assert!(make_verified_stable_lazyframe(
            domain()?, metric, lf.sort(["x"], SortMultipleOptions::default()),
        ).is_err());
        Ok(())
    }
}

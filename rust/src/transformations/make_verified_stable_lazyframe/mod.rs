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

/// Operator metadata deliberately excluded from the name-free verified AST.
enum PlanMetadata {
    Input,
    Project {
        input: Box<PlanMetadata>,
        input_columns: Vec<SeriesDomain>,
        names: Vec<PlSmallStr>,
        options: ProjectionOptions,
    },
    Filter {
        input: Box<PlanMetadata>,
        input_columns: Vec<SeriesDomain>,
    },
}

fn broad_column(dtype: DataType, nullable: bool) -> Fallible<SeriesDomain> {
    let mut domain = SeriesDomain::new_from_field(Field::new("_".into(), dtype))?;
    domain.nullable = nullable;
    Ok(domain)
}

fn kernel_schema(columns: &[SeriesDomain]) -> Fallible<Vec<verified::Field>> {
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
    if let verified::Expr::Column(index) = expr {
        return Ok(columns[*index].clone());
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
        Expr::Alias(inner, _) => normalize_expr(inner, columns),
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
) -> Fallible<(verified::Plan, PlanMetadata, Vec<SeriesDomain>)> {
    match plan {
        DslPlan::IR { dsl, .. } => normalize_plan(dsl, input_columns),
        DslPlan::DataFrameScan { .. } => {
            kernel_schema(input_columns)?;
            Ok((verified::Plan::Input, PlanMetadata::Input, input_columns.to_vec()))
        },
        DslPlan::Select { input, expr, options } => {
            let (child, child_meta, columns) = normalize_plan(input, input_columns)?;
            if !options.should_broadcast {
                return fallible!(MakeTransformation,
                    "verified select requires scalar broadcasting");
            }
            let mut nodes = Vec::new();
            let mut output_columns = Vec::new();
            let mut names = Vec::new();
            let mut row_dependent = false;
            for item in expr {
                let node = normalize_expr(item, &columns)?;
                let field = inferred_field(&node, &columns)?;
                let mut domain = output_column(&node, field, &columns)?;
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
            let metadata = PlanMetadata::Project {
                input: Box::new(child_meta),
                input_columns: columns,
                names,
                options: options.clone(),
            };
            Ok((verified::Plan::Project {
                input: Box::new(child), exprs: nodes,
            }, metadata, output_columns))
        },
        DslPlan::Filter { input, predicate } => {
            let (child, child_meta, columns) = normalize_plan(input, input_columns)?;
            let node = normalize_expr(predicate, &columns)?;
            if inferred_field(&node, &columns)?.dtype != verified::DType::Boolean {
                return fallible!(MakeTransformation, "filter predicate must be Boolean");
            }
            let metadata = PlanMetadata::Filter {
                input: Box::new(child_meta),
                input_columns: columns.clone(),
            };
            Ok((verified::Plan::Filter {
                input: Box::new(child), predicate: node,
            }, metadata, columns))
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
    plan: &verified::Plan, metadata: &PlanMetadata, source: DslPlan,
) -> Fallible<DslPlan> {
    match (plan, metadata) {
        (verified::Plan::Input, PlanMetadata::Input) => Ok(source),
        (verified::Plan::Project { input, exprs },
            PlanMetadata::Project { input: child_meta, input_columns, names, options }) => {
            let child = render_plan(input, child_meta, source)?;
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
        (verified::Plan::Filter { input, predicate },
            PlanMetadata::Filter { input: child_meta, input_columns }) => {
            let child = render_plan(input, child_meta, source)?;
            Ok(DslPlan::Filter {
                input: Arc::new(child),
                predicate: render_expr(predicate, input_columns)?,
            })
        },
        _ => fallible!(MakeTransformation, "verified plan metadata mismatch"),
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
    let (plan, metadata, columns) =
        normalize_plan(&lazyframe.logical_plan, &input_domain.series_domains)?;
    // Conservatively discard margins and grouped distance bounds.
    let output_domain = LazyFrameDomain::new(columns)?;
    Transformation::new(
        input_domain,
        input_metric.clone(),
        output_domain,
        input_metric,
        Function::new_fallible(move |input: &LazyFrame| {
            let output = render_plan(&plan, &metadata, input.logical_plan.clone())?;
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
    fn rejects_unmodeled_columns() -> Fallible<()> {
        let lf = df!(
            "text" => ["a", "b"],
            "x" => [1i64, 2],
        )?.lazy();
        let domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("text", AtomDomain::<String>::default()),
            SeriesDomain::new("x", AtomDomain::<i64>::default()),
        ])?;
        assert!(make_verified_stable_lazyframe(
            domain,
            FrameDistance(SymmetricDistance),
            lf.select([col("x")]),
        ).is_err());
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

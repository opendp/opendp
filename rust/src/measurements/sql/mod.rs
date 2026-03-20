use std::{collections::HashMap, sync::Arc};

use opendp_derive::bootstrap;
use polars::{
    error::PolarsResult,
    prelude::{DslPlan, LazyFrame, LazySerde, PlSmallStr, SpecialEq, col, lit},
    sql::{FunctionRegistry, SQLContext},
};
use polars_plan::dsl::{Expr, UserDefinedFunction};
use sqlparser::{
    ast::{
        Expr as SqlExpr, Function, FunctionArg, FunctionArgExpr, FunctionArgumentList,
        FunctionArguments, GroupByExpr, Ident, ObjectName, Query, Select, SelectItem, SetExpr,
        Statement, Value,
    },
    dialect::GenericDialect,
    parser::Parser,
};

use crate::{
    error::Fallible,
    measurements::{
        expr_dp_counting_query::{DPCountShim, DPLenShim, DPNUniqueShim, DPNullCountShim},
        expr_dp_frame_len::DPFrameLenShim,
        expr_dp_mean::DPMeanShim,
        expr_dp_median::DPMedianShim,
        expr_dp_quantile::DPQuantileShim,
        expr_dp_sum::DPSumShim,
        expr_index_candidates::IndexCandidatesShim,
        expr_noise::NoiseShim,
        expr_noisy_max::NoisyMaxShim,
    },
    polars::OpenDPPlugin,
    transformations::expr_discrete_quantile_score::DiscreteQuantileScoreShim,
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

/// Translate a SQL query into a lazyframe plan.
///
/// # Arguments
/// * `query` - The sql query.
/// * `tables` - Hashmap of tables involved in query.
#[bootstrap(arguments(tables(c_type = "AnyObject *")))]
pub fn sql_to_plan(query: String, tables: HashMap<String, LazyFrame>) -> Fallible<LazyFrame> {
    let registry = ODPFunctionRegistry::new()?;
    let mut context = SQLContext::new().with_function_registry(Arc::new(registry));
    tables.into_iter().for_each(|(name, table)| {
        context.register(name.as_str(), table);
    });
    let rewritten = rewrite_grouped_opendp_aggregates(&query)?;
    let query = rewritten
        .as_ref()
        .map(|(query, _)| query.as_str())
        .unwrap_or(query.as_str());
    let lf = context.execute(query).map_err(crate::error::Error::from)?;
    let Some((_, replacements)) = rewritten else {
        return Ok(lf);
    };

    let optimizations = lf.get_current_optimizations();
    let logical_plan = replace_grouped_aggregates(lf.logical_plan, &replacements);
    Ok(LazyFrame::from(logical_plan).with_optimizations(optimizations))
}

#[derive(Clone)]
struct GroupedAggregateReplacement {
    alias: String,
    expr: Expr,
}

fn rewrite_grouped_opendp_aggregates(
    query: &str,
) -> Fallible<Option<(String, Vec<GroupedAggregateReplacement>)>> {
    let dialect = GenericDialect {};
    let mut statements =
        Parser::parse_sql(&dialect, query).map_err(|e| err!(FailedFunction, "{}", e))?;

    if statements.len() != 1 {
        return Ok(None);
    }
    let Some(Statement::Query(query)) = statements.get_mut(0) else {
        return Ok(None);
    };

    let Some(select) = extract_select_mut(query) else {
        return Ok(None);
    };
    let has_group_by = match &select.group_by {
        GroupByExpr::All(_) => true,
        GroupByExpr::Expressions(exprs, _) => !exprs.is_empty(),
    };
    if !has_group_by {
        return Ok(None);
    }

    let mut replacements = Vec::new();
    for (idx, item) in select.projection.iter_mut().enumerate() {
        let (expr, alias) = match item {
            SelectItem::UnnamedExpr(expr) => (expr, None),
            SelectItem::ExprWithAlias { expr, alias } => (expr, Some(alias.clone())),
            _ => continue,
        };

        let Some((replacement_expr, placeholder_alias)) =
            grouped_opendp_sql_expr(expr, alias.as_ref(), idx)?
        else {
            continue;
        };

        replacements.push(GroupedAggregateReplacement {
            alias: placeholder_alias.value.clone(),
            expr: replacement_expr,
        });

        *item = SelectItem::ExprWithAlias {
            expr: count_star(),
            alias: placeholder_alias,
        };
    }

    if replacements.is_empty() {
        return Ok(None);
    }

    Ok(Some((statements[0].to_string(), replacements)))
}

fn extract_select_mut(query: &mut Query) -> Option<&mut Select> {
    let SetExpr::Select(select) = query.body.as_mut() else {
        return None;
    };
    Some(select.as_mut())
}

fn grouped_opendp_sql_expr(
    expr: &SqlExpr,
    alias: Option<&Ident>,
    idx: usize,
) -> Fallible<Option<(Expr, Ident)>> {
    let SqlExpr::Function(function) = expr else {
        return Ok(None);
    };

    let Some(name) = function_name(function) else {
        return Ok(None);
    };

    let expr = match name.as_str() {
        "dp_sum" => build_dp_sum_expr(function)?,
        "dp_mean" => build_dp_mean_expr(function)?,
        "dp_quantile" => build_dp_quantile_expr(function)?,
        "dp_median" => build_dp_median_expr(function)?,
        "dp_len" => build_dp_len_expr(function)?,
        "dp_count" => build_dp_count_expr(function)?,
        "dp_null_count" => build_dp_null_count_expr(function)?,
        "dp_n_unique" => build_dp_n_unique_expr(function)?,
        _ => return Ok(None),
    };

    let alias = alias
        .cloned()
        .unwrap_or_else(|| Ident::new(format!("__opendp_sql_agg_{idx}")));

    Ok(Some((expr.alias(alias.value.as_str()), alias)))
}

fn function_name(function: &Function) -> Option<String> {
    let ident = function.name.0.last()?.as_ident()?;
    Some(ident.value.to_ascii_lowercase())
}

fn count_star() -> SqlExpr {
    SqlExpr::Function(Function {
        name: ObjectName::from(vec![Ident::new("COUNT")]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args: vec![FunctionArg::Unnamed(FunctionArgExpr::Wildcard)],
            clauses: vec![],
        }),
        filter: None,
        null_treatment: None,
        over: None,
        within_group: vec![],
    })
}

fn build_dp_sum_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 4, DPSumShim)
}

fn build_dp_mean_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 4, DPMeanShim)
}

fn build_dp_quantile_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 4, DPQuantileShim)
}

fn build_dp_median_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 3, DPMedianShim)
}

fn build_dp_len_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 2, DPLenShim)
}

fn build_dp_count_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 2, DPCountShim)
}

fn build_dp_null_count_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 2, DPNullCountShim)
}

fn build_dp_n_unique_expr(function: &Function) -> Fallible<Expr> {
    build_plugin_expr(function, 2, DPNUniqueShim)
}

fn build_plugin_expr<P: OpenDPPlugin>(
    function: &Function,
    max_args: usize,
    plugin: P,
) -> Fallible<Expr> {
    let FunctionArguments::List(args) = &function.args else {
        return fallible!(FailedFunction, "{} requires positional arguments", P::NAME);
    };
    if args.args.is_empty() || args.args.len() > max_args {
        return fallible!(
            FailedFunction,
            "{} expects between 1 and {max_args} arguments",
            P::NAME
        );
    }

    let args = args
        .args
        .iter()
        .map(sql_function_arg_to_polars_expr)
        .collect::<Fallible<Vec<_>>>()?;
    Ok(crate::polars::apply_anonymous_function(args, plugin))
}

fn sql_function_arg_to_polars_expr(arg: &FunctionArg) -> Fallible<Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => sql_expr_to_polars_expr(expr),
        FunctionArg::Unnamed(FunctionArgExpr::Wildcard) => Ok(Expr::Selector(polars::prelude::Selector::Wildcard)),
        _ => fallible!(
            FailedFunction,
            "OpenDP SQL grouped aggregates only support positional arguments"
        ),
    }
}

fn sql_expr_to_polars_expr(expr: &SqlExpr) -> Fallible<Expr> {
    match expr {
        SqlExpr::Identifier(ident) => Ok(col(ident.value.as_str())),
        SqlExpr::CompoundIdentifier(idents) if idents.len() == 1 => Ok(col(idents[0].value.as_str())),
        SqlExpr::Value(value) => sql_value_to_polars_expr(&value.value),
        SqlExpr::UnaryOp { op, expr } => {
            let value = literal_expr_to_f64(expr)?;
            match op.to_string().as_str() {
                "-" => Ok(lit(-value)),
                "+" => Ok(lit(value)),
                _ => fallible!(FailedFunction, "unsupported unary operator in OpenDP SQL aggregate"),
            }
        }
        _ => fallible!(
            FailedFunction,
            "OpenDP SQL grouped aggregates currently support column references and literals"
        ),
    }
}

fn literal_expr_to_f64(expr: &SqlExpr) -> Fallible<f64> {
    let SqlExpr::Value(value) = expr else {
        return fallible!(FailedFunction, "expected literal, found {:?}", expr);
    };
    match &value.value {
        Value::Number(value, _) => value
            .parse::<f64>()
            .map_err(|_| err!(FailedFunction, "unable to parse numeric literal {:?}", value)),
        _ => fallible!(FailedFunction, "expected numeric literal, found {:?}", value.value),
    }
}

fn sql_value_to_polars_expr(value: &Value) -> Fallible<Expr> {
    Ok(match value {
        Value::Number(value, _) => {
            if let Ok(value) = value.parse::<i64>() {
                lit(value)
            } else if let Ok(value) = value.parse::<f64>() {
                lit(value)
            } else {
                return fallible!(FailedFunction, "unable to parse numeric literal {:?}", value);
            }
        }
        Value::SingleQuotedString(value)
        | Value::TripleSingleQuotedString(value)
        | Value::TripleDoubleQuotedString(value)
        | Value::EscapedStringLiteral(value)
        | Value::UnicodeStringLiteral(value)
        | Value::DoubleQuotedString(value)
        | Value::SingleQuotedRawStringLiteral(value)
        | Value::DoubleQuotedRawStringLiteral(value)
        | Value::TripleSingleQuotedRawStringLiteral(value)
        | Value::TripleDoubleQuotedRawStringLiteral(value)
        | Value::NationalStringLiteral(value) => lit(value.clone()),
        Value::Boolean(value) => lit(*value),
        Value::Null => lit(polars::prelude::NULL),
        _ => return fallible!(FailedFunction, "unsupported SQL literal {:?}", value),
    })
}

fn replace_grouped_aggregates(
    plan: DslPlan,
    replacements: &[GroupedAggregateReplacement],
) -> DslPlan {
    let replacements = replacements
        .iter()
        .map(|replacement| (replacement.alias.as_str(), replacement.expr.clone()))
        .collect::<HashMap<_, _>>();

    fn recurse(plan: DslPlan, replacements: &HashMap<&str, Expr>) -> DslPlan {
        match plan {
            DslPlan::Filter { input, predicate } => DslPlan::Filter {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                predicate,
            },
            DslPlan::Select { expr, input, options } => DslPlan::Select {
                expr,
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                options,
            },
            DslPlan::GroupBy {
                input,
                keys,
                predicates,
                aggs,
                maintain_order,
                options,
                apply,
            } => DslPlan::GroupBy {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                keys,
                predicates,
                aggs: aggs
                    .into_iter()
                    .map(|agg| replace_agg_expr(agg, replacements))
                    .collect(),
                maintain_order,
                options,
                apply,
            },
            DslPlan::Join {
                input_left,
                input_right,
                left_on,
                right_on,
                predicates,
                options,
            } => DslPlan::Join {
                input_left: Arc::new(recurse(Arc::unwrap_or_clone(input_left), replacements)),
                input_right: Arc::new(recurse(Arc::unwrap_or_clone(input_right), replacements)),
                left_on,
                right_on,
                predicates,
                options,
            },
            DslPlan::HStack { input, exprs, options } => DslPlan::HStack {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                exprs,
                options,
            },
            DslPlan::Distinct { input, options } => DslPlan::Distinct {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                options,
            },
            DslPlan::Sort {
                input,
                by_column,
                slice,
                sort_options,
            } => DslPlan::Sort {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                by_column,
                slice,
                sort_options,
            },
            DslPlan::Slice { input, offset, len } => DslPlan::Slice {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                offset,
                len,
            },
            DslPlan::MapFunction { input, function } => DslPlan::MapFunction {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                function,
            },
            DslPlan::ExtContext { input, contexts } => DslPlan::ExtContext {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                contexts: contexts
                    .into_iter()
                    .map(|plan| recurse(plan, replacements))
                    .collect(),
            },
            DslPlan::Sink { input, payload } => DslPlan::Sink {
                input: Arc::new(recurse(Arc::unwrap_or_clone(input), replacements)),
                payload,
            },
            DslPlan::Union { inputs, args } => DslPlan::Union {
                inputs: inputs
                    .into_iter()
                    .map(|plan| recurse(plan, replacements))
                    .collect(),
                args,
            },
            DslPlan::HConcat { inputs, options } => DslPlan::HConcat {
                inputs: inputs
                    .into_iter()
                    .map(|plan| recurse(plan, replacements))
                    .collect(),
                options,
            },
            other => other,
        }
    }

    recurse(plan, &replacements)
}

fn replace_agg_expr(expr: Expr, replacements: &HashMap<&str, Expr>) -> Expr {
    expr.map_expr(|node| match &node {
        Expr::Alias(_, alias) => replacements.get(alias.as_str()).cloned().unwrap_or(node),
        _ => node,
    })
}

#[derive(Default)]
struct ODPFunctionRegistry {
    registry: HashMap<PlSmallStr, UserDefinedFunction>,
}

fn register<P: OpenDPPlugin>(plugin: P) -> Fallible<(PlSmallStr, UserDefinedFunction)> {
    let function = UserDefinedFunction {
        name: P::NAME.into(),
        fun: LazySerde::Deserialized(SpecialEq::new(Arc::new(plugin))),
        options: P::function_options(),
    };
    Ok((P::NAME.into(), function))
}

impl ODPFunctionRegistry {
    fn new() -> Fallible<Self> {
        macro_rules! map_shims {
            ($($expr:expr),+) => ([$(register($expr)?),+])
        }
        Ok(ODPFunctionRegistry {
            registry: HashMap::from(map_shims![
                IndexCandidatesShim,
                NoiseShim,
                NoisyMaxShim,
                DiscreteQuantileScoreShim,
                DPFrameLenShim,
                DPLenShim,
                DPCountShim,
                DPNullCountShim,
                DPNUniqueShim,
                DPSumShim,
                DPMeanShim,
                DPQuantileShim,
                DPMedianShim
            ]),
        })
    }
}

impl FunctionRegistry for ODPFunctionRegistry {
    fn register(&mut self, name: &str, fun: UserDefinedFunction) -> PolarsResult<()> {
        self.registry.insert(name.into(), fun);
        Ok(())
    }

    fn get_udf(&self, name: &str) -> PolarsResult<Option<UserDefinedFunction>> {
        Ok(self.registry.get(name).cloned())
    }

    fn contains(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }
}

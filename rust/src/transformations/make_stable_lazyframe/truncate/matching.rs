use std::collections::HashSet;

use crate::{metrics::Bound, polars::literal_value_of};
use polars_plan::prelude::{ApplyOptions, FunctionOptions, GroupbyOptions};

use polars::prelude::{
    BooleanFunction, DataType, DslPlan, Expr, FunctionExpr, Operator, RankMethod, WindowMapping,
    WindowType, int_range, len, lit,
};

pub(crate) enum TruncatePlan {
    Filter(Expr),
    GroupBy { keys: Vec<Expr>, aggs: Vec<Expr> },
}

pub(crate) struct TruncateMatch {
    pub bounds: Vec<Bound>,
    pub plan: TruncatePlan,
}

pub(crate) fn match_truncations(
    mut plan: DslPlan,
    identifier: &Expr,
) -> (DslPlan, Vec<TruncateMatch>) {
    let mut matches = vec![];
    // continue until we reach a plan step that is not a truncation
    loop {
        let (input, truncate) = match plan.clone() {
            DslPlan::Filter { input, predicate } => {
                let Some(bounds) = match_truncate_filter(&predicate, identifier) else {
                    return (plan, matches);
                };

                let truncate = TruncateMatch {
                    bounds,
                    plan: TruncatePlan::Filter(predicate.clone()),
                };
                (input, truncate)
            }
            DslPlan::GroupBy {
                input,
                keys,
                aggs,
                apply,
                options,
                ..
            } => {
                if apply.is_some() || options.as_ref() != &GroupbyOptions::default() {
                    return (plan, matches);
                }

                let Some(bound) = match_truncate_group_by(&keys, identifier) else {
                    return (plan, matches);
                };

                let truncate = TruncateMatch {
                    bounds: vec![bound],
                    plan: TruncatePlan::GroupBy { keys, aggs },
                };

                (input, truncate)
            }
            _ => return (plan, matches),
        };

        plan = (*input).clone();
        matches.push(truncate);
    }
}

fn match_truncate_group_by(keys: &Vec<Expr>, identifier: &Expr) -> Option<Bound> {
    let (ids, by) = (keys.iter().cloned()).partition::<HashSet<_>, _>(|expr| expr == identifier);

    if ids.len() != 1 {
        return None;
    }

    Some(Bound {
        by: by.iter().cloned().collect(),
        per_group: Some(1),
        num_groups: None,
    })
}

fn match_truncate_filter(predicate: &Expr, identifier: &Expr) -> Option<Vec<Bound>> {
    match predicate {
        // handles all_horizontal which is emitted by filter(a, b, c) in polars
        Expr::Function {
            input,
            function: FunctionExpr::Boolean(BooleanFunction::All { .. }),
            ..
        } => Some(
            input
                .iter()
                .map(|expr| match_truncate_filter(expr, identifier))
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect(),
        ),
        // handles logical where multiple criteria must be met
        Expr::BinaryExpr {
            left,
            op: Operator::And,
            right,
        } => {
            let left = match_truncate_filter(left, identifier)?;
            let right = match_truncate_filter(right, identifier)?;
            Some([left, right].concat())
        }
        // conditions that limit contributions per partition or influenced partitions
        Expr::BinaryExpr { .. } => {
            match_truncate_filter_predicate(predicate, identifier).map(|bound| vec![bound])
        }
        _ => None,
    }
}

fn match_truncate_filter_predicate(predicate: &Expr, identifier: &Expr) -> Option<Bound> {
    let Expr::BinaryExpr { left, op, right } = predicate else {
        return None;
    };

    let (over, threshold) = match op {
        Operator::Lt => (left, literal_value_of::<u32>(&right).ok()??),
        Operator::LtEq => (left, literal_value_of::<u32>(&right).ok()?? + 1),
        Operator::Gt => (right, literal_value_of::<u32>(&left).ok()??),
        Operator::GtEq => (right, literal_value_of::<u32>(&left).ok()?? + 1),
        _ => return None,
    };

    let Expr::Window {
        function,
        partition_by,
        order_by,
        options: WindowType::Over(WindowMapping::GroupsToRows),
    } = over.as_ref()
    else {
        return None;
    };

    // check if the function is limiting max influenced partitions
    if let Expr::Function {
        input,
        function: FunctionExpr::Rank { options, .. },
        ..
    } = function.as_ref()
    {
        if partition_by != &vec![identifier.clone()] {
            return None;
        }

        if !matches!(options.method, RankMethod::Dense) {
            return None;
        }
        let [input] = <&[_; 1]>::try_from(input.as_slice()).ok()?;

        // determine the columns that the bound is with respect to
        let Expr::Function {
            input,
            function: FunctionExpr::AsStruct,
            ..
        } = input
        else {
            return None;
        };

        return Some(Bound {
            by: input.into_iter().cloned().collect(),
            per_group: None,
            num_groups: Some(threshold),
        });
    }

    // check if the function is limiting partition contributions
    if !is_enumeration(&**function) || order_by.is_some() {
        return None;
    }

    let (ids, over) = partition_by
        .iter()
        .cloned()
        .partition::<HashSet<_>, _>(|expr| expr == identifier);

    if ids.len() != 1 {
        return None;
    }

    Some(Bound {
        by: over,
        per_group: Some(threshold),
        num_groups: None,
    })
}

fn is_enumeration(expr: &Expr) -> bool {
    let expr = ignore_reorder(expr);
    expr.eq(&int_range(lit(0), len(), 1, DataType::Int64))
}

fn ignore_reorder(expr: &Expr) -> &Expr {
    let Expr::Function {
        input,
        function,
        options,
    } = expr
    else {
        return expr;
    };

    if options
        != &(FunctionOptions {
            collect_groups: ApplyOptions::GroupWise,
            ..Default::default()
        })
    {
        return expr;
    }

    match function {
        FunctionExpr::Reverse => (),
        FunctionExpr::Random { method, .. } => {
            // since method is not public, we can't match on the enum directly.
            // however, we can convert it to a string and match on that.
            let method: &'static str = method.into();
            if method == "shuffle" {
                return <&[Expr; 1]>::try_from(input.as_slice())
                    .map(|[v]| v)
                    .unwrap_or(expr);
            }
        }
        _ => return expr,
    }

    let Ok([first_input]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return expr;
    };

    first_input
}

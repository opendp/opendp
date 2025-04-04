use std::collections::HashSet;

use crate::{metrics::GroupBound, polars::literal_value_of};
use polars_plan::prelude::{ApplyOptions, FunctionOptions};

use polars::prelude::{
    AggExpr, BooleanFunction, DataType, DslPlan, Expr, FunctionExpr, Operator, WindowMapping,
    WindowType, int_range, len, lit,
};

pub(crate) struct TruncateMatch {
    pub bounds: Vec<GroupBound>,
    pub predicate: Expr,
}

pub(crate) fn match_truncations(
    mut plan: DslPlan,
    identifier: &Expr,
) -> (Vec<TruncateMatch>, DslPlan) {
    let mut matches = vec![];
    // continue until we reach a plan step that is not a truncation
    loop {
        let DslPlan::Filter { input, predicate } = plan.clone() else {
            return (matches, plan);
        };

        let Some(bounds) = match_truncate_predicate(&predicate, identifier) else {
            return (matches, plan);
        };

        plan = (*input).clone();

        matches.push(TruncateMatch {
            bounds,
            predicate: predicate.clone(),
        });
    }
}

pub(crate) fn match_truncate_predicate(
    predicate: &Expr,
    identifier: &Expr,
) -> Option<Vec<GroupBound>> {
    match predicate {
        // handles all_horizontal which is emitted by filter(a, b, c) in polars
        Expr::Function {
            input,
            function: FunctionExpr::Boolean(BooleanFunction::All { .. }),
            ..
        } => Some(
            input
                .iter()
                .map(|expr| match_truncate_predicate(expr, identifier))
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
            let left = match_truncate_predicate(left, identifier)?;
            let right = match_truncate_predicate(right, identifier)?;
            Some([left, right].concat())
        }
        // conditions that limit max influenced partitions
        Expr::Function {
            function: FunctionExpr::ListExpr(_),
            ..
        } => match_truncate_partitions(predicate, identifier).map(|bound| vec![bound]),
        // conditions that limit max partition contributions
        Expr::BinaryExpr { .. } => {
            match_truncate_contributions(predicate, identifier).map(|bound| vec![bound])
        }
        _ => None,
    }
}

fn match_list_function(expr: &Expr, name: String) -> Option<&Vec<Expr>> {
    let Expr::Function {
        input,
        function: FunctionExpr::ListExpr(function),
        ..
    } = expr
    else {
        return None;
    };
    // this check is done via fmt::Debug because ListFunction is not public until after 0.44
    (format!("{function:?}") == name).then_some(input)
}

fn match_truncate_partitions(predicate: &Expr, identifier: &Expr) -> Option<GroupBound> {
    // matches through: list.arr.contains(value)
    let input = match_list_function(predicate, "Contains".to_string())?;
    let [list, value] = <&[_; 2]>::try_from(input.as_slice()).ok()?;

    // matches through: input.arr.get(0)
    let input = match_list_function(list, "Get(false)".to_string())?;
    let [input, idx] = <&[_; 2]>::try_from(input.as_slice()).ok()?;
    if literal_value_of::<u32>(idx).ok()?? != 0 {
        return None;
    }
    // matches through: function.over(identifier, mapping_strategy="join")
    let Expr::Window {
        function,
        partition_by,
        order_by,
        options,
    } = input
    else {
        return None;
    };

    if order_by.is_some() || !matches!(options, WindowType::Over(WindowMapping::Join)) {
        return None;
    }

    if partition_by.clone() != vec![identifier.clone()] {
        return None;
    }

    // matches through: input.implode()
    let Expr::Agg(AggExpr::Implode(input)) = function.as_ref() else {
        return None;
    };

    // matches through: input.slice(), input.head(), input.tail()
    let (input, threshold) = match input.as_ref() {
        Expr::Slice { input, length, .. } => {
            let threshold = literal_value_of::<u32>(length).ok()??;
            (input.as_ref(), threshold)
        }
        Expr::Function {
            input: args,
            function: FunctionExpr::Random { seed, .. },
            ..
        } => {
            let [new_input, length] = <&[_; 2]>::try_from(args.as_slice()).ok()?;

            if input.as_ref().clone()
                != new_input
                    .clone()
                    .sample_n(length.clone(), false, false, seed.clone())
            {
                return None;
            }
            let threshold = literal_value_of::<u32>(length).ok()??;
            (new_input, threshold)
        }
        _ => return None,
    };

    // matches through: input.unique()
    let Expr::Function {
        input,
        function: FunctionExpr::Unique(_),
        ..
    } = input
    else {
        return None;
    };

    let Ok([input]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return None;
    };

    // key check that list items match the contains value
    if input != value {
        return None;
    }

    // determine the columns that the bound is with respect to
    let Expr::Function {
        input: over,
        function: FunctionExpr::AsStruct,
        ..
    } = input
    else {
        return None;
    };

    Some(GroupBound {
        by: over.into_iter().cloned().collect(),
        max_partition_contributions: None,
        max_influenced_partitions: Some(threshold),
    })
}

fn match_truncate_contributions(predicate: &Expr, identifier: &Expr) -> Option<GroupBound> {
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
        options,
    } = over.as_ref()
    else {
        return None;
    };

    if !is_enumeration(&**function) || order_by.is_some() {
        return None;
    }

    if !matches!(options, WindowType::Over(WindowMapping::GroupsToRows)) {
        return None;
    }

    let (ids, over) = partition_by
        .iter()
        .cloned()
        .partition::<HashSet<_>, _>(|expr| expr == identifier);

    if ids.is_empty() {
        return None;
    }

    Some(GroupBound {
        by: over,
        max_partition_contributions: Some(threshold),
        max_influenced_partitions: None,
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

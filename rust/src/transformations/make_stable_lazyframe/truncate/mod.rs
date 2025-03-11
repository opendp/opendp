use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::DslPlanDomain;
use crate::error::*;
use crate::metrics::{GroupBound, GroupBounds, Multi, SymmetricDistance, SymmetricIdDistance};
use crate::polars::literal_value_of;
use crate::traits::InfMul;
use polars::prelude::*;
use polars_plan::prelude::{ApplyOptions, FunctionOptions};

use super::StableDslPlan;

#[cfg(test)]
mod test;

/// Transformation for creating a stable LazyFrame truncation.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_truncate(
    input_domain: DslPlanDomain,
    input_metric: Multi<SymmetricIdDistance>,
    plan: DslPlan,
) -> Fallible<
    Transformation<
        DslPlanDomain,
        DslPlanDomain,
        Multi<SymmetricIdDistance>,
        Multi<SymmetricDistance>,
    >,
> {
    let DslPlan::Filter { input, predicate } = plan else {
        return fallible!(MakeTransformation, "Expected filter in logical plan");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, Multi<SymmetricIdDistance>) = t_prior.output_space();

    let Some((over, threshold)) = match_truncate(&predicate, &middle_metric.0.identifier) else {
        return fallible!(MakeTransformation, "Expected truncation in logical plan");
    };

    let mut output_domain = middle_domain.clone();

    output_domain.margins.iter_mut().for_each(|m| {
        // After filtering you no longer know partition lengths or keys.
        m.public_info = None;
    });

    t_prior
        >> Transformation::new(
            middle_domain,
            output_domain,
            Function::new_fallible(move |plan: &DslPlan| {
                Ok(DslPlan::Filter {
                    input: Arc::new(plan.clone()),
                    predicate: predicate.clone(),
                })
            }),
            middle_metric.clone(),
            Multi(SymmetricDistance),
            StabilityMap::new_fallible(move |d_in: &GroupBounds| {
                // once truncated, max partition contributions when grouped by "over" are bounded
                let bound = d_in.get_bound(&over);
                let mut new_bound = GroupBound::by(&over.iter().cloned().collect::<Vec<_>>());
                if let Some(mpc) = bound.max_partition_contributions {
                    new_bound =
                        new_bound.with_max_partition_contributions(mpc.inf_mul(&threshold)?);
                } else {
                    // if the bound is not set, we can't do anything
                    return fallible!(
                        FailedMap,
                        "ID contributions to grouping ({over:?}) are not bounded."
                    );
                }

                // truncation does not affect max influenced partitions
                if let Some(mip) = bound.max_influenced_partitions {
                    new_bound = new_bound.with_max_influenced_partitions(mip);
                }
                Ok(GroupBounds(vec![new_bound]))
            }),
        )?
}

pub(crate) fn match_truncate(predicate: &Expr, identifier: &Expr) -> Option<(HashSet<Expr>, u32)> {
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
    } = &**over
    else {
        return None;
    };

    if !is_enumeration(&**function) || order_by.is_some() {
        return None;
    }

    if !matches!(options, WindowType::Over(WindowMapping::GroupsToRows)) {
        return None;
    }

    let (ids, other) = partition_by
        .iter()
        .cloned()
        .partition::<HashSet<_>, _>(|expr| expr == identifier);

    if ids.is_empty() {
        return None;
    }

    Some((other, threshold))
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
            if method != "Shuffle" {
                return expr;
            }
        }
        _ => return expr,
    }

    let Ok([first_input]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return expr;
    };

    first_input
}

use std::collections::HashSet;

use crate::{
    error::Fallible,
    metrics::Bound,
    polars::literal_value_of,
    traits::InfAdd,
    transformations::make_stable_lazyframe::group_by::{Resize, check_infallible},
};
use opendp_derive::proven;
use polars_plan::prelude::GroupbyOptions;

use polars::prelude::{
    BooleanFunction, DataType, DslPlan, Expr, FunctionExpr, Operator, RankMethod, WindowMapping,
    WindowType, int_range, len, lit,
};

#[cfg(test)]
mod test;

#[derive(PartialEq, Debug)]
pub(crate) enum Truncation {
    Filter(Expr),
    GroupBy { keys: Vec<Expr>, aggs: Vec<Expr> },
}

/// Matches multiple truncations in a compute plan.
/// Errors only when the plan is unambiguously a mis-specified truncation.
///
/// # Proof Definition
/// For any choice of LazyFrame plan,
/// returns the plan with the truncations removed,
/// the truncations that were removed,
/// and per-id bounds on row and/or group contributions.
#[proven]
pub(crate) fn match_truncations(
    mut plan: DslPlan,
    identifier: &Expr,
) -> Fallible<(DslPlan, Vec<Truncation>, Vec<Bound>)> {
    let mut truncations = vec![];
    let mut bounds = vec![];

    let allowed_keys =
        match_group_by_truncation(&plan, identifier).map(|(input, truncate, new_bound)| {
            plan = input;
            truncations.push(truncate);
            bounds.push(new_bound.clone());
            new_bound.by
        });

    // match until not a filter truncation
    while let DslPlan::Filter { input, predicate } = plan.clone() {
        let Some(new_bounds) = match_truncation_predicate(&predicate, identifier)? else {
            break;
        };

        // When filter truncation is behind a groupby truncation,
        // if the groupby group keys don't cover the filter truncation keys,
        // then the groupby aggs can overwrite the filter truncation keys,
        // invalidating the filter truncation bounds.
        if let Some(allowed_keys) = &allowed_keys {
            new_bounds.iter().try_for_each(|bound| if bound.by.is_subset(allowed_keys) {
                Ok(())
            } else {
                fallible!(
                    MakeTransformation,
                    "Filter truncation keys ({:?}) must be a subset of groupby truncation keys ({:?}). Otherwise the groupby truncation may invalidate filter truncation.",
                    bound.by, allowed_keys
                )
            })?
        }
        plan = input.as_ref().clone();
        truncations.push(Truncation::Filter(predicate.clone()));
        bounds.extend(new_bounds);
    }

    // just for better error messages, no privacy implications
    if match_group_by_truncation(&plan, identifier).is_some() {
        return fallible!(
            MakeTransformation,
            "Groupby truncation must be the last truncation in the plan. Otherwise the groupby truncation may invalidate later truncations."
        );
    }
    // since the parse descends to the source,
    // truncations and bounds are in reverse order
    truncations.reverse();
    bounds.reverse();

    Ok((plan, truncations, bounds))
}

/// Matches a truncation via a groupby.
///
/// # Proof Definition
/// For a given query plan and user identifier expression,
/// if the query plan bounds row contributions per-identifier via a group by,
/// returns a triple containing the input to the truncation,
/// the truncation itself, and the per-id bound on user contribution.
#[proven]
fn match_group_by_truncation(
    plan: &DslPlan,
    identifier: &Expr,
) -> Option<(DslPlan, Truncation, Bound)> {
    let DslPlan::GroupBy {
        input,
        keys,
        aggs,
        apply,
        options,
        ..
    } = plan.clone()
    else {
        return None;
    };
    if apply.is_some() || options.as_ref() != &GroupbyOptions::default() {
        return None;
    }

    let (ids, by) = (keys.iter().cloned()).partition::<HashSet<_>, _>(|expr| expr == identifier);

    if ids.is_empty() {
        return None;
    }

    Some((
        (*input).clone(),
        Truncation::GroupBy { keys, aggs },
        Bound {
            by,
            per_group: Some(1),
            num_groups: None,
        },
    ))
}

/// Match user identifier truncations in a predicate.
///
/// # Proof Definition
/// For a given filter predicate and identifier expression,
/// returns an error if the predicate contains a mis-specified truncation,
/// none if the predicate is not a truncation,
/// otherwise the per-identifier bounds on user contribution.
#[proven]
fn match_truncation_predicate(predicate: &Expr, identifier: &Expr) -> Fallible<Option<Vec<Bound>>> {
    Ok(Some(match predicate {
        // handles all_horizontal which is emitted by filter(a, b, c) in polars Python
        Expr::Function {
            input,
            function: FunctionExpr::Boolean(BooleanFunction::AllHorizontal),
            ..
        } => {
            // propagate errors
            let bounds = (input.iter())
                .map(|expr| match_truncation_predicate(expr, identifier))
                .collect::<Fallible<Vec<Option<Vec<Bound>>>>>()?;

            // propagate nones
            let Some(bounds) = bounds.into_iter().collect::<Option<Vec<Vec<Bound>>>>() else {
                return Ok(None);
            };

            // flatten the bounds
            bounds.into_iter().flatten().collect::<Vec<_>>()
        }

        // handles logical and where multiple criteria must be met
        Expr::BinaryExpr {
            left,
            op: Operator::And,
            right,
        } => {
            let left = match_truncation_predicate(left, identifier)?;
            let right = match_truncation_predicate(right, identifier)?;
            let Some((left, right)) = left.zip(right) else {
                return Ok(None);
            };
            [left, right].concat()
        }

        // conditions that limit contributions per partition or influenced partitions
        Expr::BinaryExpr { left, op, right } => {
            let (over, threshold, offset) = match op {
                Operator::Lt => (left, right, 0),
                Operator::LtEq => (left, right, 1),
                Operator::Gt => (right, left, 0),
                Operator::GtEq => (right, left, 1),
                // don't throw an error as non-truncation filters may still be valid
                _ => return Ok(None),
            };

            let Expr::Window {
                function,
                partition_by,
                options: WindowType::Over(WindowMapping::GroupsToRows),
                ..
            } = over.as_ref()
            else {
                return Ok(None);
            };

            // Filters that aren't truncations don't support window functions,
            // so from here on, we can safely assume the predicate is meant to be a truncation,
            // and raise informative errors when the predicate is mis-shapen.

            let Some(threshold) = literal_value_of::<u32>(&threshold)? else {
                return fallible!(
                    MakeTransformation,
                    "literal value for truncation threshold ({:?}) must be representable as a u32",
                    threshold
                );
            };

            // account for distinction between gt and ge
            let threshold_value = threshold.inf_add(&offset)?;

            let num_groups = match_num_groups_predicate(
                function.as_ref(),
                partition_by,
                identifier,
                threshold_value,
            )?;
            let per_group = match_per_group_predicate(
                function.as_ref(),
                partition_by,
                identifier,
                threshold_value,
            )?;

            let Some(bound) = num_groups.or(per_group) else {
                return fallible!(
                    MakeTransformation,
                    "expected a predicate that limits per_group contributions (via int_range) or num_groups contributions (via rank). Found {:?}",
                    function
                );
            };

            vec![bound]
        }
        _ => return Ok(None),
    }))
}

/// # Proof Definition
/// If `ranks` is a dense ranking of grouping columns,
/// and `partition_by` is a singleton of `identifier`,
/// then returns the bound on per-identifier contributions,
/// or an error if the truncation is mis-specified.
#[proven]
fn match_num_groups_predicate(
    ranks: &Expr,
    partition_by: &Vec<Expr>,
    identifier: &Expr,
    threshold: u32,
) -> Fallible<Option<Bound>> {
    // check if the function is limiting num_groups
    let Expr::Function {
        input,
        function: FunctionExpr::Rank { options, .. },
        ..
    } = ranks
    else {
        return Ok(None);
    };
    if partition_by != &vec![identifier.clone()] {
        return fallible!(
            MakeTransformation,
            "num_groups truncation must use the identifier in the over clause"
        );
    }

    if !matches!(options.method, RankMethod::Dense) {
        return fallible!(
            MakeTransformation,
            "num_groups truncation's rank must be dense"
        );
    }

    let Ok([input_item]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeTransformation,
            "rank function must be applied to a single input, found {:?}",
            input.len()
        );
    };

    let by = match input_item.clone() {
        // Treat as_struct as a special case that represents multiple columns.
        // Could still actually have a struct column via to_struct(to_struct(...)).
        Expr::Function {
            function: FunctionExpr::AsStruct,
            mut input,
            ..
        } => {
            // If the first field is a hash of the second field,
            // then interpret the grouping columns as the hash input.
            // The second field disambiguates hash collisions when ranking.
            if let Some(Expr::Function {
                input: hash_input,
                function: FunctionExpr::Hash(_, _, _, _),
                ..
            }) = input.get(0)
            {
                if hash_input.get(0) == input.get(1) {
                    let Some(Expr::Function {
                        input: true_input,
                        function: FunctionExpr::AsStruct,
                        ..
                    }) = hash_input.get(0)
                    else {
                        return fallible!(
                            MakeTransformation,
                            "expected hash input to be a struct, found {:?}",
                            hash_input
                        );
                    };
                    input = true_input.clone();
                }
            }
            input.into_iter().collect()
        }
        input => HashSet::from([input.clone()]),
    };

    Ok(Some(Bound {
        by,
        per_group: None,
        num_groups: Some(threshold),
    }))
}

/// # Proof Definition
/// If `enumeration` is an enumeration of rows,
/// and `partition_by` includes `identifier`,
/// then returns a `threshold` bound on per-group contribution,
/// when grouped by the non-identifier columns in `partition_by`.
#[proven]
fn match_per_group_predicate(
    mut enumeration: &Expr,
    partition_by: &Vec<Expr>,
    identifier: &Expr,
    threshold: u32,
) -> Fallible<Option<Bound>> {
    // reorderings of an enumeration are still enumerations
    match enumeration {
        Expr::Function {
            input, function, ..
        } => {
            // FunctionExprs that may reorder data
            let is_reorder = match function {
                FunctionExpr::Reverse => true,
                FunctionExpr::Random { method, .. } => {
                    // since method is not public, we can't match on the enum directly.
                    // however, we can convert it to a string and match on that.
                    let method: &'static str = method.into();
                    method == "shuffle"
                }
                _ => false,
            };

            if is_reorder {
                enumeration = input
                    .get(0)
                    .ok_or_else(|| err!(MakeTransformation, "expected one input"))?;
            }
        }
        Expr::SortBy { expr, by, .. } => {
            by.iter()
                .try_for_each(|key| check_infallible(key, Resize::Ban))?;
            enumeration = expr.as_ref()
        }
        _ => (),
    };

    if enumeration.ne(&int_range(lit(0), len(), 1, DataType::Int64)) {
        return Ok(None);
    }
    // we now know this is a per group predicate,
    // and can raise more informative error messages

    // check if the function is limiting partition contributions
    let (ids, by) = partition_by
        .iter()
        .cloned()
        .partition::<HashSet<_>, _>(|expr| expr == identifier);

    if ids.is_empty() {
        return fallible!(
            MakeTransformation,
            "failed to find identifier column in per_group predicate condition"
        );
    }

    Ok(Some(Bound {
        by,
        per_group: Some(threshold),
        num_groups: None,
    }))
}

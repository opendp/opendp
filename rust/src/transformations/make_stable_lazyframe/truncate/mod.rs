use std::collections::{HashMap, HashSet};

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{
    Bound, Bounds, FrameDistance, L0PInfDistance, L01InfDistance, SymmetricDistance,
    SymmetricIdDistance,
};
use crate::traits::{InfMul, option_min};
use crate::transformations::make_stable_expr;
use matching::{GroupByTruncation, MatchedTruncations, num_groups_keys};
use opendp_derive::proven;
use polars::prelude::*;
use polars_plan::plans::visitor::TreeWalker;

use super::StableDslPlan;
use super::group_by::{group_by_plan, materialize_grouping_keys, prepare_group_by};

#[cfg(test)]
mod test;

mod matching;
pub(crate) use matching::match_truncations;

/// Transformation for creating a stable LazyFrame truncation.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `matched` - The matched input, truncations and bounds.
pub(super) fn make_stable_truncate(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<SymmetricIdDistance>,
    matched: MatchedTruncations,
) -> Fallible<
    Transformation<
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
        DslPlanDomain,
        FrameDistance<SymmetricDistance>,
    >,
> {
    let t_prior = matched
        .input
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, FrameDistance<SymmetricIdDistance>) =
        t_prior.output_space();

    let stable_input = t_prior.invoke(&matched.input)?;
    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };
    let identifier = &middle_metric.0.identifier;
    let keys = std::iter::once(identifier).chain(matched.bounds.iter().flat_map(|b| &b.bound.by));
    let mut stable_keys = HashMap::new();
    for key in keys.collect::<HashSet<_>>() {
        let t_key = make_stable_expr::<_, L01InfDistance<SymmetricIdDistance>>(
            expr_domain.clone(),
            L0PInfDistance(middle_metric.0.clone()),
            key.clone(),
        )?;
        let stable_key = t_key.invoke(&stable_input)?.expr;
        if key == identifier && stable_key != *key {
            return fallible!(MakeTransformation, "The identifier cannot be rewritten.");
        }
        stable_keys.insert(key.clone(), stable_key);
    }

    let MatchedTruncations {
        mut filters,
        mut group_by,
        bounds,
        ..
    } = matched;

    // Rewrite only the grouping keys of each matched truncation.
    for predicate in &mut filters {
        *predicate = rewrite_windows(predicate.clone(), &stable_keys)?;
    }
    let schema = middle_domain.schema();
    let truncation_bounds = bounds
        .into_iter()
        .map(|mut candidate| {
            candidate.bound.by = candidate
                .bound
                .by
                .iter()
                .map(|key| stable_keys[key].clone())
                .collect();
            candidate.validate_ranking(&schema)
        })
        .collect::<Fallible<Vec<_>>>()?;

    if let Some(group_by) = &mut group_by {
        for key in &mut group_by.keys {
            *key = stable_keys[&*key].clone();
        }
        let allowed_keys: HashSet<_> = group_by.keys.iter().cloned().collect();
        for bound in &truncation_bounds {
            if !bound.by.is_subset(&allowed_keys) {
                return fallible!(
                    MakeTransformation,
                    "Filter truncation keys ({:?}) must be a subset of groupby truncation keys ({:?}). Otherwise the groupby truncation may invalidate filter truncation.",
                    bound.by,
                    allowed_keys
                );
            }
        }
    }

    let output_domain = if let Some(GroupByTruncation { keys, aggs }) = &mut group_by {
        // Filters preserve schema and margin upper bounds.
        // Group-by preparation already discards margin invariants.
        prepare_group_by(&middle_domain, keys, aggs)?
    } else {
        let mut domain = middle_domain.clone();
        domain.margins.iter_mut().for_each(|m| m.invariant = None);
        domain
    };
    let has_group_by = group_by.is_some();

    let t_truncate = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        FrameDistance(SymmetricDistance),
        Function::new(move |plan: &DslPlan| {
            let plan = filters
                .iter()
                .fold(plan.clone(), |plan, predicate| DslPlan::Filter {
                    input: Arc::new(plan),
                    predicate: predicate.clone(),
                });
            match &group_by {
                Some(GroupByTruncation { keys, aggs }) => {
                    group_by_plan(plan, keys.clone(), aggs.clone(), false)
                }
                None => plan,
            }
        }),
        StabilityMap::new_fallible(move |id_bounds: &Bounds| {
            let total_num_ids = id_bounds.get_bound(&Default::default()).per_group;

            // each truncation is used to derive row bounds
            let new_bounds = truncation_bounds
                .iter()
                .map(|truncation_bound| {
                    // Look up the bound on the actual execution keys.
                    let mut row_bound = truncate_id_bound(
                        id_bounds.get_bound(&truncation_bound.by),
                        truncation_bound.clone(),
                        total_num_ids,
                    )?;
                    if has_group_by {
                        row_bound.by = materialize_grouping_keys(&row_bound.by)?;
                    }
                    Ok(row_bound)
                })
                .collect::<Fallible<Vec<Bound>>>()?;
            Ok(Bounds(new_bounds))
        }),
    )?;
    t_prior >> t_truncate
}

// Visit the matched predicate's windows, without rewriting the rank priorities,
// comparison thresholds or other non-grouping expressions.
fn rewrite_windows(mut expr: Expr, keys: &HashMap<Expr, Expr>) -> PolarsResult<Expr> {
    let (function, partition_by) = match &mut expr {
        #[cfg(patch_polars)]
        Expr::Over {
            function,
            partition_by,
            ..
        } => (function, partition_by),
        #[cfg(not(patch_polars))]
        Expr::Window {
            function,
            partition_by,
            ..
        } => (function, partition_by),
        _ => return expr.map_children(&mut |child, _| rewrite_windows(child, keys), &mut ()),
    };
    if let Expr::Function {
        input,
        function: FunctionExpr::Rank { options, .. },
    } = Arc::make_mut(function)
        && options.method == RankMethod::Dense
    {
        let [rank_input] = input.as_mut_slice() else {
            polars_bail!(ComputeError: "Expected a single matched rank input");
        };
        let by = num_groups_keys(rank_input).map_err(|e| polars_err!(ComputeError: "{}", e))?;
        *rank_input = replace_keys(rank_input.clone(), by, keys)?;
    } else {
        for key in partition_by {
            *key = keys[&*key].clone();
        }
    }
    Ok(expr)
}

// The shared dense-rank key extractor identifies the exact grouping
// expressions, including both occurrences within hashed ranks.
fn replace_keys(expr: Expr, by: &[Expr], keys: &HashMap<Expr, Expr>) -> PolarsResult<Expr> {
    if by.contains(&expr) {
        return Ok(keys[&expr].clone());
    }
    expr.map_children(&mut |child, _| replace_keys(child, by, keys), &mut ())
}

/// # Proof Definition
/// See proof document.
#[proven]
fn truncate_id_bound(
    id_bound: Bound,
    truncation: Bound,
    total_ids: Option<u32>,
) -> Fallible<Bound> {
    // Once truncated, L0 and/or LInf norms between group-wise distances are bounded
    let mut row_bound = Bound::by(&truncation.by.iter().cloned().collect::<Vec<_>>());

    // In each group, the worst-case row contributions is the
    // the number of ids contributed (known from id_bound)
    // times the number of rows contributed under each id (known from truncation),
    if let Some((num_ids, num_rows)) = id_bound.per_group.zip(truncation.per_group) {
        row_bound = row_bound.with_per_group(num_ids.inf_mul(&num_rows)?);
    }

    // Worst case number of groups contributed is the
    // total number of ids contributed (total_ids)
    // times the number of groups contributed under each id (known from truncation).
    let num_groups_via_truncation = total_ids
        .zip(truncation.num_groups)
        .map(|(num_ids, num_groups)| num_ids.inf_mul(&num_groups))
        .transpose()?;

    // Alternatively, the number of groups contributed may be known outright from id_bound.
    // Use the smaller of the two if both are known.
    if let Some(num_groups) = option_min(num_groups_via_truncation, id_bound.num_groups) {
        row_bound = row_bound.with_num_groups(num_groups);
    }

    Ok(row_bound)
}

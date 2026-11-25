use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{
    Bound, Bounds, FrameDistance, L0PInfDistance, L01InfDistance, SymmetricDistance,
    SymmetricIdDistance,
};
use crate::traits::{InfMul, option_min};
use crate::transformations::make_stable_expr;
use matching::Truncation;
use opendp_derive::proven;
use polars::prelude::*;

use super::StableDslPlan;
use super::group_by::{
    group_by_output_domain, group_by_plan, has_canonical_representatives, materialize_grouping_keys,
};

#[cfg(test)]
mod test;

mod matching;
pub(crate) use matching::match_truncations;

/// Transformation for creating a stable LazyFrame truncation.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_truncate(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<SymmetricIdDistance>,
    plan: DslPlan,
) -> Fallible<
    Transformation<
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
        DslPlanDomain,
        FrameDistance<SymmetricDistance>,
    >,
> {
    // the identifier is protected from changes, so we can use the identifier from the input metric
    // instead of the identifier from the middle_metric to match truncations
    let (input, truncations, truncation_bounds) =
        match_truncations(plan, &input_metric.0.identifier)?;

    if truncations.is_empty() {
        // should be unreachable in practice, but makes this function self-contained
        return fallible!(MakeTransformation, "failed to match truncation");
    };

    let t_prior = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, FrameDistance<SymmetricIdDistance>) =
        t_prior.output_space();

    let grouping_keys: HashSet<_> = truncation_bounds.iter().flat_map(|b| &b.by).collect();
    for key in grouping_keys {
        // check that each grouping key/over key is infallible row-by-row
        make_stable_expr::<_, L01InfDistance<SymmetricIdDistance>>(
            WildExprDomain {
                columns: middle_domain.series_domains.clone(),
                context: Context::RowByRow,
            },
            L0PInfDistance(middle_metric.0.clone()),
            key.clone(),
        )?;
    }

    let output_domain = (truncations.iter())
        .try_fold(middle_domain.clone(), |domain, truncation| {
            truncate_domain(domain, truncation)
        })?;

    // The matcher permits filters followed by an optional final group-by.
    // Only that final group-by changes the expressions naming the output groups.
    let has_group_by = matches!(truncations.last(), Some(Truncation::GroupBy { .. }));
    let bounds = truncation_bounds
        .into_iter()
        .map(|bound| {
            let output_by = if has_group_by {
                materialize_grouping_keys(&bound.by)?
            } else {
                bound.by.clone()
            };
            Ok((bound, output_by))
        })
        .collect::<Fallible<Vec<_>>>()?;

    let t_truncate = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        FrameDistance(SymmetricDistance),
        Function::new(move |plan: &DslPlan| {
            (truncations.iter()).fold(plan.clone(), |plan, truncation| match truncation {
                Truncation::Filter(predicate) => DslPlan::Filter {
                    input: Arc::new(plan),
                    predicate: predicate.clone(),
                },
                Truncation::GroupBy { keys, aggs } => {
                    group_by_plan(plan, keys.clone(), aggs.clone(), false)
                }
            })
        }),
        StabilityMap::new_fallible(move |id_bounds: &Bounds| {
            let total_num_ids = id_bounds.get_bound(&Default::default()).per_group;

            // each truncation is used to derive row bounds
            let new_bounds = bounds
                .iter()
                .map(|(truncation_bound, output_by)| {
                    // Input certificates still refer to the original grouping expressions.
                    let mut row_bound = truncate_id_bound(
                        id_bounds.get_bound(&truncation_bound.by),
                        truncation_bound.clone(),
                        total_num_ids,
                    )?;
                    row_bound.by = output_by.clone();
                    Ok(row_bound)
                })
                .collect::<Fallible<Vec<Bound>>>()?;
            Ok(Bounds(new_bounds))
        }),
    )?;
    t_prior >> t_truncate
}

/// # Proof Definition
/// Returns the domain that spans all outputs when truncation is applied to all members of the input domain.
#[proven]
fn truncate_domain(mut domain: DslPlanDomain, truncation: &Truncation) -> Fallible<DslPlanDomain> {
    match truncation {
        Truncation::Filter { .. } => {
            domain.margins.iter_mut().for_each(|m| {
                // after filtering you no longer know partition lengths or keys
                m.invariant = None;
            });

            // upper bounds on the per-group length and num groups remains valid
            Ok(domain)
        }
        Truncation::GroupBy { keys, aggs } => {
            // The input identifier metric is unordered even within an id.
            // Validate the entire aggregation before dropping identifiers.
            let schema = domain.schema();
            for key in keys {
                if !has_canonical_representatives(&key.clone().to_field(&schema)?.dtype) {
                    return fallible!(
                        MakeTransformation,
                        "Floating-point grouping keys may emit permutation-dependent representatives (signed zeros or NaN payloads)."
                    );
                }
            }
            group_by_output_domain(&domain, &keys.iter().cloned().collect(), keys, aggs)
        }
    }
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

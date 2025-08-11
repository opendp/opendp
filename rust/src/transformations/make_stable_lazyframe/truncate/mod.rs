use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, FrameDomain, SeriesDomain, WildExprDomain};
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
use polars_plan::prelude::GroupbyOptions;

use super::StableDslPlan;
use super::group_by::{Resize, check_infallible};

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

    (truncation_bounds.iter().flat_map(|b| &b.by)).try_for_each(|key| {
        // check that each grouping key/over key is infallible row-by-row
        make_stable_expr::<_, L01InfDistance<SymmetricIdDistance>>(
            WildExprDomain {
                columns: middle_domain.series_domains.clone(),
                context: Context::RowByRow,
            },
            L0PInfDistance(middle_metric.0.clone()),
            key.clone(),
        )
        .map(|_| ())
    })?;

    let output_domain = (truncations.iter())
        .try_fold(middle_domain.clone(), |domain, truncation| {
            truncate_domain(domain, truncation)
        })?;

    let t_truncate = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        FrameDistance(SymmetricDistance),
        Function::new(move |plan: &DslPlan| {
            (truncations.iter()).fold(plan.clone(), |plan, truncation| match truncation {
                Truncation::Filter(predicate) => DslPlan::Filter {
                    input: Arc::new(plan.clone()),
                    predicate: predicate.clone(),
                },
                Truncation::GroupBy { keys, aggs } => DslPlan::GroupBy {
                    input: Arc::new(plan),
                    keys: keys.clone(),
                    aggs: aggs.clone(),
                    apply: None,
                    maintain_order: false,
                    options: Arc::new(GroupbyOptions::default()),
                },
            })
        }),
        StabilityMap::new_fallible(move |id_bounds: &Bounds| {
            let total_num_ids = id_bounds.get_bound(&Default::default()).per_group;

            // each truncation is used to derive row bounds
            let new_bounds = (truncation_bounds.iter())
                .map(|truncation_bound| {
                    truncate_id_bound(
                        id_bounds.get_bound(&truncation_bound.by),
                        truncation_bound.clone(),
                        total_num_ids,
                    )
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
    match &truncation {
        Truncation::Filter { .. } => {
            domain.margins.iter_mut().for_each(|m| {
                // after filtering you no longer know partition lengths or keys
                m.invariant = None;
            });

            // upper bounds on the per-group length and num groups remains valid
            Ok(domain)
        }
        Truncation::GroupBy { keys, aggs } => {
            // each agg expression must be infallible
            aggs.iter()
                .try_for_each(|e| check_infallible(e, Resize::Allow))?;

            // derive new output domain
            FrameDomain::new_with_margins(
                domain
                    .simulate_schema(|lf| lf.group_by(&keys).agg(&aggs))?
                    .iter_fields()
                    .map(SeriesDomain::new_from_field)
                    .collect::<Fallible<_>>()?,
                domain
                    .margins
                    .into_iter()
                    // only keep margins that are a subset of the truncation keys
                    .filter(|m| m.by.is_subset(&HashSet::from_iter(keys.clone())))
                    // discard invariants as multiverses are mixed
                    .map(|mut m| {
                        m.invariant = None;
                        m
                    })
                    .collect(),
            )
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

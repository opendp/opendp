use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{
    Context, DslPlanDomain, FrameDomain, SeriesDomain, WildExprDomain, option_min,
};
use crate::error::*;
use crate::metrics::{
    Bound, Bounds, FrameDistance, PartitionDistance, SymmetricDistance, SymmetricIdDistance,
};
use crate::traits::InfMul;
use crate::transformations::make_stable_expr;
use matching::TruncatePlan;
use polars::prelude::*;
use polars_plan::prelude::GroupbyOptions;

use super::StableDslPlan;
use super::group_by::assert_infallible;

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
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
        FrameDistance<SymmetricDistance>,
    >,
> {
    // the identifier is protected from changes, so we can use the identifier from the input metric
    // instead of the identifier from the middle_metric to match truncations
    let (input, truncations) = match_truncations(plan, &input_metric.0.identifier);

    if truncations.is_empty() {
        return fallible!(MakeTransformation, "failed to match truncation");
    };

    let t_prior = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, FrameDistance<SymmetricIdDistance>) =
        t_prior.output_space();

    let output_domain =
        truncations
            .iter()
            .try_fold(middle_domain.clone(), |mut domain, truncation| {
                match &truncation.plan {
                    TruncatePlan::Filter(_) => {
                        domain.margins.iter_mut().for_each(|m| {
                            // After filtering you no longer know partition lengths or keys.
                            m.invariant = None;
                        });
                        Ok(domain)
                    }
                    TruncatePlan::GroupBy { keys, aggs } => {
                        // each key expression must be stable row by row
                        keys.iter().try_for_each(|key| {
                            make_stable_expr::<_, PartitionDistance<SymmetricIdDistance>>(
                                WildExprDomain {
                                    columns: middle_domain.series_domains.clone(),
                                    context: Context::RowByRow,
                                },
                                PartitionDistance(middle_metric.0.clone()),
                                key.clone(),
                            )
                            .map(|_| ())
                        })?;

                        // each agg expression must be infallible. True means resize is allowed
                        aggs.iter().try_for_each(|e| assert_infallible(e, true))?;

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
            })?;

    let per_id_bounds = truncations
        .iter()
        .flat_map(|truncation| truncation.bounds.clone())
        .collect::<Vec<Bound>>();

    let t_truncate = Transformation::new(
        middle_domain,
        output_domain,
        Function::new(move |plan: &DslPlan| {
            truncations
                .iter()
                .fold(plan.clone(), |plan, truncation| match &truncation.plan {
                    TruncatePlan::Filter(predicate) => DslPlan::Filter {
                        input: Arc::new(plan.clone()),
                        predicate: predicate.clone(),
                    },
                    TruncatePlan::GroupBy { keys, aggs } => DslPlan::GroupBy {
                        input: Arc::new(plan),
                        keys: keys.clone(),
                        aggs: aggs.clone(),
                        apply: None,
                        maintain_order: false,
                        options: Arc::new(GroupbyOptions::default()),
                    },
                })
        }),
        middle_metric.clone(),
        FrameDistance(SymmetricDistance),
        StabilityMap::new_fallible(move |d_in: &Bounds| {
            let total_num_ids = d_in.get_bound(&Default::default()).per_group;

            let new_bounds = (per_id_bounds.iter())
                .map(|per_id_bound| {
                    let Bound {
                        by,
                        per_group: rows_per_id,
                        num_groups: groups_per_id,
                    } = per_id_bound.clone();
                    let Bound {
                        by,
                        per_group: num_ids_per_partition,
                        num_groups: num_groups_via_bound,
                    } = d_in.get_bound(&by);

                    // once truncated, max partition contributions when grouped by "over" are bounded
                    let mut new_bound = Bound::by(&by.iter().cloned().collect::<Vec<_>>());

                    if let Some((per_id, num_ids)) = rows_per_id.zip(num_ids_per_partition) {
                        new_bound = new_bound.with_per_group(per_id.inf_mul(&num_ids)?);
                    }

                    let num_groups_via_truncation = groups_per_id
                        .zip(total_num_ids)
                        .map(|(per_id, num_ids)| per_id.inf_mul(&num_ids))
                        .transpose()?;

                    if let Some(num_groups) =
                        option_min(num_groups_via_truncation, num_groups_via_bound)
                    {
                        new_bound = new_bound.with_num_groups(num_groups);
                    }
                    Ok(new_bound)
                })
                .collect::<Fallible<Vec<Bound>>>()?;
            Ok(Bounds(new_bounds))
        }),
    )?;
    t_prior >> t_truncate
}

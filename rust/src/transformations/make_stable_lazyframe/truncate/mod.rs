use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{DslPlanDomain, option_min};
use crate::error::*;
use crate::metrics::{GroupBound, GroupBounds, Multi, SymmetricDistance, SymmetricIdDistance};
use crate::traits::InfMul;
use polars::prelude::*;

use super::StableDslPlan;

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
    // the identifier is protected from changes, so we can use the identifier from the input metric
    // instead of the identifier from the middle_metric to match truncations
    let (truncations, input) = match_truncations(plan, &input_metric.0.identifier);

    if truncations.is_empty() {
        return fallible!(MakeTransformation, "failed to match truncation");
    };

    let t_prior = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, Multi<SymmetricIdDistance>) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();

    output_domain.margins.iter_mut().for_each(|m| {
        // After filtering you no longer know partition lengths or keys.
        m.public_info = None;
    });

    let per_id_bounds = truncations
        .iter()
        .flat_map(|truncation| truncation.bounds.clone())
        .collect::<Vec<GroupBound>>();

    t_prior
        >> Transformation::new(
            middle_domain,
            output_domain,
            Function::new(move |plan: &DslPlan| {
                truncations
                    .iter()
                    .fold(plan.clone(), |plan, truncation| DslPlan::Filter {
                        input: Arc::new(plan.clone()),
                        predicate: truncation.predicate.clone(),
                    })
            }),
            middle_metric.clone(),
            Multi(SymmetricDistance),
            StabilityMap::new_fallible(move |d_in: &GroupBounds| {
                let total_num_ids = d_in
                    .get_bound(&Default::default())
                    .max_partition_contributions;

                let new_bounds = (per_id_bounds.iter())
                    .map(|per_id_bound| {
                        let GroupBound {
                            by,
                            max_partition_contributions: rows_per_id,
                            max_influenced_partitions: partitions_per_id,
                        } = per_id_bound.clone();
                        let GroupBound {
                            by,
                            max_partition_contributions: num_ids_per_partition,
                            max_influenced_partitions: overall_influenced_partitions,
                        } = d_in.get_bound(&by);

                        // once truncated, max partition contributions when grouped by "over" are bounded
                        let mut new_bound = GroupBound::by(&by.iter().cloned().collect::<Vec<_>>());

                        if let Some((per_id, num_ids)) = rows_per_id.zip(num_ids_per_partition) {
                            new_bound = new_bound
                                .with_max_partition_contributions(num_ids.inf_mul(&per_id)?);
                        }

                        let mip = partitions_per_id
                            .zip(total_num_ids)
                            .map(|(per_id, num_ids)| per_id.inf_mul(&num_ids))
                            .transpose()?;

                        if let Some(mip) = option_min(mip, overall_influenced_partitions) {
                            new_bound = new_bound.with_max_influenced_partitions(mip);
                        }
                        Ok(new_bound)
                    })
                    .collect::<Fallible<Vec<GroupBound>>>()?;
                Ok(GroupBounds(new_bounds))
            }),
        )?
}

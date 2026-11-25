use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, FrameDomain, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{Bound, Bounds, FrameDistance, L0PInfDistance, L01InfDistance};
use crate::polars::contains_float;
use crate::polars::options::ORDINARY_GROUP_BY_OPTIONS;
use crate::traits::{InfMul, option_min};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use polars::prelude::*;

use super::StableDslPlan;
use super::permutation::{check_permutation_invariance, rewrite_aggregation};

#[cfg(test)]
mod test;

/// Transformation for stable group-by and aggregate.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_group_by<M: UnboundedMetric>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<M>,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>> {
    if M::ORDERED {
        return fallible!(
            MakeTransformation,
            "Sample-and-aggregate requires an unordered input metric."
        );
    }
    let (input, keys, aggs, maintain_order) = match_ordinary_group_by(&plan)?;

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, FrameDistance<M>) = t_prior.output_space();
    let stable_input = t_prior.invoke(input)?;

    // create a transformation for each expression
    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };

    // each expression must be stable row by row
    let stable_keys = keys
        .iter()
        .map(|key| {
            let t_key: Transformation<_, _, _, L01InfDistance<M>> = key
                .clone()
                .make_stable(expr_domain.clone(), L0PInfDistance(middle_metric.0.clone()))?;
            Ok(t_key.invoke(&stable_input)?.expr)
        })
        .collect::<Fallible<Vec<_>>>()?;

    if middle_metric.0.identifier().is_some() {
        return fallible!(
            MakeTransformation,
            "stable groupby (sample and aggregate) is not supported on datasets with unbounded row contributions. If you want to execute a groupby truncation, include the identifier in the groupby keys."
        );
    }

    let grouping_keys = stable_keys.iter().cloned().collect();
    let mut aggs = aggs.to_vec();
    let output_domain = prepare_group_by(&middle_domain, &stable_keys, &mut aggs)?;
    let t_group_agg = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        middle_metric.clone(),
        Function::new(move |plan: &DslPlan| {
            group_by_plan(
                plan.clone(),
                stable_keys.clone(),
                aggs.clone(),
                maintain_order,
            )
        }),
        StabilityMap::new_fallible(move |d_in: &Bounds| {
            let contributed_rows = d_in.get_bound(&HashSet::new()).per_group;
            let contributed_groups = d_in.get_bound(&grouping_keys).num_groups;

            let Some(influenced_groups) = option_min(contributed_rows, contributed_groups) else {
                return fallible!(
                    FailedMap,
                    "an upper bound on the number of contributed rows or groups is required"
                );
            };

            Ok(Bounds(vec![Bound {
                by: HashSet::new(),
                per_group: Some(influenced_groups.inf_mul(&2)?),
                num_groups: None,
            }]))
        }),
    )?;

    t_prior >> t_group_agg
}

/// Match a group-by without apply, HAVING, slicing or dynamic/rolling options.
pub(super) fn match_ordinary_group_by(
    plan: &DslPlan,
) -> Fallible<(&DslPlan, &[Expr], &[Expr], bool)> {
    let DslPlan::GroupBy {
        input,
        keys,
        aggs,
        apply,
        maintain_order,
        options,
        #[cfg(patch_polars)]
        predicates,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected group-by in logical plan");
    };

    if apply.is_some() {
        return fallible!(
            MakeTransformation,
            "Apply is not currently supported in the logical plan. Please open an issue if this would be useful to you."
        );
    }

    #[cfg(patch_polars)]
    if !predicates.is_empty() {
        return fallible!(
            MakeTransformation,
            "Having is not currently supported in logical plan. Please open an issue if this would be useful to you."
        );
    }

    if options.as_ref() != &ORDINARY_GROUP_BY_OPTIONS {
        return fallible!(
            MakeTransformation,
            "Unsupported stable group-by options: {options:?}"
        );
    }

    Ok((input.as_ref(), keys, aggs, *maintain_order))
}

/// Rewrite and certify aggregations and derive the output domain for validated grouping keys.
/// No metric conversion or stability map occurs here.
pub(super) fn prepare_group_by(
    domain: &DslPlanDomain,
    keys: &[Expr],
    aggs: &mut [Expr],
) -> Fallible<DslPlanDomain> {
    let schema = domain.schema();
    for key in keys {
        if contains_float(&key.clone().to_field(&schema)?.dtype) {
            return fallible!(
                MakeTransformation,
                "Floating-point grouping keys may emit permutation-dependent representatives (signed zeros or NaN payloads)."
            );
        }
    }
    for agg in aggs.iter_mut() {
        *agg = rewrite_aggregation(agg.clone(), &schema);
        check_permutation_invariance(agg, &schema)?;
    }

    let grouping_keys = keys.iter().cloned().collect();
    FrameDomain::new_with_margins(
        domain
            .simulate_schema(|lf| lf.group_by(keys).agg(&*aggs))?
            .iter_fields()
            .map(SeriesDomain::new_from_field)
            .collect::<Fallible<_>>()?,
        domain
            .margins
            .iter()
            .filter(|m| m.by.is_subset(&grouping_keys))
            .cloned()
            .map(|mut m| {
                m.by = materialize_grouping_keys(&m.by)?;
                m.invariant = None;
                Ok(m)
            })
            .collect::<Fallible<_>>()?,
    )
}

/// Build an ordinary group-by from the expressions certified by the caller.
pub(super) fn group_by_plan(
    input: DslPlan,
    keys: Vec<Expr>,
    aggs: Vec<Expr>,
    maintain_order: bool,
) -> DslPlan {
    DslPlan::GroupBy {
        input: Arc::new(input),
        keys,
        #[cfg(patch_polars)]
        predicates: vec![],
        aggs,
        apply: None,
        maintain_order,
        options: Arc::new(ORDINARY_GROUP_BY_OPTIONS),
    }
}

/// Express a subset of validated grouping keys as their materialized output columns.
/// Re-evaluating a grouping expression after aggregation may describe different groups.
pub(super) fn materialize_grouping_keys(by: &HashSet<Expr>) -> Fallible<HashSet<Expr>> {
    by.iter()
        .map(|key| Ok(col(key.clone().meta().output_name()?)))
        .collect()
}

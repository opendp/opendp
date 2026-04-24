use polars::prelude::*;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{
        strip_table_markers_from_schema, Database, DatabaseDomain, DslPlanDomain, FrameDomain,
        SeriesDomain,
    },
    error::*,
    metrics::{
        normalize_claim, normalize_claims, Binding, DatabaseIdDistance, FrameDistance, PolarsMetric,
        OwnerClaim, SymmetricIdDistance,
    },
};

use super::StableDslPlan;
use super::database_metric;
use super::truncate::has_any_truncation;

#[cfg(test)]
mod test;

fn output_domain_from_join(
    _left_domain: &DslPlanDomain,
    _right_domain: &DslPlanDomain,
    left_plan: &DslPlan,
    right_plan: &DslPlan,
    left_on: &[Expr],
    right_on: &[Expr],
    options: &std::sync::Arc<JoinOptions>,
) -> Fallible<DslPlanDomain> {
    let schema = LazyFrame::from(DslPlan::Join {
        input_left: std::sync::Arc::new(left_plan.clone()),
        input_right: std::sync::Arc::new(right_plan.clone()),
        left_on: left_on.to_vec(),
        right_on: right_on.to_vec(),
        predicates: vec![],
        options: options.clone(),
    })
    .collect_schema()?;

    let series_domains: Vec<SeriesDomain> = strip_table_markers_from_schema(&schema)
        .iter_fields()
        .map(SeriesDomain::new_from_field)
        .collect::<Fallible<_>>()?;

    // Joins can multiply rows from either branch, so existing row-level margin
    // descriptors are not generally preserved on the output.
    FrameDomain::new_with_margins(series_domains, Vec::new())
}

fn augmented_id_sites(
    bindings: Vec<Binding>,
    from_on: &[Expr],
    to_on: &[Expr],
) -> Vec<Binding> {
    let join_pairs = from_on
        .iter()
        .cloned()
        .zip(to_on.iter().cloned())
        .collect::<Vec<_>>();

    bindings
        .into_iter()
        .map(|mut site| {
            let mut extra = Vec::new();
            for expr in &site.exprs {
                for (from_expr, to_expr) in &join_pairs {
                    if expr == from_expr && !site.exprs.contains(to_expr) {
                        extra.push(to_expr.clone());
                    }
                }
            }
            site.exprs.extend(extra);
            site
        })
        .collect()
}

fn merge_owner_claims(left_claims: &[OwnerClaim], right_claims: &[OwnerClaim]) -> Vec<OwnerClaim> {
    let left_claims = if left_claims.is_empty() {
        vec![vec![]]
    } else {
        normalize_claims(left_claims)
    };
    let right_claims = if right_claims.is_empty() {
        vec![vec![]]
    } else {
        normalize_claims(right_claims)
    };

    normalize_claims(
        &left_claims
            .iter()
            .flat_map(|left| {
                right_claims.iter().map(move |right| {
                    let mut claim = left.clone();
                    claim.extend(right.clone());
                    normalize_claim(&claim)
                })
            })
            .collect::<Vec<_>>(),
    )
}

pub fn make_stable_database_join(
    input_domain: DatabaseDomain,
    input_metric: DatabaseIdDistance,
    plan: DslPlan,
) -> Fallible<
    Transformation<
        DatabaseDomain,
        DatabaseIdDistance,
        DslPlanDomain,
        FrameDistance<SymmetricIdDistance>,
    >,
> {
    let DslPlan::Join {
        input_left,
        input_right,
        left_on,
        right_on,
        predicates,
        options,
    } = plan.clone()
    else {
        return fallible!(MakeTransformation, "Expected join logical plan");
    };

    if !predicates.is_empty() {
        return fallible!(MakeTransformation, "join predicates are not supported");
    }

    let frame_metric = database_metric(&input_metric);
    if has_any_truncation(&frame_metric, input_left.as_ref())?
        || has_any_truncation(&frame_metric, input_right.as_ref())?
    {
        return fallible!(
            MakeTransformation,
            "joins are only supported before truncation; convert to event-level stability after truncation"
        );
    }

    let t_left = input_left
        .as_ref()
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let t_right = input_right
        .as_ref()
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;

    let (left_domain, left_metric): (_, FrameDistance<SymmetricIdDistance>) = t_left.output_space();
    let (right_domain, right_metric): (_, FrameDistance<SymmetricIdDistance>) =
        t_right.output_space();

    let left_private = !left_metric.0.owner_claims.is_empty() || !left_metric.0.active_bindings().is_empty();
    let right_private =
        !right_metric.0.owner_claims.is_empty() || !right_metric.0.active_bindings().is_empty();

    let output_metric = FrameDistance(SymmetricIdDistance {
        protect: left_metric.0.protect.clone(),
        bindings: augmented_id_sites(left_metric.0.bindings.clone(), &left_on, &right_on)
            .into_iter()
            .chain(augmented_id_sites(
                right_metric.0.bindings.clone(),
                &right_on,
                &left_on,
            ))
            .collect(),
        owner_claims: merge_owner_claims(&left_metric.0.owner_claims, &right_metric.0.owner_claims),
    });
    let map_left = left_private || !right_private;
    let output_domain = output_domain_from_join(
        &left_domain,
        &right_domain,
        input_left.as_ref(),
        input_right.as_ref(),
        &left_on,
        &right_on,
        &options,
    )?;
    let f_left = t_left.function.clone();
    let f_right = t_right.function.clone();
    let m_left = t_left.stability_map.clone();
    let m_right = t_right.stability_map.clone();

    Transformation::new(
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        Function::new_fallible(move |arg: &Database| {
            Ok(DslPlan::Join {
                input_left: std::sync::Arc::new(f_left.eval(arg)?),
                input_right: std::sync::Arc::new(f_right.eval(arg)?),
                left_on: left_on.clone(),
                right_on: right_on.clone(),
                predicates: vec![],
                options: options.clone(),
            })
        }),
        StabilityMap::new_fallible(move |d_in: &u32| {
            if map_left {
                m_left.eval(d_in)
            } else {
                m_right.eval(d_in)
            }
        }),
    )
}

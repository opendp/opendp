use polars::prelude::*;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{
        strip_table_markers_from_schema, Database, DatabaseDomain, DslPlanDomain, FrameDomain,
        SeriesDomain,
    },
    error::*,
    metrics::{
        Binding, DatabaseIdDistance, FrameDistance, PolarsMetric, SymmetricIdDistance,
        unique_id_expr,
    },
};

use super::StableDslPlan;

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
    private_sites: Vec<Binding>,
    private_on: &[Expr],
    public_on: &[Expr],
) -> Vec<Binding> {
    let join_pairs = private_on
        .iter()
        .cloned()
        .zip(public_on.iter().cloned())
        .collect::<Vec<_>>();

    private_sites
        .into_iter()
        .map(|mut site| {
            let mut extra = Vec::new();
            for expr in &site.exprs {
                for (private_expr, public_expr) in &join_pairs {
                    if expr == private_expr && !site.exprs.contains(public_expr) {
                        extra.push(public_expr.clone());
                    }
                }
            }
            site.exprs.extend(extra);
            site
        })
        .collect()
}

fn merge_private_id_sites(
    left_metric: &FrameDistance<SymmetricIdDistance>,
    right_metric: &FrameDistance<SymmetricIdDistance>,
) -> Fallible<Vec<Binding>> {
    let protected_label = left_metric.0.protect.clone();
    let left_active = left_metric.0.active_id_sites();
    let right_active = right_metric.0.active_id_sites();
    let left_expr = unique_id_expr(&left_active)?.ok_or_else(|| {
        err!(
            MakeTransformation,
            "left private branch is missing a protected identifier site"
        )
    })?;
    let right_expr = unique_id_expr(&right_active)?.ok_or_else(|| {
        err!(
            MakeTransformation,
            "right private branch is missing a protected identifier site"
        )
    })?;

    let mut merged_active_exprs = vec![left_expr];
    if !merged_active_exprs.contains(&right_expr) {
        merged_active_exprs.push(right_expr);
    }

    let mut output_sites = vec![Binding {
        space: protected_label.clone(),
        exprs: merged_active_exprs,
    }];

    output_sites.extend(
        left_metric
            .0
            .bindings
            .iter()
            .chain(right_metric.0.bindings.iter())
            .filter(|site| site.space != protected_label)
            .cloned(),
    );

    Ok(output_sites)
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

    let left_private = !left_metric.0.active_id_sites().is_empty();
    let right_private = !right_metric.0.active_id_sites().is_empty();

    let (output_metric, map_left) = match (left_private, right_private) {
        (true, false) => (
            FrameDistance(SymmetricIdDistance {
                protect: left_metric.0.protect.clone(),
                bindings: augmented_id_sites(left_metric.0.bindings.clone(), &left_on, &right_on),
            }),
            true,
        ),
        (false, true) => (
            FrameDistance(SymmetricIdDistance {
                protect: right_metric.0.protect.clone(),
                bindings: augmented_id_sites(right_metric.0.bindings.clone(), &right_on, &left_on),
            }),
            false,
        ),
        (true, true) => {
            let left_id = unique_id_expr(&left_metric.0.active_id_sites())?.ok_or_else(|| {
                err!(
                    MakeTransformation,
                    "left private branch is missing a protected identifier site"
                )
            })?;
            let right_id = unique_id_expr(&right_metric.0.active_id_sites())?.ok_or_else(|| {
                err!(
                    MakeTransformation,
                    "right private branch is missing a protected identifier site"
                )
            })?;
            if left_on.as_slice() != [left_id.clone()] || right_on.as_slice() != [right_id.clone()]
            {
                return fallible!(
                    MakeTransformation,
                    "private-private joins currently require joining on the active protected identifier"
                );
            }
            (
                FrameDistance(SymmetricIdDistance {
                    protect: left_metric.0.protect.clone(),
                    bindings: merge_private_id_sites(&left_metric, &right_metric)?,
                }),
                true,
            )
        }
        (false, false) => {
            return fallible!(
                MakeTransformation,
                "database-aware joins require at least one branch to carry the protected identifier"
            );
        }
    };
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

use std::collections::HashSet;

use polars::prelude::*;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{
        strip_table_markers_from_schema, Database, DatabaseDomain, DslPlanDomain, FrameDomain,
        SeriesDomain,
    },
    error::*,
    metrics::{
        choose_owner_claim, expr_equivalent_under_bindings, normalize_bindings,
        normalize_claim_with_bindings, normalize_claims_with_bindings, Binding, DatabaseIdDistance,
        FrameDistance, OwnerClaim, PolarsMetric, SymmetricIdDistance,
        transportable_owner_factor,
    },
};

use super::StableDslPlan;
use super::database_metric;
use super::truncate::has_any_truncation;

#[cfg(test)]
mod test;

#[derive(Clone, Copy)]
enum JoinBranch {
    Left,
    Right,
}

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

fn augment_bindings_with_join_equalities(
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

fn expr_root_names(expr: &Expr) -> HashSet<polars::prelude::PlSmallStr> {
    expr.clone()
        .meta()
        .undo_aliases()
        .meta()
        .root_names()
        .into_iter()
        .collect()
}

fn expr_visible_in_output(
    expr: &Expr,
    output_names: &HashSet<polars::prelude::PlSmallStr>,
) -> bool {
    expr_root_names(expr)
        .into_iter()
        .all(|name| output_names.contains(&name))
}

fn is_plain_column_expr(expr: &Expr) -> bool {
    matches!(expr.clone().meta().undo_aliases(), Expr::Column(_))
}

fn structured_right_expr_has_branch_collision(
    expr: &Expr,
    left_names: &HashSet<polars::prelude::PlSmallStr>,
    right_names: &HashSet<polars::prelude::PlSmallStr>,
) -> bool {
    !is_plain_column_expr(expr)
        && expr_root_names(expr)
            .into_iter()
            .any(|root| left_names.contains(&root) && right_names.contains(&root))
}

fn rewrite_join_expr_to_output(
    branch: JoinBranch,
    expr: &Expr,
    left_on: &[Expr],
    right_on: &[Expr],
    left_names: &HashSet<polars::prelude::PlSmallStr>,
    right_names: &HashSet<polars::prelude::PlSmallStr>,
    output_names: &HashSet<polars::prelude::PlSmallStr>,
) -> Fallible<Expr> {
    if matches!(branch, JoinBranch::Right)
        && structured_right_expr_has_branch_collision(expr, left_names, right_names)
    {
        return fallible!(
            MakeTransformation,
            "could not safely rewrite structured right-branch expression into join output: {:?}",
            expr
        );
    }

    if let Some(expr) = left_on.iter().zip(right_on.iter()).find_map(|(left, right)| {
        if *expr == *left {
            if expr_visible_in_output(left, output_names) {
                Some(left.clone())
            } else if expr_visible_in_output(right, output_names) {
                Some(right.clone())
            } else {
                None
            }
        } else if *expr == *right {
            if expr_visible_in_output(left, output_names) {
                Some(left.clone())
            } else if expr_visible_in_output(right, output_names) {
                Some(right.clone())
            } else {
                None
            }
        } else {
            None
        }
    }) {
        return Ok(expr);
    }

    let root_names = expr_root_names(expr).into_iter().collect::<Vec<_>>();
    if root_names.len() != 1 {
        return Ok(expr.clone());
    }
    let root = &root_names[0];
    let suffixed = format!("{root}_right");
    match branch {
        JoinBranch::Left => output_names.contains(root).then(|| expr.clone()).ok_or_else(|| {
            err!(
                MakeTransformation,
                "could not map left-branch expression into join output: {:?}",
                expr
            )
        }),
        JoinBranch::Right => {
            if output_names.contains(root) && !left_names.contains(root) {
                Ok(expr.clone())
            } else if output_names.contains(suffixed.as_str()) && right_names.contains(root) {
                if is_plain_column_expr(expr) {
                    Ok(col(&suffixed))
                } else {
                    fallible!(
                        MakeTransformation,
                        "could not safely rewrite structured right-branch expression into join output: {:?}",
                        expr
                    )
                }
            } else if output_names.contains(root) && !right_names.contains(root) {
                Ok(expr.clone())
            } else {
                fallible!(
                    MakeTransformation,
                    "could not map right-branch expression into join output: {:?}",
                    expr
                )
            }
        }
    }
}

fn rewrite_exprs_to_output(
    branch: JoinBranch,
    exprs: impl IntoIterator<Item = Expr>,
    left_on: &[Expr],
    right_on: &[Expr],
    left_names: &HashSet<polars::prelude::PlSmallStr>,
    right_names: &HashSet<polars::prelude::PlSmallStr>,
    output_names: &HashSet<polars::prelude::PlSmallStr>,
) -> Fallible<Vec<Expr>> {
    exprs.into_iter()
        .map(|expr| {
            rewrite_join_expr_to_output(
                branch,
                &expr,
                left_on,
                right_on,
                left_names,
                right_names,
                output_names,
            )
        })
        .collect()
}

fn rewrite_binding_to_output(
    branch: JoinBranch,
    binding: Binding,
    left_on: &[Expr],
    right_on: &[Expr],
    left_names: &HashSet<polars::prelude::PlSmallStr>,
    right_names: &HashSet<polars::prelude::PlSmallStr>,
    output_names: &HashSet<polars::prelude::PlSmallStr>,
) -> Fallible<Binding> {
    Ok(Binding {
        exprs: rewrite_exprs_to_output(
            branch,
            binding.exprs,
            left_on,
            right_on,
            left_names,
            right_names,
            output_names,
        )?,
        space: binding.space,
    })
}

fn rewrite_claims_to_output(
    branch: JoinBranch,
    claims: &[OwnerClaim],
    left_on: &[Expr],
    right_on: &[Expr],
    left_names: &HashSet<polars::prelude::PlSmallStr>,
    right_names: &HashSet<polars::prelude::PlSmallStr>,
    output_names: &HashSet<polars::prelude::PlSmallStr>,
) -> Fallible<Vec<OwnerClaim>> {
    claims
        .iter()
        .map(|claim| {
            rewrite_exprs_to_output(
                branch,
                claim.iter().cloned(),
                left_on,
                right_on,
                left_names,
                right_names,
                output_names,
            )?
            .into_iter()
            .map(|expr| {
                transportable_owner_factor(&expr).ok_or_else(|| {
                    err!(
                        MakeTransformation,
                        "owner claim factors transported through joins must resolve to simple output columns; materialize or alias derived identifiers first"
                    )
                })
            })
            .collect()
        })
        .collect()
}

fn choose_singleton_owner_claim(claims: &[OwnerClaim]) -> Option<OwnerClaim> {
    choose_owner_claim(claims).filter(|claim| claim.len() == 1)
}

fn has_protected_identity(metric: &SymmetricIdDistance) -> bool {
    !metric.active_bindings().is_empty()
}

fn join_aligns_owner_claims(
    left_claims: &[OwnerClaim],
    right_claims: &[OwnerClaim],
    left_join_key: &Expr,
    right_join_key: &Expr,
    left_bindings: &[Binding],
    right_bindings: &[Binding],
    output_bindings: &[Binding],
    protect: &str,
) -> bool {
    let Some(left_claim) = choose_singleton_owner_claim(left_claims) else {
        return false;
    };
    let Some(right_claim) = choose_singleton_owner_claim(right_claims) else {
        return false;
    };

    let left_claim = normalize_claim_with_bindings(&left_claim, output_bindings, protect);
    let right_claim = normalize_claim_with_bindings(&right_claim, output_bindings, protect);
    left_claim.len() == 1
        && right_claim.len() == 1
        && expr_equivalent_under_bindings(
            &left_claim[0],
            left_join_key,
            left_bindings,
            protect,
        )
        && expr_equivalent_under_bindings(
            &right_claim[0],
            right_join_key,
            right_bindings,
            protect,
        )
        && left_claim == right_claim
}

fn merge_owner_claims(
    left_claims: &[OwnerClaim],
    right_claims: &[OwnerClaim],
    bindings: &[Binding],
    protect: &str,
) -> Vec<OwnerClaim> {
    let left_claims = if left_claims.is_empty() {
        vec![vec![]]
    } else {
        normalize_claims_with_bindings(left_claims, bindings, protect)
    };
    let right_claims = if right_claims.is_empty() {
        vec![vec![]]
    } else {
        normalize_claims_with_bindings(right_claims, bindings, protect)
    };

    normalize_claims_with_bindings(
        &left_claims
            .iter()
            .flat_map(|left| {
                right_claims.iter().map(move |right| {
                    let mut claim = left.clone();
                    claim.extend(right.clone());
                    claim
                })
            })
            .collect::<Vec<_>>(),
        bindings,
        protect,
    )
}

fn has_protected_ownership(metric: &SymmetricIdDistance) -> bool {
    // Row-charge semantics are stricter than protected identity:
    // a lookup table may carry protected bindings but still have no protected
    // owner factors. Use ownership, not identity, to decide whether both join
    // branches are private under the conservative MVP join policy.
    metric.owner_claims.iter().any(|claim| !claim.is_empty())
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

    let output_domain = output_domain_from_join(
        &left_domain,
        &right_domain,
        input_left.as_ref(),
        input_right.as_ref(),
        &left_on,
        &right_on,
        &options,
    )?;
    let output_names = output_domain
        .schema()
        .iter_names()
        .cloned()
        .collect::<HashSet<_>>();
    let left_names = left_domain
        .schema()
        .iter_names()
        .cloned()
        .collect::<HashSet<_>>();
    let right_names = right_domain
        .schema()
        .iter_names()
        .cloned()
        .collect::<HashSet<_>>();
    let output_bindings = normalize_bindings(
        &augment_bindings_with_join_equalities(left_metric.0.bindings.clone(), &left_on, &right_on)
            .into_iter()
            .map(|binding| {
                rewrite_binding_to_output(
                    JoinBranch::Left,
                    binding,
                    &left_on,
                    &right_on,
                    &left_names,
                    &right_names,
                    &output_names,
                )
            })
            .collect::<Fallible<Vec<_>>>()?
            .into_iter()
            .chain(augment_bindings_with_join_equalities(
                right_metric.0.bindings.clone(),
                &right_on,
                &left_on,
            )
            .into_iter()
            .map(|binding| {
                rewrite_binding_to_output(
                    JoinBranch::Right,
                    binding,
                    &left_on,
                    &right_on,
                    &left_names,
                    &right_names,
                    &output_names,
                )
            })
            .collect::<Fallible<Vec<_>>>()?
            .into_iter())
            .collect::<Vec<_>>(),
    );
    let left_claims = rewrite_claims_to_output(
        JoinBranch::Left,
        &left_metric.0.owner_claims,
        &left_on,
        &right_on,
        &left_names,
        &right_names,
        &output_names,
    )?;
    let right_claims = rewrite_claims_to_output(
        JoinBranch::Right,
        &right_metric.0.owner_claims,
        &left_on,
        &right_on,
        &left_names,
        &right_names,
        &output_names,
    )?;

    let _identity_context = (
        has_protected_identity(&left_metric.0),
        has_protected_identity(&right_metric.0),
    );
    let left_private = has_protected_ownership(&left_metric.0);
    let right_private = has_protected_ownership(&right_metric.0);
    let rewritten_left_on = rewrite_exprs_to_output(
        JoinBranch::Left,
        left_on.iter().cloned(),
        &left_on,
        &right_on,
        &left_names,
        &right_names,
        &output_names,
    )?;
    let rewritten_right_on = rewrite_exprs_to_output(
        JoinBranch::Right,
        right_on.iter().cloned(),
        &left_on,
        &right_on,
        &left_names,
        &right_names,
        &output_names,
    )?;

    if left_private
        && right_private
        && !(rewritten_left_on.len() == 1
            && rewritten_right_on.len() == 1
            && join_aligns_owner_claims(
                &left_claims,
                &right_claims,
                &rewritten_left_on[0],
                &rewritten_right_on[0],
                &left_metric.0.bindings,
                &right_metric.0.bindings,
                &output_bindings,
                &left_metric.0.protect,
            ))
    {
        return fallible!(
            MakeTransformation,
            "when both join branches are private, the join must align the active owner claim"
        );
    }

    let output_metric = FrameDistance(SymmetricIdDistance {
        protect: left_metric.0.protect.clone(),
        bindings: output_bindings.clone(),
        owner_claims: merge_owner_claims(
            &left_claims,
            &right_claims,
            &output_bindings,
            &left_metric.0.protect,
        ),
    });
    let map_left = left_private || !right_private;
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

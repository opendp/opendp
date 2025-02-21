use std::{collections::BTreeSet, sync::Arc};

use polars::prelude::PlSmallStr;
use polars::prelude::{JoinOptions, JoinType};
use polars_plan::{
    dsl::{Expr, Operator},
    plans::DslPlan,
    prelude::GroupbyOptions,
    utils::expr_output_name,
};

use crate::{measurements::expr_noise::NoisePlugin, polars::match_trusted_plugin};

use super::Fallible;

#[derive(Clone)]
pub enum KeySanitizer {
    Filter(Expr),
    Join {
        keys: Arc<DslPlan>,
        how: JoinType,
        left_on: Vec<Expr>,
        right_on: Vec<Expr>,
        options: Arc<JoinOptions>,
        fill_null: Option<Vec<Expr>>,
    },
}

pub(crate) struct MatchGroupBy {
    pub input: DslPlan,
    pub group_by: Vec<Expr>,
    pub aggs: Vec<Expr>,
    pub key_sanitizer: Option<KeySanitizer>,
}

pub(crate) fn match_group_by(mut plan: DslPlan) -> Fallible<Option<MatchGroupBy>> {
    let key_sanitizer = match plan {
        DslPlan::Filter { input, predicate } => {
            plan = input.as_ref().clone();
            Some(KeySanitizer::Filter(predicate))
        }
        DslPlan::Join {
            input_left,
            input_right,
            left_on,
            right_on,
            predicates,
            options,
        } => {
            if !predicates.is_empty() {
                return fallible!(
                    MakeMeasurement,
                    "predicates are not supported in key-privatization joins"
                );
            }
            let how = options.as_ref().args.how.clone();
            let (keys, keys_on, input_on) = match how {
                JoinType::Left => {
                    plan = input_right.as_ref().clone();
                    (input_left, &left_on, &right_on)
                }
                JoinType::Right => {
                    plan = input_left.as_ref().clone();
                    (input_right, &right_on, &left_on)
                }
                _ => {
                    return fallible!(
                        MakeMeasurement,
                        "only left or right joins can be used to privatize key-sets"
                    )
                }
            };

            let keys_on_columns = match_grouping_columns(keys_on.clone())
                .map_err(|_| err!(MakeMeasurement, "join on must consist of column exprs"))?;
            let input_on_columns = match_grouping_columns(input_on.clone())
                .map_err(|_| err!(MakeMeasurement, "join on must consist of column exprs"))?;

            if input_on_columns.len() != keys_on_columns.len() {
                return fallible!(
                    MakeMeasurement,
                    "left_on and right_on must have same number of join keys"
                );
            }

            let label_schema = keys.compute_schema()?;
            if keys_on_columns != label_schema.iter_names().cloned().collect::<BTreeSet<_>>() {
                return fallible!(
                    MakeMeasurement,
                    "label dataframe columns must match join keys"
                );
            }

            Some(KeySanitizer::Join {
                keys,
                how,
                left_on,
                right_on,
                options,
                fill_null: None,
            })
        }
        _ => None,
    };

    let DslPlan::GroupBy {
        input,
        keys,
        aggs,
        apply,
        maintain_order,
        options,
    } = plan
    else {
        return Ok(None);
    };

    if options.as_ref() != &GroupbyOptions::default() {
        return fallible!(MakeMeasurement, "Unsupported options in logical plan. Do not optimize the lazyframe passed into the constructor. Options should be default, but are {:?}", options);
    }

    if apply.is_some() {
        return fallible!(MakeMeasurement, "Apply is not supported in logical plan");
    }

    if maintain_order {
        return fallible!(MakeMeasurement, "The order of keys is sensitive");
    }

    Ok(Some(MatchGroupBy {
        input: Arc::unwrap_or_clone(input),
        group_by: keys,
        aggs,
        key_sanitizer,
    }))
}

pub fn match_grouping_columns(keys: Vec<Expr>) -> Fallible<BTreeSet<PlSmallStr>> {
    Ok(keys
        .iter()
        .map(|e| {
            Ok(match e {
                Expr::Column(name) => vec![name.clone()],
                Expr::Columns(names) => names.to_vec(),
                e => {
                    return fallible!(
                        MakeMeasurement,
                        "Expected column expression in keys, found {:?}",
                        e
                    )
                }
            })
        })
        .collect::<Fallible<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}

pub(super) fn find_len_expr(
    exprs: &Vec<Expr>,
    name: Option<&str>,
) -> Fallible<(String, NoisePlugin)> {
    // only keep expressions that compute the length
    (exprs.iter())
        .find_map(|e| is_len_expr(e, name))
        .ok_or_else(|| {
            if let Some(name) = name {
                err!(
                    MakeMeasurement,
                    "stable key release expects a DP length expression with name: {:?}",
                    name
                )
            } else {
                err!(
                    MakeMeasurement,
                    "stable key release requires a `dp.len()` expression"
                )
            }
        })
}

fn is_len_expr(expr: &Expr, name: Option<&str>) -> Option<(String, NoisePlugin)> {
    let output_name = expr_output_name(expr).ok()?;

    // check if the expression matches the expected name
    if let Some(name) = name {
        if name != output_name.as_str() {
            return None;
        }
    }
    // remove any aliasing in the expression
    let expr = expr.clone().meta().undo_aliases();

    let (inputs, args) = match_trusted_plugin::<NoisePlugin>(&expr).ok().flatten()?;

    if let Expr::Len = &inputs[0] {
        Some((output_name.to_string(), args))
    } else {
        None
    }
}

pub(crate) fn is_threshold_predicate(expr: Expr) -> Option<(String, u32)> {
    let Expr::BinaryExpr { left, op, right } = expr else {
        return None;
    };

    use Operator::{Gt, Lt};

    let (name, value) = match (left.as_ref(), op, right.as_ref()) {
        (Expr::Column(name), Gt, Expr::Literal(value)) => (name, value),
        (Expr::Literal(value), Lt, Expr::Column(name)) => (name, value),
        _ => return None,
    };

    Some((name.to_string(), value.to_any_value()?.extract()?))
}

//! Decompose group-by expressions into private aggregate leaves and
//! deterministic postprocessing on their materialized results.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use polars::prelude::{DslPlan, Expr, PlSmallStr, Selector, all, col};
use polars_plan::prelude::ProjectionOptions;

use crate::error::Fallible;
use crate::measurements::{
    expr_dp_counting_query::{DPCountShim, DPLenShim, DPNUniqueShim, DPNullCountShim},
    expr_dp_frame_len::DPFrameLenShim,
    expr_dp_mean::DPMeanShim,
    expr_dp_median::DPMedianShim,
    expr_dp_quantile::DPQuantileShim,
    expr_dp_sum::DPSumShim,
    expr_noise::match_noise,
};
use crate::polars::{apply_plugin, dp_len, literal_value_of, match_shim};

use super::{KeySanitizer, is_threshold_predicate};

/// The private expressions in aggs run within GroupBy. Postprocessing only
/// reads their *materialized* output columns, so it cannot resample noise.
pub(super) struct PrivateAggPlan {
    pub aggs: Vec<Expr>,
    postprocess: Vec<Expr>,
    outputs: Vec<String>,
    hidden: Vec<String>,
    count_aliases: HashMap<String, String>,
    pub count: Option<String>,
}

impl PrivateAggPlan {
    fn unchanged(aggs: Vec<Expr>) -> Self {
        Self {
            aggs,
            postprocess: Vec::new(),
            outputs: Vec::new(),
            hidden: Vec::new(),
            count_aliases: HashMap::new(),
            count: None,
        }
    }

    pub fn new(
        aggs: Vec<Expr>,
        group_keys: &[Expr],
        sanitizer: &Option<KeySanitizer>,
    ) -> Fallible<Self> {
        let mut has_implicit_mean = false;
        for expr in &aggs {
            has_implicit_mean |= implicit_mean(expr)?.is_some();
        }
        if !has_implicit_mean && !aggs.iter().any(can_decompose) && !has_duplicates(&aggs)? {
            return Ok(Self::unchanged(aggs));
        }

        // A filter over a reconstructed output would need predicate lowering.
        // For now, optimize only when its predicate is a threshold on a
        // directly released, implicit-scale count.
        if let Some(KeySanitizer::Filter(predicate)) = sanitizer {
            let mut safe = false;
            if let Some((name, _)) = is_threshold_predicate(predicate.clone()) {
                for expr in &aggs {
                    if expr.clone().meta().output_name()?.as_str() == name
                        && implicit_count(expr)?
                    {
                        safe = true;
                    }
                }
            }
            if !safe {
                return Ok(Self::unchanged(aggs));
            }
        }

        let outputs = aggs
            .iter()
            .map(|e| e.clone().meta().output_name().map(|s| s.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        let mut used = outputs.iter().cloned().collect::<HashSet<_>>();
        for key in group_keys {
            match key {
                Expr::Selector(Selector::ByName { names, .. }) => {
                    used.extend(names.iter().map(|n| n.to_string()));
                }
                key => {
                    used.insert(key.clone().meta().output_name()?.to_string());
                }
            }
        }
        if let Some(KeySanitizer::Join { keys, .. }) = sanitizer {
            used.extend(keys.compute_schema()?.iter_names().map(|n| n.to_string()));
        }

        // Preserve the existing cross-column count sharing for top-level
        // implicit-scale means. Nested private leaves are *not* deduplicated:
        // that is a separate workload optimization.
        let mut count = None;
        if has_implicit_mean {
            for (expr, name) in aggs.iter().zip(&outputs) {
                if implicit_count(expr)? {
                    count = Some(name.clone());
                    break;
                }
            }
        }
        let synthesized = has_implicit_mean && count.is_none();
        if synthesized {
            count = Some(unique_name(&mut used, "count"));
        }

        let mut private_aggs = Vec::with_capacity(aggs.len() + usize::from(synthesized));
        let mut postprocess = Vec::new();
        let mut hidden = Vec::new();
        let mut count_aliases = HashMap::new();
        let mut have_count = false;

        for (expr, name) in aggs.into_iter().zip(&outputs) {
            if has_implicit_mean && implicit_count(&expr)? {
                let shared_count = count.as_ref().expect("a mean requires a count");
                if name == shared_count && !have_count {
                    private_aggs.push(expr);
                    have_count = true;
                } else {
                    postprocess.push(col(shared_count.as_str()).alias(name.as_str()));
                    count_aliases.insert(name.clone(), shared_count.clone());
                }
            } else if let Some([input, lower, upper, scale]) = implicit_mean(&expr)? {
                let sum_name = unique_name(&mut used, "sum");
                let sum = apply_plugin(
                    vec![input, lower, upper, scale],
                    expr.clone().meta().undo_aliases(),
                    DPSumShim,
                );
                private_aggs.push(sum.alias(sum_name.as_str()));
                postprocess.push(
                    Expr::BinaryExpr {
                        left: Arc::new(col(sum_name.as_str())),
                        op: polars::prelude::Operator::TrueDivide,
                        right: Arc::new(col(count.as_deref().expect("a mean requires a count"))),
                    }
                    .alias(name.as_str()),
                );
                hidden.push(sum_name);
            } else if can_decompose(&expr) {
                // Preserve one independent private measurement per leaf.
                // Only deterministic postprocessing is deferred.
                let reconstruction = decompose(
                    expr.meta().undo_aliases(),
                    &mut private_aggs,
                    &mut hidden,
                    &mut used,
                );
                postprocess.push(reconstruction.alias(name.as_str()));
            } else {
                private_aggs.push(expr);
            }
        }
        if synthesized {
            let name = count.as_ref().expect("synthesized count has a name");
            private_aggs.push(dp_len(None, false).alias(name.as_str()));
            hidden.push(name.clone());
        }

        // Only one private measurement is constructed per distinct implicit-
        // scale leaf. All references to removed leaves point to its noisy
        // materialized column, including after an empty-group fill.
        private_aggs = deduplicate(
            private_aggs,
            &mut postprocess,
            &hidden,
            &mut count_aliases,
        )?;

        Ok(Self {
            aggs: private_aggs,
            postprocess,
            outputs,
            hidden,
            count_aliases,
            count,
        })
    }

    /// Resolve a user-visible count alias to the one noisy intermediate.
    pub fn raw_count(&self, output_name: &str) -> String {
        self.count_aliases
            .get(output_name)
            .cloned()
            .unwrap_or_else(|| output_name.to_string())
    }

    /// Postprocess only after key sanitization and any empty-group fills.
    /// Preserve public-key join columns and the original aggregate ordering.
    pub fn finish(&self, input: DslPlan) -> DslPlan {
        if self.postprocess.is_empty() {
            return input;
        }

        let input = DslPlan::HStack {
            input: Arc::new(input),
            exprs: self.postprocess.clone(),
            options: projection_options(),
        };

        let excluded = self
            .outputs
            .iter()
            .chain(&self.hidden)
            .map(|s| PlSmallStr::from(s.as_str()))
            .collect::<Vec<_>>();
        let mut exprs = vec![all().exclude_cols(excluded).as_expr()];
        exprs.extend(self.outputs.iter().map(|name| col(name.as_str())));
        DslPlan::Select {
            input: Arc::new(input),
            expr: exprs,
            options: projection_options(),
        }
    }
}

/// Lift only supported postprocessing; leave arbitrary private expressions
/// opaque, and never treat data-dependent column references as public.
fn can_decompose(expr: &Expr) -> bool {
    match expr {
        Expr::Alias(inner, _) => can_decompose(inner),
        Expr::BinaryExpr { .. } | Expr::Cast { .. } => has_leaf(expr),
        _ => false,
    }
}

fn has_leaf(expr: &Expr) -> bool {
    match expr {
        Expr::Alias(inner, _) | Expr::Cast { expr: inner, .. } => has_leaf(inner),
        Expr::BinaryExpr { left, right, .. } => has_leaf(left) || has_leaf(right),
        Expr::Literal(_) => false,
        _ => true,
    }
}

/// Produce private leaves and a deterministic reconstruction expression over
/// their *materialized* output columns. Leaf deduplication is a later pass.
fn decompose(
    expr: Expr,
    aggs: &mut Vec<Expr>,
    hidden: &mut Vec<String>,
    used: &mut HashSet<String>,
) -> Expr {
    match expr {
        Expr::Alias(inner, name) => decompose(*inner, aggs, hidden, used).alias(name),
        Expr::BinaryExpr { left, op, right } => Expr::BinaryExpr {
            left: Arc::new(decompose(*left, aggs, hidden, used)),
            op,
            right: Arc::new(decompose(*right, aggs, hidden, used)),
        },
        Expr::Cast {
            expr,
            dtype,
            options,
        } => Expr::Cast {
            expr: Arc::new(decompose(*expr, aggs, hidden, used)),
            dtype,
            options,
        },
        Expr::Literal(_) => expr,
        expr => {
            let name = unique_name(used, "agg");
            aggs.push(expr.alias(name.as_str()));
            hidden.push(name.clone());
            col(name.as_str())
        }
    }
}

/// Semantic identity of a supported mechanism, not of its output alias.
/// Comparing the original shim's arguments avoids relying on plugin-object
/// pointer identity, and includes preprocessing, bounds and dtype-affecting
/// parameters. All leaves share the same group-by input domain and budget.
#[derive(PartialEq)]
struct LeafKey {
    mechanism: &'static str,
    args: Vec<Expr>,
}

fn leaf_key(expr: &Expr) -> Fallible<Option<LeafKey>> {
    let expr = expr.clone().meta().undo_aliases();

    macro_rules! keyed {
        ($shim:ty, $arity:literal, $scale:literal, $name:literal) => {
            if let Some(args) = match_shim::<$shim, $arity>(&expr)? {
                if literal_value_of::<f64>(&args[$scale])?.is_none() {
                    return Ok(Some(LeafKey {
                        mechanism: $name,
                        args: args.into_iter().collect(),
                    }));
                }
                return Ok(None);
            }
        };
    }

    keyed!(DPFrameLenShim, 2, 0, "frame_len");
    keyed!(DPLenShim, 3, 1, "len");
    keyed!(DPCountShim, 3, 1, "count");
    keyed!(DPNullCountShim, 3, 1, "null_count");
    keyed!(DPNUniqueShim, 3, 1, "n_unique");
    keyed!(DPSumShim, 4, 3, "sum");
    keyed!(DPMeanShim, 4, 3, "mean");
    keyed!(DPMedianShim, 3, 2, "median");
    keyed!(DPQuantileShim, 4, 3, "quantile");

    if let Some((input, scale)) = match_noise(&expr)? {
        if scale.is_none() {
            return Ok(Some(LeafKey {
                mechanism: "noise",
                args: vec![input.clone()],
            }));
        }
    }

    // Do not merge explicit-scale measurements, opaque randomized plugins,
    // or expressions whose privacy semantics have not been certified here.
    Ok(None)
}

fn has_duplicates(aggs: &[Expr]) -> Fallible<bool> {
    let mut seen = Vec::new();
    for expr in aggs {
        if let Some(key) = leaf_key(expr)? {
            if seen.contains(&key) {
                return Ok(true);
            }
            seen.push(key);
        }
    }
    Ok(false)
}

/// Rewrite only deterministic reconstruction expressions created by this
/// module. Never traverse and clone a randomized aggregate expression.
fn replace_columns(expr: Expr, replacements: &[(String, String)]) -> Expr {
    match expr {
        Expr::Column(name) => {
            let new_name = replacements
                .iter()
                .find(|(from, _)| from == name.as_str())
                .map(|(_, to)| to.as_str())
                .unwrap_or(name.as_str());
            col(new_name)
        }
        Expr::Alias(inner, name) => {
            replace_columns(*inner, replacements).alias(name)
        }
        Expr::BinaryExpr { left, op, right } => Expr::BinaryExpr {
            left: Arc::new(replace_columns(*left, replacements)),
            op,
            right: Arc::new(replace_columns(*right, replacements)),
        },
        Expr::Cast { expr, dtype, options } => Expr::Cast {
            expr: Arc::new(replace_columns(*expr, replacements)),
            dtype,
            options,
        },
        // Only the cases above and literals are generated by decompose().
        other => other,
    }
}

fn deduplicate(
    aggs: Vec<Expr>,
    postprocess: &mut Vec<Expr>,
    hidden: &[String],
    aliases: &mut HashMap<String, String>,
) -> Fallible<Vec<Expr>> {
    let mut seen: Vec<(LeafKey, String)> = Vec::new();
    let mut unique = Vec::with_capacity(aggs.len());
    let mut substitutions = Vec::new();
    let mut output_aliases = Vec::new();

    for expr in aggs {
        let key = leaf_key(&expr)?;
        if let Some(key) = key {
            if let Some((_, existing)) = seen.iter().find(|(other, _)| other == &key) {
                let name = expr.clone().meta().output_name()?.to_string();
                aliases.insert(name.clone(), existing.clone());
                if hidden.iter().any(|hidden| hidden == &name) {
                    substitutions.push((name, existing.clone()));
                } else {
                    output_aliases.push(col(existing.as_str()).alias(name.as_str()));
                }
                continue;
            }
            let name = expr.clone().meta().output_name()?.to_string();
            seen.push((key, name));
        }
        unique.push(expr);
    }

    for expr in postprocess.iter_mut() {
        *expr = replace_columns(expr.clone(), &substitutions);
    }
    postprocess.extend(output_aliases);
    Ok(unique)
}

fn projection_options() -> ProjectionOptions {
    ProjectionOptions {
        run_parallel: true,
        duplicate_check: true,
        should_broadcast: true,
    }
}

fn unique_name(used: &mut HashSet<String>, kind: &str) -> String {
    for index in 0.. {
        let name = format!("__opendp_{kind}_{index}");
        if used.insert(name.clone()) {
            return name;
        }
    }
    unreachable!()
}

/// Only a direct column expression is guaranteed to count the same records
/// as other columns in this group. Filtered expressions are deliberately not
/// fused, because their row populations may differ.
fn implicit_mean(expr: &Expr) -> Fallible<Option<[Expr; 4]>> {
    let expr = expr.clone().meta().undo_aliases();
    let Some([input, lower, upper, scale]) = match_shim::<DPMeanShim, 4>(&expr)? else {
        return Ok(None);
    };
    if !matches!(&input, Expr::Column(_)) || literal_value_of::<f64>(&scale)?.is_some() {
        return Ok(None);
    }
    Ok(Some([input, lower, upper, scale]))
}

/// An implicit-scale unsigned frame count is interchangeable with the
/// denominator of a direct-column DP mean. Explicit or signed noise requests
/// retain their original semantics.
fn implicit_count(expr: &Expr) -> Fallible<bool> {
    let expr = expr.clone().meta().undo_aliases();
    if let Some([scale, signed]) = match_shim::<DPFrameLenShim, 2>(&expr)? {
        let unsigned = matches!(signed, Expr::Literal(value) if value.bool() == Some(false));
        return Ok(unsigned && literal_value_of::<f64>(&scale)?.is_none());
    }
    if let Some((input, scale)) = match_noise(&expr)? {
        return Ok(matches!(input, Expr::Len) && scale.is_none());
    }
    Ok(false)
}

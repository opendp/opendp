//! Decompose compatible group-by aggregates into noisy intermediate statistics
//! and postprocessing on those statistics.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use polars::prelude::{DslPlan, Expr, PlSmallStr, Selector, all, col};
use polars_plan::prelude::ProjectionOptions;

use crate::error::Fallible;
use crate::measurements::{
    expr_dp_frame_len::DPFrameLenShim,
    expr_dp_mean::DPMeanShim,
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
        if !has_implicit_mean {
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

        // Prefer an existing public output of dp.len(), if compatible, so
        // explicit count queries and means consume the same random variable.
        let mut count = None;
        for (expr, name) in aggs.iter().zip(&outputs) {
            if implicit_count(expr)? {
                count = Some(name.clone());
                break;
            }
        }
        let synthesized = count.is_none();
        let count = count.unwrap_or_else(|| unique_name(&mut used, "count"));

        let mut private_aggs = Vec::with_capacity(aggs.len() + usize::from(synthesized));
        let mut postprocess = Vec::new();
        let mut hidden = Vec::new();
        let mut count_aliases = HashMap::new();
        let mut have_count = false;

        for (expr, name) in aggs.into_iter().zip(&outputs) {
            if implicit_count(&expr)? {
                if name == &count && !have_count {
                    private_aggs.push(expr);
                    have_count = true;
                } else {
                    postprocess.push(col(count.as_str()).alias(name.as_str()));
                    count_aliases.insert(name.clone(), count.clone());
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
                        right: Arc::new(col(count.as_str())),
                    }
                    .alias(name.as_str()),
                );
                hidden.push(sum_name);
            } else {
                private_aggs.push(expr);
            }
        }
        if synthesized {
            private_aggs.push(dp_len(None, false).alias(count.as_str()));
            hidden.push(count.clone());
        }

        Ok(Self {
            aggs: private_aggs,
            postprocess,
            outputs,
            hidden,
            count_aliases,
            count: Some(count),
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

//! Decompose group-by expressions into private aggregate leaves and
//! deterministic postprocessing on their materialized results.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use polars::prelude::{DslPlan, Expr, Operator, PlSmallStr, Selector, all, col, lit};
use polars_plan::prelude::ProjectionOptions;

use crate::domains::SeriesDomain;
use crate::error::Fallible;
use crate::measurements::{
    expr_dp_counting_query::{DPCountShim, DPLenShim, DPNUniqueShim, DPNullCountShim},
    expr_dp_frame_len::DPFrameLenShim,
    expr_dp_mean::DPMeanShim,
    expr_dp_median::DPMedianShim,
    expr_dp_quantile::DPQuantileShim,
    expr_dp_sum::DPSumShim,
    expr_dp_var::DPVarShim,
    expr_noise::{NoisePlugin, match_noise},
};
use crate::polars::{apply_plugin, dp_len, literal_value_of, match_shim};

use super::{KeySanitizer, is_threshold_predicate};
use super::matching::{find_len_expr, is_len_expr};

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

    #[cfg(test)]
    pub fn new(
        aggs: Vec<Expr>,
        group_keys: &[Expr],
        sanitizer: &Option<KeySanitizer>,
    ) -> Fallible<Self> {
        Self::new_with_columns(aggs, group_keys, sanitizer, &[])
    }

    pub fn new_with_columns(
        aggs: Vec<Expr>,
        group_keys: &[Expr],
        sanitizer: &Option<KeySanitizer>,
        columns: &[SeriesDomain],
    ) -> Fallible<Self> {
        let mut has_implicit_mean = false;
        let mut has_var = false;
        let mut has_implicit_var = false;
        for expr in &aggs {
            if let Some(variance) = variance_args(expr, columns)? {
                has_var = true;
                has_implicit_var |= variance.scale.is_none();
            }
        }
        for expr in &aggs {
            has_implicit_mean |= implicit_mean(expr)?.is_some();
        }
        let needs_shared_count = has_implicit_mean || has_implicit_var;
        if !has_var && !has_implicit_mean
            && !aggs.iter().any(can_decompose)
            && !has_duplicates(&aggs)?
        {
            return Ok(Self::unchanged(aggs));
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

        // One implicit-scale count serves top-level means and variances,
        // including requests on different columns.
        let mut count = None;
        if needs_shared_count {
            for (expr, name) in aggs.iter().zip(&outputs) {
                if implicit_count(expr)? {
                    count = Some(name.clone());
                    break;
                }
            }
        }
        let synthesized = needs_shared_count && count.is_none();
        if synthesized {
            count = Some(unique_name(&mut used, "count"));
        }

        let mut private_aggs = Vec::with_capacity(aggs.len() + usize::from(synthesized) + 3 * usize::from(has_var));
        let mut postprocess = Vec::new();
        let mut hidden = Vec::new();
        let mut count_aliases = HashMap::new();
        let mut have_count = false;

        for (expr, name) in aggs.into_iter().zip(&outputs) {
            if needs_shared_count && implicit_count(&expr)? {
                let shared_count = count.as_ref().expect("a shared count is required");
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
            } else if let Some(var) = variance_args(&expr, columns)? {
                let VarArgs {
                    input,
                    lower,
                    upper,
                    midpoint,
                    square_upper,
                    scale,
                } = var;

                // Matching means and sums already use the original clipped
                // input and can share this same first-moment mechanism.
                let sum_name = unique_name(&mut used, "sum");
                private_aggs.push(
                    apply_plugin(
                        vec![input.clone(), lit(lower), lit(upper), scale.map(lit).unwrap_or_default()],
                        expr.clone().meta().undo_aliases(),
                        DPSumShim,
                    ).alias(sum_name.as_str()),
                );
                hidden.push(sum_name.clone());

                // Fill nulls and NaNs with exactly the same midpoint as
                // dp.sum(), *before* squaring. Otherwise the two moments
                // describe different input populations.
                let clipped = input
                    .fill_null(lit(midpoint))
                    .fill_nan(lit(midpoint))
                    .clip(lit(lower), lit(upper));
                let centered = clipped - lit(midpoint);
                let squared = centered.clone() * centered;
                let sq_name = unique_name(&mut used, "squared");
                private_aggs.push(
                    apply_plugin(
                        vec![squared, lit(0.0f64), lit(square_upper), scale.map(lit).unwrap_or_default()],
                        expr.clone().meta().undo_aliases(),
                        DPSumShim,
                    ).alias(sq_name.as_str()),
                );
                hidden.push(sq_name.clone());

                let count_name = match scale {
                    None => count.as_ref().expect("implicit variance needs shared count").clone(),
                    Some(scale) => {
                        // Explicit-scale variance components are not merged
                        // with implicitly scaled counts or other estimates.
                        let name = unique_name(&mut used, "count");
                        private_aggs.push(dp_len(Some(scale), false).alias(name.as_str()));
                        hidden.push(name.clone());
                        name
                    }
                };
                let denom = col(count_name.as_str());
                let mean = col(sum_name.as_str()) / denom.clone();
                let centered_mean = mean - lit(midpoint);
                let variance =
                    col(sq_name.as_str()) / denom - centered_mean.clone() * centered_mean;
                postprocess.push(
                    variance.clip(lit(0.0f64), lit(square_upper)).alias(name.as_str()),
                );
            } else if can_decompose(&expr) {
                // Defer deterministic postprocessing; equivalent private
                // leaves are deduplicated in the final pass below.
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

    /// Resolve a user-visible count alias to its materialized noisy column.
    pub fn raw_count(&self, output_name: &str) -> String {
        self.count_aliases
            .get(output_name)
            .cloned()
            .unwrap_or_else(|| output_name.to_string())
    }

    /// Lower a filter over requested results to a deterministic expression
    /// over the already materialized noisy aggregate columns.
    fn lower_predicate(&self, expr: Expr) -> Fallible<Expr> {
        match expr {
            Expr::Column(name) => {
                let name = name.as_str();
                if let Some(reconstructed) = self.postprocess.iter().find(|expr| {
                    expr.clone().meta().output_name().is_ok_and(|output| output.as_str() == name)
                }) {
                    return self.lower_predicate(reconstructed.clone().meta().undo_aliases());
                }
                let raw = self.raw_count(name);
                Ok(col(raw.as_str()))
            }
            Expr::Literal(_) => Ok(expr),
            Expr::Alias(inner, name) => Ok(self.lower_predicate(*inner)?.alias(name)),
            Expr::BinaryExpr { left, op, right } => Ok(Expr::BinaryExpr {
                left: Arc::new(self.lower_predicate(*left)?),
                op,
                right: Arc::new(self.lower_predicate(*right)?),
            }),
            Expr::Cast { expr, dtype, options } => Ok(Expr::Cast {
                expr: Arc::new(self.lower_predicate(*expr)?),
                dtype,
                options,
            }),
            other => {
                // Preserve other deterministic Polars filters when none of
                // their input columns require reconstruction. Rewriting an
                // unknown expression containing a deferred result is unsafe.
                let deferred = self.postprocess.iter().filter_map(|expr| {
                    expr.clone().meta().output_name().ok().map(|n| n.to_string())
                }).chain(self.count_aliases.keys().cloned()).collect::<HashSet<_>>();
                if other.clone().meta().root_names().iter().any(|n| deferred.contains(n.as_str())) {
                    return crate::fallible!(
                        MakeMeasurement,
                        "unsupported filter postprocessing: {other:?}"
                    );
                }
                Ok(other)
            }
        }
    }

    /// Build one key-selection predicate and (when required) certify that it
    /// contains a positive threshold on a genuine noisy group length.
    pub fn selection(
        &self,
        filter: Option<Expr>,
        threshold: Option<u32>,
        dp_exprs: &[Expr],
        group_keys: &[Expr],
        private_keys: bool,
    ) -> Fallible<Option<Selection>> {
        let guard = if private_keys {
            let mut found = None;
            if let Some(predicate) = &filter {
                for (name, bound) in guarded_thresholds(predicate) {
                    let raw_name = self.raw_count(&name);
                    if let Some((_, noise)) = dp_exprs.iter().find_map(|expr| {
                        is_len_expr(expr, Some(&raw_name))
                    }) {
                        found = Some((raw_name, noise, bound));
                        break;
                    }
                }
            }
            if let Some(found) = found {
                Some(found)
            } else if let Some(bound) = threshold {
                let (name, noise) = find_len_expr(&dp_exprs.to_vec(), self.count.as_deref())?;
                Some((name, noise, bound))
            } else {
                return crate::fallible!(
                    MakeMeasurement,
                    "private group keys require a threshold on a noisy length or a public-key join"
                );
            }
        } else {
            None
        };

        let mut predicate = filter.map(|expr| self.lower_predicate(expr)).transpose()?;
        // If the user's predicate already contains a certified threshold,
        // retain it. Otherwise add the constructor threshold as a conjunction,
        // never discard the original filter.
        if let Some((name, _, bound)) = &guard {
            let user_guarded = filter_has_certified_guard(
                predicate.as_ref(), name, *bound
            );
            if !user_guarded {
                let gate = col(name.as_str()).gt(lit(*bound));
                predicate = Some(match predicate {
                    Some(predicate) => gate.and(predicate),
                    None => gate,
                });
            }
        }

        let Some(predicate) = predicate else {
            return Ok(None);
        };

        let mut used = self.outputs.iter().cloned().collect::<HashSet<_>>();
        used.extend(self.hidden.iter().cloned());
        for expr in dp_exprs {
            used.insert(expr.clone().meta().output_name()?.to_string());
        }
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
        let column = unique_name(&mut used, "keep");
        Ok(Some(Selection {
            column,
            predicate,
            certificate: guard.map(|(_, noise, bound)| (noise, bound)),
        }))
    }

    /// Postprocess only after key sanitization and any empty-group fills.
    /// Remove the temporary selection column and internal statistics.
    pub fn finish(&self, input: DslPlan, selection_column: Option<&str>) -> DslPlan {
        if self.postprocess.is_empty() && selection_column.is_none() {
            return input;
        }

        let input = if self.postprocess.is_empty() {
            input
        } else {
            DslPlan::HStack {
                input: Arc::new(input),
                exprs: self.postprocess.clone(),
                options: projection_options(),
            }
        };
        let excluded = self
            .outputs
            .iter()
            .chain(&self.hidden)
            .map(|s| PlSmallStr::from(s.as_str()))
            .chain(selection_column.map(PlSmallStr::from))
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

/// A deterministic predicate and its separately certified sparse-key guard.
/// Neither the predicate nor this intermediate column samples additional noise.
pub(super) struct Selection {
    pub column: String,
    predicate: Expr,
    pub certificate: Option<(NoisePlugin, u32)>,
}

impl Selection {
    pub fn apply(&self, input: DslPlan) -> DslPlan {
        let input = DslPlan::HStack {
            input: Arc::new(input),
            exprs: vec![self.predicate.clone().alias(self.column.as_str())],
            options: projection_options(),
        };
        DslPlan::Filter {
            input: Arc::new(input),
            predicate: col(self.column.as_str()),
        }
    }
}

/// Only a positive conjunctive threshold can certify sparse-key selection.
/// A threshold beneath OR, NOT or any other operation is insufficient.
fn guarded_thresholds(predicate: &Expr) -> Vec<(String, u32)> {
    match predicate {
        Expr::BinaryExpr { left, op: Operator::And | Operator::LogicalAnd, right } => {
            let mut guards = guarded_thresholds(left);
            guards.extend(guarded_thresholds(right));
            guards
        }
        predicate => is_threshold_predicate(predicate.clone()).into_iter().collect(),
    }
}

fn filter_has_certified_guard(
    predicate: Option<&Expr>,
    name: &str,
    bound: u32,
) -> bool {
    predicate.is_some_and(|predicate| {
        guarded_thresholds(predicate).iter().any(|(n, t)| n == name && *t == bound)
    })
}

/// A population-variance workload. Direct Float64 inputs and finite public
/// bounds keep the second moment bounded and avoid overflow in its sensitivity
/// analysis. The existing dp.sum constructor certifies both noisy moments.
struct VarArgs {
    input: Expr,
    lower: f64,
    upper: f64,
    midpoint: f64,
    square_upper: f64,
    scale: Option<f64>,
}

fn variance_args(expr: &Expr, columns: &[SeriesDomain]) -> Fallible<Option<VarArgs>> {
    let expr = expr.clone().meta().undo_aliases();
    let Some([input, lower, upper, scale]) = match_shim::<DPVarShim, 4>(&expr)? else {
        return Ok(None);
    };

    let Expr::Column(ref name) = input else {
        return crate::fallible!(
            MakeMeasurement,
            "dp.var() requires a direct Float64 column"
        );
    };
    let domain = columns.iter().find(|col| col.name.as_str() == name.as_str()).ok_or_else(|| {
        crate::err!(MakeMeasurement, "missing domain for dp.var() column {name}")
    })?;
    if domain.dtype() != polars::prelude::DataType::Float64 {
        return crate::fallible!(
            MakeMeasurement,
            "dp.var() currently requires Float64 input, found {}",
            domain.dtype()
        );
    }

    let lower = literal_value_of::<f64>(&lower)?.ok_or_else(|| {
        crate::err!(MakeMeasurement, "dp.var() lower bound must be a finite Float64 literal")
    })?;
    let upper = literal_value_of::<f64>(&upper)?.ok_or_else(|| {
        crate::err!(MakeMeasurement, "dp.var() upper bound must be a finite Float64 literal")
    })?;
    if !lower.is_finite() || !upper.is_finite() || lower > upper {
        return crate::fallible!(
            MakeMeasurement,
            "dp.var() bounds must be finite and ordered"
        );
    }

    // Match the midpoint filler in make_expr_dp_sum exactly. Reject bounds
    // whose midpoint or centered square could overflow.
    let midpoint = (lower + upper) / 2.0;
    let radius = (lower - midpoint).abs().max((upper - midpoint).abs());
    let square_upper = (radius * radius).next_up();
    if !midpoint.is_finite() || !radius.is_finite() || !square_upper.is_finite() {
        return crate::fallible!(
            MakeMeasurement,
            "dp.var() bounds are too large for finite centered squares"
        );
    }
    let scale = literal_value_of::<f64>(&scale)?;
    Ok(Some(VarArgs { input, lower, upper, midpoint, square_upper, scale }))
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

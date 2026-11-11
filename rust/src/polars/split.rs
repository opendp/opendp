//! Partition a resolved private plan into deterministic heads and an IPC-reading
//! continuation. This is not a privacy validator or a SQL compatibility checker.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};

use polars::{
    error::{PolarsResult, polars_err},
    prelude::{
        AnyValue, DataType, DslPlan, Expr, JoinArgs, JoinCoalesce, JoinType, LiteralValue, NamedFrom,
        PlSmallStr, Scalar, Series, col, len, when,
    },
};
use polars_plan::{
    plans::visitor::{RewriteRecursion, RewritingVisitor, TreeWalker},
    prelude::{DslBuilder, JoinOptions, ScanSources},
};
use polars_utils::plpath::PlPath;

use super::{apply_plugin, match_trusted_plugin};
use crate::{
    error::Fallible,
    measurements::{expr_noise::NoisePlugin, expr_noisy_max::NoisyMaxPlugin},
    transformations::expr_discrete_quantile_score::{
        DiscreteQuantileScorePlugin, HistogramQuantileScorePlugin,
    },
};

/// Return ordinary executable deterministic heads followed by the randomized
/// continuation, without I/O or schema resolution. Head i must be materialized
/// at work_dir/i.arrow. Inputs preceding the private Select/GroupBy are retained
/// verbatim. Additional sources outside that opaque private boundary are unsupported.
///
/// The trusted executor must enforce protected intermediates, a consistent
/// one-shot execution, final shuffling and cleanup. Cross-process
/// serialization retains the input plan's requirements for FFI-backed plugins.
#[cfg_attr(not(feature = "ffi"), allow(dead_code))]
pub(crate) fn split_plan(plan: DslPlan, work_dir: &Path) -> Fallible<Vec<DslPlan>> {
    let mut splitter = Splitter {
        work_dir,
        heads: vec![],
        found_private: false,
        names: Names::default(),
    };
    // Reserve names in the continuation and private expressions. Do not inspect
    // the already-analyzed private base.
    splitter.names.reserve_plan(&plan)?;
    let continuation = splitter.split(plan)?;
    if !splitter.found_private {
        return fallible!(
            FailedFunction,
            "no resolved private aggregation or selection"
        );
    }
    splitter.heads.push(continuation);
    Ok(splitter.heads)
}

fn ipc_scan(path: &Path) -> Fallible<DslPlan> {
    Ok(DslBuilder::scan_ipc(
        ScanSources::Paths(vec![PlPath::Local(path.into())].into()),
        Default::default(),
        Default::default(),
    )?
    .0)
}

fn noise(expr: &Expr) -> PolarsResult<bool> {
    Ok(match_trusted_plugin::<NoisePlugin>(expr)
        .map_err(|e| polars_err!(InvalidOperation: "{e}"))?
        .is_some()
        || match_trusted_plugin::<NoisyMaxPlugin>(expr)
            .map_err(|e| polars_err!(InvalidOperation: "{e}"))?
            .is_some())
}

fn has_noise(expr: &Expr) -> PolarsResult<bool> {
    for node in expr {
        if noise(node)? {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Default)]
struct Names {
    used: HashSet<PlSmallStr>,
    next: usize,
}

impl Names {
    fn fresh(&mut self) -> PlSmallStr {
        loop {
            let name: PlSmallStr = format!("__opendp_split_{}", self.next).into();
            self.next += 1;
            if self.used.insert(name.clone()) {
                return name;
            }
        }
    }

    fn reserve_expr(&mut self, expr: &Expr) {
        if let Ok(name) = expr.clone().meta().output_name() {
            self.used.insert(name);
        }
        for node in expr {
            if let Expr::Column(name) | Expr::Alias(_, name) = node {
                self.used.insert(name.clone());
            }
        }
    }

    fn reserve_plan(&mut self, plan: &DslPlan) -> Fallible<()> {
        match plan {
            DslPlan::Select { input, expr, .. }
            | DslPlan::GroupBy {
                input, aggs: expr, ..
            } => {
                for e in expr {
                    self.reserve_expr(e);
                }
                if let DslPlan::GroupBy { keys, .. } = plan {
                    for e in keys {
                        self.reserve_expr(e);
                    }
                }
                for e in expr {
                    if has_noise(e)? {
                        return Ok(());
                    } // Opaque base starts here.
                }
                self.reserve_plan(input)?;
            }
            DslPlan::HStack { input, exprs, .. } => {
                for e in exprs {
                    self.reserve_expr(e);
                }
                self.reserve_plan(input)?;
            }
            DslPlan::Filter { input, predicate } => {
                self.reserve_expr(predicate);
                self.reserve_plan(input)?;
            }
            DslPlan::Join {
                input_left,
                input_right,
                left_on,
                right_on,
                predicates,
                ..
            } => {
                for e in left_on.iter().chain(right_on).chain(predicates) {
                    self.reserve_expr(e);
                }
                self.reserve_plan(input_left)?;
                self.reserve_plan(input_right)?;
            }
            DslPlan::Distinct { input, options } => {
                for e in options.subset.iter().flatten() {
                    self.reserve_expr(e);
                }
                self.reserve_plan(input)?;
            }
            DslPlan::IR { dsl, .. } => self.reserve_plan(dsl)?,
            DslPlan::DataFrameScan { .. } | DslPlan::Scan { .. } => {}
            _ => {
                return fallible!(
                    FailedFunction,
                    "unsupported plan surrounding private aggregation"
                );
            }
        }
        Ok(())
    }
}

struct Splitter<'a> {
    work_dir: &'a Path,
    heads: Vec<DslPlan>,
    found_private: bool,
    names: Names,
}

impl Splitter<'_> {
    fn externalize(&mut self, plan: DslPlan) -> Fallible<DslPlan> {
        let path = self.work_dir.join(format!("{}.arrow", self.heads.len()));
        let scan = ipc_scan(&path)?;
        self.heads.push(plan);
        Ok(scan)
    }

    fn split(&mut self, mut plan: DslPlan) -> Fallible<DslPlan> {
        // Cached IR must not retain the original private aggregation.
        if let DslPlan::IR { dsl, .. } = plan {
            return self.split((*dsl).clone());
        }
        let expressions = match &plan {
            DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => Some(expr),
            _ => None,
        };
        if let Some(exprs) = expressions {
            for expr in exprs {
                if has_noise(expr)? {
                    return self.cut(plan);
                }
            }
        }
        match &mut plan {
            DslPlan::Select { input, .. }
            | DslPlan::HStack { input, .. }
            | DslPlan::Filter { input, .. }
            | DslPlan::Distinct { input, .. } => {
                *input = Arc::new(self.split((**input).clone())?);
            }
            DslPlan::Join {
                input_left,
                input_right,
                ..
            } => {
                *input_left = Arc::new(self.split((**input_left).clone())?);
                *input_right = Arc::new(self.split((**input_right).clone())?);
            }
            DslPlan::DataFrameScan { .. } | DslPlan::Scan { .. } => {
                return fallible!(
                    FailedFunction,
                    "unsupported additional source outside the private boundary"
                );
            }
            _ => {
                return fallible!(
                    FailedFunction,
                    "unsupported plan surrounding private aggregation"
                );
            }
        }
        Ok(plan)
    }

    fn cut(&mut self, mut plan: DslPlan) -> Fallible<DslPlan> {
        if self.found_private {
            return fallible!(
                FailedFunction,
                "multiple private boundaries are unsupported"
            );
        }
        self.found_private = true;
        let projection_options = match &plan {
            DslPlan::Select { options, .. } => options.clone(),
            _ => Default::default(),
        };
        let (exprs, keys) = match &plan {
            DslPlan::Select { expr, .. } => (expr.clone(), vec![]),
            DslPlan::GroupBy {
                aggs,
                keys,
                apply,
                options,
                #[cfg(patch_polars)]
                predicates,
                ..
            } => {
                if apply.is_some() || options.as_ref() != &Default::default() {
                    return fallible!(
                        FailedFunction,
                        "unsupported private group-by options or callback"
                    );
                }
                #[cfg(patch_polars)]
                if !predicates.is_empty() {
                    return fallible!(FailedFunction, "group-by predicates are unsupported");
                }
                (aggs.clone(), keys.clone())
            }
            _ => unreachable!(),
        };
        let key_columns = keys
            .iter()
            .map(|key| Ok(col(key.clone().meta().output_name()?)))
            .collect::<PolarsResult<Vec<_>>>()?;
        let mut extract = Extract {
            splitter: self,
            keys,
            // Slot zero is the compatible scalar head. It is only materialized
            // if used; the other slots are native histogram continuations.
            locals: vec![plan.clone()],
            fills: vec![vec![]],
            aggs: vec![],
            owners: HashMap::new(),
        };
        let rewritten = exprs
            .iter()
            .map(|expr| expr.clone().rewrite(&mut extract, &mut ()))
            .collect::<PolarsResult<Vec<_>>>()?;
        let Extract {
            mut locals,
            fills,
            aggs,
            owners,
            ..
        } = extract;
        match &mut plan {
            DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => *expr = aggs,
            _ => unreachable!(),
        }
        let scalar_used = rewritten
            .iter()
            .map(|e| dependencies(e, &owners))
            .collect::<PolarsResult<Vec<_>>>()?
            .iter()
            .any(|deps| deps.contains(&0));
        if scalar_used {
            locals[0] = self.externalize(plan)?;
        }
        let mut outputs = vec![vec![]; locals.len()];
        let mut place = Place {
            owners: &owners,
            outputs: &mut outputs,
            names: &mut self.names,
        };
        let rewritten = rewritten
            .into_iter()
            .zip(exprs)
            .map(|(expr, original)| {
                Ok(expr
                    .rewrite(&mut place, &mut ())?
                    .alias(original.meta().output_name()?))
            })
            .collect::<PolarsResult<Vec<_>>>()?;
        let mut completed = locals
            .into_iter()
            .zip(outputs)
            .zip(fills)
            .filter(|((_, outputs), _)| !outputs.is_empty());
        let ((first, first_outputs), _) = completed
            .next()
            .expect("a randomized boundary has a head");
        let mut carried = first_outputs
            .iter()
            .map(|expr| Ok(col(expr.clone().meta().output_name()?)))
            .collect::<PolarsResult<Vec<_>>>()?;
        let mut combined = DslPlan::Select {
            input: Arc::new(first),
            expr: key_columns.iter().cloned().chain(first_outputs).collect(),
            options: Default::default(),
        };
        for ((right, outputs), fills) in completed {
            let mut args = JoinArgs::new(if key_columns.is_empty() {
                JoinType::Cross
            } else {
                JoinType::Left
            });
            args.nulls_equal = true;
            args.coalesce = JoinCoalesce::CoalesceColumns;
            // Join deterministic head results before randomizing the right head:
            // a missing group must receive its empty-group input, not a null release.
            let joined = DslPlan::Join {
                input_left: Arc::new(combined),
                input_right: Arc::new(right),
                left_on: key_columns.clone(),
                right_on: key_columns.clone(),
                predicates: vec![],
                options: Arc::new(JoinOptions {
                    args,
                    ..Default::default()
                }),
            };
            let input = if fills.is_empty() {
                joined
            } else {
                DslPlan::HStack {
                    input: Arc::new(joined),
                    exprs: fills,
                    options: Default::default(),
                }
            };
            let next = outputs
                .iter()
                .map(|expr| Ok(col(expr.clone().meta().output_name()?)))
                .collect::<PolarsResult<Vec<_>>>()?;
            combined = DslPlan::Select {
                input: Arc::new(input),
                expr: key_columns
                    .iter()
                    .cloned()
                    .chain(carried.iter().cloned())
                    .chain(outputs)
                    .collect(),
                options: Default::default(),
            };
            carried.extend(next);
        }
        Ok(DslPlan::Select {
            input: Arc::new(combined),
            expr: key_columns.into_iter().chain(rewritten).collect(),
            options: projection_options,
        })
    }
}

struct Extract<'a, 'b> {
    splitter: &'a mut Splitter<'b>,
    keys: Vec<Expr>,
    locals: Vec<DslPlan>,
    fills: Vec<Vec<Expr>>,
    aggs: Vec<Expr>,
    owners: HashMap<PlSmallStr, usize>,
}

fn score_plugin(expr: &Expr) -> PolarsResult<Option<(&Vec<Expr>, DiscreteQuantileScorePlugin)>> {
    match_trusted_plugin::<DiscreteQuantileScorePlugin>(expr)
        .map_err(|e| polars_err!(InvalidOperation: "{e}"))
}

impl RewritingVisitor for Extract<'_, '_> {
    type Node = Expr;
    type Arena = ();

    fn pre_visit(&mut self, expr: &Expr, _: &mut ()) -> PolarsResult<RewriteRecursion> {
        if score_plugin(expr)?.is_some() {
            return Ok(RewriteRecursion::MutateAndStop);
        }
        let contains_score = expr
            .into_iter()
            .map(score_plugin)
            .collect::<PolarsResult<Vec<_>>>()?
            .iter()
            .any(Option::is_some);
        Ok(if has_noise(expr)? || contains_score {
            RewriteRecursion::NoMutateAndContinue
        } else if scalar_literals(expr)? {
            RewriteRecursion::Stop
        } else {
            RewriteRecursion::MutateAndStop
        })
    }

    fn mutate(&mut self, expr: Expr, _: &mut ()) -> PolarsResult<Expr> {
        let name = self.splitter.names.fresh();
        if let Some((inputs, scorer)) = score_plugin(&expr)? {
            let bucket = self.splitter.names.fresh();
            let count = self.splitter.names.fresh();
            let [input] = inputs.as_slice() else {
                return Err(
                    polars_err!(InvalidOperation: "quantile extraction expects one scorer input"),
                );
            };
            if scalar_literals(input)? {
                return Err(
                    polars_err!(InvalidOperation: "quantile extraction does not support literal-only scorer inputs"),
                );
            }
            let value = self.splitter.names.fresh();
            let base = match &self.locals[0] {
                DslPlan::Select { input, .. } | DslPlan::GroupBy { input, .. } => input.clone(),
                _ => unreachable!(),
            };
            let values = DslPlan::HStack {
                input: base,
                exprs: vec![
                    input
                        .clone()
                        .cast(scorer.candidates.dtype().clone())
                        .alias(value.clone()),
                ],
                options: Default::default(),
            };
            let buckets = DslPlan::HStack {
                input: Arc::new(values),
                exprs: vec![
                    quantile_bucket(col(value), &scorer.candidates, 0, scorer.candidates.len())?
                        .alias(bucket.clone()),
                ],
                options: Default::default(),
            };
            let histogram = group_by(
                buckets,
                self.keys
                    .iter()
                    .cloned()
                    .chain([col(bucket.clone())])
                    .collect(),
                vec![len().cast(DataType::UInt64).alias(count.clone())],
            );
            let scan = self
                .splitter
                .externalize(histogram)
                .map_err(|e| polars_err!(InvalidOperation: "{e}"))?;
            let scores = apply_plugin(
                vec![col(bucket), col(count)],
                expr.clone(),
                HistogramQuantileScorePlugin {
                    num_candidates: scorer.candidates.len(),
                    alpha: scorer.alpha,
                    size_limit: scorer.size_limit,
                },
            )
            .alias(name.clone());
            let local = if self.keys.is_empty() {
                DslPlan::Select {
                    input: Arc::new(scan),
                    expr: vec![scores],
                    options: Default::default(),
                }
            } else {
                group_by(
                    scan,
                    self.keys
                        .iter()
                        .map(|key| Ok(col(key.clone().meta().output_name()?)))
                        .collect::<PolarsResult<_>>()?,
                    vec![scores],
                )
            };
            // Match the native scorer's typed empty-group fill before noise.
            let len = scorer.candidates.len();
            let zeros = Expr::Literal(LiteralValue::Scalar(Scalar::new(
                DataType::Array(Box::new(DataType::UInt64), len),
                AnyValue::Array(Series::new("".into(), &vec![0u64; len]), len),
            )));
            self.fills.push(vec![
                col(name.clone()).fill_null(zeros).alias(name.clone())
            ]);
            self.owners.insert(name.clone(), self.locals.len());
            self.locals.push(local);
        } else {
            self.aggs.push(expr.alias(name.clone()));
            self.owners.insert(name.clone(), 0);
        }
        Ok(col(name))
    }
}

fn bucket_literal(value: u64) -> Expr {
    Expr::Literal(LiteralValue::Scalar(Scalar::new(
        DataType::UInt64,
        value.into(),
    )))
}

/// Balanced exact typed comparisons: open intervals have even IDs and
/// candidate equalities have odd IDs. No conversion through floating point.
fn quantile_bucket(
    value: Expr,
    candidates: &Series,
    start: usize,
    end: usize,
) -> PolarsResult<Expr> {
    if start == end {
        return Ok(bucket_literal((2 * start) as u64));
    }
    let mid = start + (end - start) / 2;
    let pivot = Expr::Literal(LiteralValue::Scalar(Scalar::new(
        candidates.dtype().clone(),
        candidates.get(mid)?.into_static(),
    )));
    Ok(when(value.clone().lt(pivot.clone()))
        .then(quantile_bucket(value.clone(), candidates, start, mid)?)
        .otherwise(
            when(value.clone().eq(pivot))
                .then(bucket_literal((2 * mid + 1) as u64))
                .otherwise(quantile_bucket(value, candidates, mid + 1, end)?),
        ))
}

fn group_by(input: impl Into<Arc<DslPlan>>, keys: Vec<Expr>, aggs: Vec<Expr>) -> DslPlan {
    DslPlan::GroupBy {
        input: input.into(),
        keys,
        aggs,
        apply: None,
        maintain_order: false,
        options: Default::default(),
        #[cfg(patch_polars)]
        predicates: vec![],
    }
}

fn dependencies(expr: &Expr, owners: &HashMap<PlSmallStr, usize>) -> PolarsResult<HashSet<usize>> {
    let mut deps = HashSet::new();
    for node in expr {
        if let Expr::Column(name) = node {
            if let Some(owner) = owners.get(name) {
                deps.insert(*owner);
            }
        }
        // Literal noise still belongs to the scalar head, where Polars retains
        // its one-draw broadcasting semantics even when there are many keys.
        if noise(node)?
            && !node
                .into_iter()
                .any(|node| matches!(node, Expr::Column(name) if owners.contains_key(name)))
        {
            deps.insert(0);
        }
    }
    Ok(deps)
}

struct Place<'a> {
    owners: &'a HashMap<PlSmallStr, usize>,
    outputs: &'a mut [Vec<Expr>],
    names: &'a mut Names,
}
impl RewritingVisitor for Place<'_> {
    type Node = Expr;
    type Arena = ();
    fn pre_visit(&mut self, expr: &Expr, _: &mut ()) -> PolarsResult<RewriteRecursion> {
        Ok(if dependencies(expr, self.owners)?.len() == 1 {
            RewriteRecursion::MutateAndStop
        } else {
            RewriteRecursion::NoMutateAndContinue
        })
    }
    fn mutate(&mut self, expr: Expr, _: &mut ()) -> PolarsResult<Expr> {
        let owner = *dependencies(&expr, self.owners)?.iter().next().unwrap();
        let name = self.names.fresh();
        self.outputs[owner].push(expr.alias(name.clone()));
        Ok(col(name))
    }
}

/// Analysis establishes validity, not Polars' literal broadcasting state.
/// Turning a literal-only input into an IPC column would turn one shared noise
/// draw into a draw per group. Leave these trees intact, recognizing only their
/// scalar literal leaves, not their operators. Nullary context inputs such as
/// len() are not literals and must still be computed in the original context.
fn scalar_literals(expr: &Expr) -> PolarsResult<bool> {
    for node in expr {
        if node.clone().meta().pop(None)?.is_empty()
            && !matches!(node, Expr::Literal(value) if value.is_scalar())
        {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod test;

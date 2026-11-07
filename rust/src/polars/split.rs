//! Partition a resolved private plan into native IPC-writing and IPC-reading
//! stages. This is not a privacy validator or a SQL compatibility checker.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};

use polars::{
    error::{PolarsResult, polars_err},
    prelude::{
        DataType, DslPlan, ExplodeOptions, Expr, JoinArgs, JoinCoalesce, JoinType, LiteralValue,
        PlSmallStr, Scalar, col, cols, lit, when,
    },
};
use polars_plan::{
    plans::visitor::{RewriteRecursion, RewritingVisitor, TreeWalker},
    prelude::{
        DslBuilder, FileSinkOptions, FileType, JoinOptions, ScanSources, SinkTarget, SinkType,
    },
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

/// Return (IPC SinkMultiple, randomized continuation), without I/O or schema
/// resolution. Inputs preceding the private Select/GroupBy are retained verbatim.
///
/// The trusted executor must enforce protected intermediates, a consistent
/// snapshot, one-shot execution, final shuffling and cleanup. Cross-process
/// serialization retains the input plan's requirements for FFI-backed plugins.
#[allow(dead_code)] // Pending trusted OnceFrame integration.
pub(crate) fn split_plan(plan: DslPlan, work_dir: &Path) -> Fallible<(DslPlan, DslPlan)> {
    let mut splitter = Splitter {
        work_dir,
        heads: vec![],
        found_private: false,
        names: Names::default(),
    };
    // Reserve names before rewriting either join branch, including public
    // DataFrame schemas. Do not inspect the already-analyzed private base.
    splitter.names.reserve_plan(&plan)?;
    let continuation = splitter.split(plan)?;
    if !splitter.found_private {
        return fallible!(
            FailedFunction,
            "no resolved private aggregation or selection"
        );
    }
    Ok((
        DslPlan::SinkMultiple {
            inputs: splitter.heads,
        },
        continuation,
    ))
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
            DslPlan::DataFrameScan { schema, .. } => self.used.extend(schema.iter_names().cloned()),
            DslPlan::Scan {
                unified_scan_args, ..
            } => {
                if let Some(schema) = &unified_scan_args.schema {
                    self.used.extend(schema.iter_names().cloned());
                }
            }
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
        self.heads.push(DslPlan::Sink {
            input: Arc::new(plan),
            payload: SinkType::File(FileSinkOptions {
                target: SinkTarget::Path(PlPath::Local(path.into())),
                file_format: Arc::new(FileType::Ipc(Default::default())),
                unified_sink_args: Default::default(),
            }),
        });
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
            DslPlan::DataFrameScan { .. } => return self.externalize(plan),
            DslPlan::Scan { .. } => {}
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
        let (exprs, keys, base) = match &plan {
            DslPlan::Select { expr, input, .. } => (expr.clone(), vec![], input.clone()),
            DslPlan::GroupBy {
                aggs,
                keys,
                input,
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
                (aggs.clone(), keys.clone(), input.clone())
            }
            _ => unreachable!(),
        };
        let key_columns = keys
            .iter()
            .map(|key| Ok(col(key.clone().meta().output_name()?)))
            .collect::<PolarsResult<Vec<_>>>()?;
        let mut extract = Extract {
            splitter: self,
            base,
            keys,
            // Slot zero is the compatible scalar head. It is only materialized
            // if used; the other slots are native histogram continuations.
            locals: vec![plan.clone()],
            aggs: vec![],
            owners: HashMap::new(),
        };
        let rewritten = exprs
            .iter()
            .map(|expr| expr.clone().rewrite(&mut extract, &mut ()))
            .collect::<PolarsResult<Vec<_>>>()?;
        let Extract {
            mut locals,
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
            .filter_map(|(input, outputs)| {
                if outputs.is_empty() {
                    return None;
                }
                Some(DslPlan::Select {
                    input: Arc::new(input),
                    expr: key_columns.iter().cloned().chain(outputs).collect(),
                    options: Default::default(),
                })
            });
        // These one-row/group plans are black boxes to combination. Mechanism
        // internals and missing-group fills never enter the join logic.
        let mut combined = completed.next().expect("a randomized boundary has a head");
        for right in completed {
            let mut args = JoinArgs::new(if key_columns.is_empty() {
                JoinType::Cross
            } else {
                JoinType::Inner
            });
            args.nulls_equal = true;
            args.coalesce = JoinCoalesce::CoalesceColumns;
            combined = DslPlan::Join {
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
    base: Arc<DslPlan>,
    keys: Vec<Expr>,
    locals: Vec<DslPlan>,
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
            // Evaluate in the original aggregation context: scorer inputs can
            // filter/drop observations without dropping their observed group.
            // Implode also preserves literal and empty-global semantics without
            // classifying the input's operators or cardinality.
            let assignment = buckets(
                inputs[0].clone(),
                &scorer.candidates,
                0,
                scorer.candidates.len(),
            )?
            .cast(DataType::UInt64)
            .implode()
            .alias(bucket.clone());
            let collected = if self.keys.is_empty() {
                DslPlan::Select {
                    input: self.base.clone(),
                    expr: vec![assignment],
                    options: Default::default(),
                }
            } else {
                group_by(self.base.clone(), self.keys.clone(), vec![assignment])
            };
            let exploded = DslBuilder::from(collected)
                .explode(
                    cols([bucket.clone()]),
                    ExplodeOptions {
                        empty_as_null: true,
                        keep_nulls: true,
                    },
                    false,
                )
                .0;
            let histogram = group_by(
                exploded,
                self.keys
                    .iter()
                    .map(|key| Ok(col(key.clone().meta().output_name()?)))
                    .chain([Ok(col(bucket.clone())
                        .fill_null(lit(0u64).cast(DataType::UInt64))
                        .alias(bucket.clone()))])
                    .collect::<PolarsResult<_>>()?,
                // Empty-list sentinels keep the group with bucket zero/count
                // zero. Count the original non-null buckets, not sentinel rows.
                vec![
                    col(bucket.clone())
                        .count()
                        .cast(DataType::UInt64)
                        .alias(count.clone()),
                ],
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
            self.owners.insert(name.clone(), self.locals.len());
            self.locals.push(local);
        } else {
            self.aggs.push(expr.alias(name.clone()));
            self.owners.insert(name.clone(), 0);
        }
        Ok(col(name))
    }
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

/// Native CASE decisions have logarithmic depth, and distinguish equality from
/// each adjacent open interval. Resolved candidates already have the input dtype.
fn buckets(
    input: Expr,
    candidates: &polars::series::Series,
    start: usize,
    end: usize,
) -> PolarsResult<Expr> {
    if start == end {
        return Ok(lit(2 * start as u64));
    }
    let mid = start + (end - start) / 2;
    let candidate = Expr::Literal(LiteralValue::Scalar(Scalar::new(
        candidates.dtype().clone(),
        candidates.get(mid)?.into_static(),
    )));
    Ok(when(input.clone().lt(candidate.clone()))
        .then(buckets(input.clone(), candidates, start, mid)?)
        .otherwise(
            when(input.clone().eq(candidate))
                .then(lit(2 * mid as u64 + 1))
                .otherwise(buckets(input, candidates, mid + 1, end)?),
        ))
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

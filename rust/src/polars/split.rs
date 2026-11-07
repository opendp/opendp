//! Partition a resolved private plan into native IPC-writing and IPC-reading
//! stages. This is not a privacy validator or a SQL compatibility checker.

use std::{collections::HashSet, path::Path, sync::Arc};

use polars::{
    error::{PolarsResult, polars_err},
    prelude::{DslPlan, Expr, PlSmallStr, col},
};
use polars_plan::{
    plans::visitor::{RewriteRecursion, RewritingVisitor, TreeWalker},
    prelude::{DslBuilder, FileSinkOptions, FileType, ScanSources, SinkTarget, SinkType},
};
use polars_utils::plpath::PlPath;

use super::match_trusted_plugin;
use crate::{error::Fallible, measurements::expr_noise::NoisePlugin};

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
    match_trusted_plugin::<NoisePlugin>(expr)
        .map(|matched| matched.is_some())
        .map_err(|e| polars_err!(InvalidOperation: "{e}"))
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
        let (exprs, keys) = match &mut plan {
            DslPlan::Select { expr, .. } => (expr, vec![]),
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
                (aggs, keys.clone())
            }
            _ => unreachable!(),
        };
        let key_columns = keys
            .into_iter()
            .map(|key| Ok(col(key.meta().output_name()?)))
            .collect::<PolarsResult<Vec<_>>>()?;
        let mut extract = Extract {
            aggs: vec![],
            names: &mut self.names,
        };
        let rewritten = exprs
            .iter()
            .map(|expr| {
                let name = expr.clone().meta().output_name()?;
                Ok(expr.clone().rewrite(&mut extract, &mut ())?.alias(name))
            })
            .collect::<PolarsResult<Vec<_>>>()?;
        *exprs = extract.aggs;
        Ok(DslPlan::Select {
            input: Arc::new(self.externalize(plan)?),
            expr: key_columns.into_iter().chain(rewritten).collect(),
            options: projection_options,
        })
    }
}

struct Extract<'a> {
    aggs: Vec<Expr>,
    names: &'a mut Names,
}

impl RewritingVisitor for Extract<'_> {
    type Node = Expr;
    type Arena = ();

    fn pre_visit(&mut self, expr: &Expr, _: &mut ()) -> PolarsResult<RewriteRecursion> {
        Ok(if has_noise(expr)? {
            // Keep each trusted plugin and its surrounding postprocessing;
            // only its deterministic inputs (and public operands) are cut.
            RewriteRecursion::NoMutateAndContinue
        } else if scalar_literals(expr)? {
            RewriteRecursion::Stop
        } else {
            RewriteRecursion::MutateAndStop
        })
    }

    fn mutate(&mut self, expr: Expr, _: &mut ()) -> PolarsResult<Expr> {
        let name = self.names.fresh();
        self.aggs.push(expr.alias(name.clone()));
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

//! Split a **resolved** private plan into one external deterministic stage and a
//! Polars continuation. This is not a privacy validator or a SQL translator.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use polars::{
    error::{PolarsResult, polars_bail, polars_err},
    prelude::{DslPlan, Expr, col},
};
use polars_plan::{
    dsl::FunctionExpr,
    plans::visitor::{RewriteRecursion, RewritingVisitor, TreeWalker, VisitRecursion, Visitor},
    prelude::{AggExpr, DslBuilder, FileScanDsl, ScanSources},
};
use polars_utils::plpath::PlPath;

use super::match_trusted_plugin;
use crate::{error::Fallible, measurements::expr_noise::NoisePlugin};

/// Private intermediates must only be materialized in an executor-owned,
/// protected directory. The executor must enforce one-shot execution, a common
/// database snapshot, final shuffling, and cleanup; this type does none of those.
pub struct SplitPlan {
    pub base: DslPlan,
    /// Aggregate fragments have the shared base as their input (ordinary DSL,
    /// without placeholders). Additional fragments may be independent public
    /// inputs, such as an in-memory public key set. No fragment reads another.
    pub fragments: Vec<(DslPlan, PathBuf)>,
    pub continuation: DslPlan,
}

/// Does not resolve schemas, read files, create files, or create `work_dir`.
/// Accepts one private scalar Select or GroupBy and the surrounding resolved
/// Select/Filter/HStack/public-key Join plans. Unsupported shapes fail closed.
/// The base is copied verbatim, without validating its operations or expressions.
/// As with the input plan, cross-process serialization requires FFI-backed
/// plugins; Rust anonymous UDFs need a matching Polars function registry.
pub fn split_plan(plan: DslPlan, work_dir: &Path) -> Fallible<SplitPlan> {
    let mut splitter = Splitter {
        work_dir,
        base: None,
        fragments: vec![],
    };
    let mut continuation = splitter.split(plan)?;
    let base = splitter.base.take().ok_or_else(|| {
        err!(
            FailedFunction,
            "no resolved private aggregation or selection"
        )
    })?;
    // In particular, public key DataFrames must not be embedded in the plan
    // sent to Python. Keep Distinct/Join/imputation in their original positions.
    splitter.externalize_frames(&mut continuation)?;
    Ok(SplitPlan {
        base,
        fragments: splitter.fragments,
        continuation,
    })
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

/// Check expressions at the split boundary and in the continuation, including
/// aggregate interiors. The shared base is opaque and is not checked here.
fn check_expr(expr: &Expr, allow_noise: bool) -> PolarsResult<()> {
    for node in expr {
        match node {
            Expr::AnonymousFunction { .. } => {
                if !allow_noise || !noise(node)? {
                    polars_bail!(InvalidOperation: "unsupported plugin in split plan");
                }
            }
            #[cfg(feature = "ffi")]
            Expr::Function {
                function: FunctionExpr::FfiPlugin { .. },
                ..
            } => {
                if !allow_noise || !noise(node)? {
                    polars_bail!(InvalidOperation: "unsupported plugin in split plan");
                }
            }
            Expr::Cast { dtype, .. } if dtype.as_literal().is_none() => {
                polars_bail!(InvalidOperation: "dynamic cast types are unsupported");
            }
            Expr::Agg(AggExpr::Quantile { .. }) => {
                polars_bail!(InvalidOperation: "quantile aggregates are unsupported");
            }
            Expr::Function {
                function:
                    FunctionExpr::FoldHorizontal { .. }
                    | FunctionExpr::ReduceHorizontal { .. }
                    | FunctionExpr::CumFoldHorizontal { .. }
                    | FunctionExpr::CumReduceHorizontal { .. },
                ..
            } => {
                polars_bail!(InvalidOperation: "callback functions are unsupported");
            }
            Expr::Function {
                function: FunctionExpr::Random { .. },
                ..
            }
            | Expr::Function {
                function:
                    FunctionExpr::Rank {
                        options:
                            polars::prelude::RankOptions {
                                method: polars::prelude::RankMethod::Random,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                polars_bail!(InvalidOperation: "random builtin in split plan");
            }
            Expr::Alias(..)
            | Expr::Column(_)
            | Expr::Literal(_)
            | Expr::Cast { .. }
            | Expr::BinaryExpr { .. }
            | Expr::Ternary { .. }
            | Expr::Agg(_)
            | Expr::Len
            | Expr::Function { .. }
            | Expr::Filter { .. }
            | Expr::Sort { .. }
            | Expr::SortBy { .. }
            | Expr::Slice { .. }
            | Expr::Over { .. } => {}
            _ => polars_bail!(InvalidOperation: "unsupported expression in split plan"),
        }
    }
    Ok(())
}

struct Splitter<'a> {
    work_dir: &'a Path,
    base: Option<DslPlan>,
    fragments: Vec<(DslPlan, PathBuf)>,
}

impl Splitter<'_> {
    fn fragment(&mut self, plan: DslPlan) -> Fallible<DslPlan> {
        let path = self
            .work_dir
            .join(format!("{}.arrow", self.fragments.len()));
        let scan = ipc_scan(&path)?;
        self.fragments.push((plan, path));
        Ok(scan)
    }

    fn split(&mut self, mut plan: DslPlan) -> Fallible<DslPlan> {
        // Work on DSL, not a cached optimized IR that could retain the old data.
        if let DslPlan::IR { dsl, .. } = plan {
            return self.split((*dsl).clone());
        }
        let expressions = match &plan {
            DslPlan::Select { expr, .. } => Some(expr),
            DslPlan::GroupBy { aggs, .. } => Some(aggs),
            _ => None,
        };
        if let Some(exprs) = expressions {
            let private = exprs
                .iter()
                .map(has_noise)
                .collect::<PolarsResult<Vec<_>>>()?
                .into_iter()
                .any(|b| b);
            if private {
                return self.cut(plan);
            }
        }
        match &mut plan {
            DslPlan::Select { input, expr, .. }
            | DslPlan::HStack {
                input, exprs: expr, ..
            } => {
                for e in expr {
                    check_expr(e, true)?;
                }
                *input = Arc::new(self.split((**input).clone())?);
            }
            DslPlan::Filter { input, predicate } => {
                check_expr(predicate, false)?;
                *input = Arc::new(self.split((**input).clone())?);
            }
            DslPlan::Join {
                input_left,
                input_right,
                left_on,
                right_on,
                predicates,
                ..
            } => {
                for e in left_on
                    .iter()
                    .chain(right_on.iter())
                    .chain(predicates.iter())
                {
                    check_expr(e, false)?;
                }
                *input_left = Arc::new(self.split((**input_left).clone())?);
                *input_right = Arc::new(self.split((**input_right).clone())?);
            }
            DslPlan::Distinct { input, options } => {
                for expr in options.subset.iter().flatten() {
                    check_expr(expr, false)?;
                }
                *input = Arc::new(self.split((**input).clone())?);
            }
            DslPlan::Scan { scan_type, .. } => check_scan(scan_type)?,
            DslPlan::DataFrameScan { .. } => {}
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
        if self.base.is_some() {
            return fallible!(
                FailedFunction,
                "multiple private boundaries are unsupported"
            );
        }
        let projection_options = match &plan {
            DslPlan::Select { options, .. } => options.clone(),
            _ => Default::default(),
        };
        let (input, exprs, keys) = match &mut plan {
            DslPlan::Select { input, expr, .. } => (input, expr, vec![]),
            DslPlan::GroupBy {
                input,
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
                (input, aggs, keys.clone())
            }
            _ => unreachable!(),
        };
        let key_columns = keys
            .iter()
            .map(|key| {
                check_expr(key, false)?;
                Ok(col(key.clone().meta().output_name()?))
            })
            .collect::<PolarsResult<Vec<_>>>()?;
        self.base = Some((**input).clone());
        let mut reserved = HashSet::new();
        for expr in exprs.iter().chain(keys.iter()) {
            check_expr(expr, true)?;
            reserved.insert(expr.clone().meta().output_name()?);
            for node in expr {
                if let Expr::Column(name) | Expr::Alias(_, name) = node {
                    reserved.insert(name.clone());
                }
            }
        }
        let mut extract = Extract {
            aggs: vec![],
            reserved,
            next: 0,
        };
        let rewritten = exprs
            .iter()
            .map(|expr| {
                let name = expr.clone().meta().output_name()?;
                Ok(expr.clone().rewrite(&mut extract, &mut ())?.alias(name))
            })
            .collect::<PolarsResult<Vec<_>>>()?;
        if extract.aggs.is_empty() {
            return fallible!(
                FailedFunction,
                "private operation has no deterministic aggregate input"
            );
        }
        *exprs = extract.aggs;
        let scan = self.fragment(plan)?;
        Ok(DslPlan::Select {
            input: Arc::new(scan),
            expr: key_columns.into_iter().chain(rewritten).collect(),
            options: projection_options,
        })
    }

    fn externalize_frames(&mut self, plan: &mut DslPlan) -> Fallible<()> {
        match plan {
            DslPlan::DataFrameScan { .. } => {
                *plan = self.fragment(plan.clone())?;
            }
            DslPlan::Select { input, .. }
            | DslPlan::Filter { input, .. }
            | DslPlan::HStack { input, .. }
            | DslPlan::Distinct { input, .. } => {
                self.externalize_frames(Arc::make_mut(input))?;
            }
            DslPlan::Join {
                input_left,
                input_right,
                ..
            } => {
                self.externalize_frames(Arc::make_mut(input_left))?;
                self.externalize_frames(Arc::make_mut(input_right))?;
            }
            DslPlan::Scan { .. } => {}
            _ => return fallible!(FailedFunction, "unsupported continuation node"),
        }
        Ok(())
    }
}

fn check_scan(scan_type: &FileScanDsl) -> Fallible<()> {
    match scan_type {
        FileScanDsl::Ipc { .. }
        | FileScanDsl::Parquet { .. }
        | FileScanDsl::Csv { .. }
        | FileScanDsl::NDJson { .. } => Ok(()),
        _ => fallible!(FailedFunction, "unsupported scan or scan callback"),
    }
}

struct Extract {
    aggs: Vec<Expr>,
    reserved: HashSet<polars::prelude::PlSmallStr>,
    next: usize,
}

impl Extract {
    fn extract(&mut self, expr: Expr) -> PolarsResult<Expr> {
        let name = loop {
            let name: polars::prelude::PlSmallStr = format!("__opendp_split_{}", self.next).into();
            self.next += 1;
            if self.reserved.insert(name.clone()) {
                break name;
            }
        };
        self.aggs.push(expr.alias(name.clone()));
        Ok(col(name))
    }
}

impl RewritingVisitor for Extract {
    type Node = Expr;
    type Arena = ();

    fn pre_visit(&mut self, expr: &Expr, _: &mut ()) -> PolarsResult<RewriteRecursion> {
        if has_noise(expr)? {
            // Changing aggregation context is safe only for scalar arithmetic,
            // not for aggregation, filtering or reshaping of noisy values.
            if !noise(expr)?
                && !matches!(
                    expr,
                    Expr::Alias(..) | Expr::Cast { .. } | Expr::BinaryExpr { .. }
                )
            {
                polars_bail!(InvalidOperation: "unsupported operation on randomized values");
            }
            return Ok(RewriteRecursion::NoMutateAndContinue);
        }
        expr.visit(&mut Scalar, &())?;
        // Literal-only expressions must stay scalar. Polars samples noise on a
        // literal once, even in GroupBy, and broadcasts that same draw. Turning
        // it into a fragment column would silently resample noise per group.
        Ok(if expr.into_iter().any(is_aggregate) {
            RewriteRecursion::MutateAndStop
        } else {
            RewriteRecursion::Stop
        })
    }

    fn mutate(&mut self, expr: Expr, _: &mut ()) -> PolarsResult<Expr> {
        // Preserve the plugin nodes verbatim; only deterministic subtrees reach
        // this method. Randomized occurrences are never deduplicated.
        self.extract(expr)
    }
}

/// A deliberately narrow scalar check using the Polars visitor. Scalar cuts
/// prevent broadcasting an aggregate before noise (which would resample noise).
fn is_aggregate(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Agg(_)
            | Expr::Len
            | Expr::Function {
                function: FunctionExpr::NullCount,
                ..
            }
    )
}

struct Scalar;
impl Visitor for Scalar {
    type Node = Expr;
    type Arena = ();
    fn pre_visit(&mut self, node: &Expr, _: &()) -> PolarsResult<VisitRecursion> {
        match node {
            Expr::Agg(
                AggExpr::Sum(_)
                | AggExpr::Count { .. }
                | AggExpr::NUnique(_)
                | AggExpr::Min { .. }
                | AggExpr::Max { .. }
                | AggExpr::Mean(_),
            )
            | Expr::Len
            | Expr::Function {
                function: FunctionExpr::NullCount,
                ..
            } => Ok(VisitRecursion::Skip),
            Expr::Literal(value) if value.is_scalar() => Ok(VisitRecursion::Skip),
            Expr::Alias(..)
            | Expr::Cast { .. }
            | Expr::BinaryExpr { .. }
            | Expr::Ternary { .. } => Ok(VisitRecursion::Continue),
            _ => {
                polars_bail!(InvalidOperation: "unsupported non-scalar or non-aggregate private input")
            }
        }
    }
}

#[cfg(test)]
mod test;

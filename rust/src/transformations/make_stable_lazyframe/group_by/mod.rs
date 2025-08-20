use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, FrameDomain, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{Bound, Bounds, FrameDistance, L0PInfDistance, L01InfDistance};
use crate::traits::{InfMul, option_min};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::prelude::GroupbyOptions;

use super::StableDslPlan;

#[cfg(test)]
mod test;

/// Transformation for stable group by and aggregate.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_group_by<M: UnboundedMetric>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<M>,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>> {
    let DslPlan::GroupBy {
        input,
        keys,
        aggs,
        apply,
        maintain_order,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected group by in logical plan");
    };

    if apply.is_some() {
        return fallible!(MakeTransformation, "apply is not currently supported");
    }

    if maintain_order {
        return fallible!(
            MakeTransformation,
            "maintain_order is wasted compute because row ordering is protected information"
        );
    }

    if options.as_ref() != &GroupbyOptions::default() {
        return fallible!(MakeTransformation, "options is not currently supported");
    }

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, FrameDistance<M>) = t_prior.output_space();

    // create a transformation for each expression
    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };

    // each expression must be stable row by row
    keys.iter().try_for_each(|key| {
        key.clone()
            .make_stable(expr_domain.clone(), L0PInfDistance(middle_metric.0.clone()))
            .map(|_: Transformation<_, _, _, L01InfDistance<M>>| ())
    })?;

    // check that aggregations are infallible. Aggregations are allowed to resize data
    aggs.iter()
        .try_for_each(|e| check_infallible(e, Resize::Allow))?;

    if middle_metric.0.identifier().is_some() {
        return fallible!(
            MakeTransformation,
            "stable groupby (sample and aggregate) is not supported on datasets with unbounded row contributions. If you want to execute a groupby truncation, include the identifier in the groupby keys."
        );
    }

    // use Polars to compute the output dtype
    let series_domains = middle_domain
        .simulate_schema(|lf| lf.group_by(&keys).agg(&aggs))?
        .iter_fields()
        .map(SeriesDomain::new_from_field)
        .collect::<Fallible<_>>()?;

    let h_keys = keys.iter().cloned().collect();
    let output_domain = FrameDomain::new_with_margins(
        series_domains,
        (middle_domain.margins.iter().cloned())
            .filter(|m| m.by.is_subset(&h_keys))
            .map(|mut m| {
                m.invariant = None;
                m
            })
            .collect(),
    )?;

    let t_group_agg = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        middle_metric.clone(),
        Function::new(move |plan: &DslPlan| DslPlan::GroupBy {
            input: Arc::new(plan.clone()),
            keys: keys.clone(),
            aggs: aggs.clone(),
            apply: None,
            maintain_order: false,
            options: options.clone(),
        }),
        StabilityMap::new_fallible(move |d_in: &Bounds| {
            let contributed_rows = d_in.get_bound(&HashSet::new()).per_group;
            let contributed_groups = d_in.get_bound(&h_keys).num_groups;

            let Some(influenced_groups) = option_min(contributed_rows, contributed_groups) else {
                return fallible!(
                    FailedMap,
                    "an upper bound on the number of contributed rows or groups is required"
                );
            };

            Ok(Bounds(vec![Bound {
                by: HashSet::new(),
                per_group: Some(influenced_groups.inf_mul(&2)?),
                num_groups: None,
            }]))
        }),
    )?;

    t_prior >> t_group_agg
}

#[derive(Clone, Copy)]
pub(crate) enum Resize {
    Allow,
    Ban,
}

/// # Proof Definition
/// Returns an error if the expression may raise data-dependent errors,
/// or if `resize` is Ban and the expression resizes the data.
///
/// A resize is an expression that changes the number of rows in the data.
/// Scalar-valued expressions are not considered a resize,
/// because they can be broadcasted.
pub(crate) fn check_infallible(expr: &Expr, resize: Resize) -> Fallible<()> {
    Ok(match expr {
        Expr::Alias(e, _) => check_infallible(e.as_ref(), resize)?,
        Expr::Column(_) => (),
        Expr::Selector(_) => (),
        Expr::Literal(_) => (),
        Expr::BinaryExpr { left, right, .. } => {
            check_infallible(&left, Resize::Ban)?;
            check_infallible(&right, Resize::Ban)?;
        }
        Expr::Cast { options, expr, .. } => {
            if matches!(options, CastOptions::Strict) {
                return fallible!(
                    MakeTransformation,
                    "Strict casting may cause data-dependent errors. Set strict to false."
                );
            }
            check_infallible(expr, resize)?;
        }
        Expr::Sort { expr, .. } => check_infallible(expr.as_ref(), resize)?,
        Expr::Gather { .. } => fallible!(
            MakeTransformation,
            "Gather may cause data-dependent errors due to OOB indexing."
        )?,
        Expr::SortBy { expr, by, .. } => {
            check_infallible(expr, Resize::Ban)?;
            by.iter()
                .try_for_each(|by| check_infallible(by, Resize::Ban))?;
        }
        Expr::Agg(agg_expr) => match agg_expr {
            AggExpr::Sum(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::Mean(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::Median(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::NUnique(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::First(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::Last(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::Implode(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::Count(e, _) => check_infallible(e, Resize::Allow)?,
            AggExpr::Quantile { expr: e, .. } => check_infallible(e, Resize::Allow)?,
            AggExpr::Max { input: e, .. } => check_infallible(e, Resize::Allow)?,
            AggExpr::Min { input: e, .. } => check_infallible(e, Resize::Allow)?,
            AggExpr::Std(e, _) => check_infallible(e, Resize::Allow)?,
            AggExpr::Var(e, _) => check_infallible(e, Resize::Allow)?,
            AggExpr::AggGroups(e) => check_infallible(e, Resize::Allow)?,
        },
        Expr::Ternary {
            predicate,
            truthy,
            falsy,
        } => {
            check_infallible(predicate, Resize::Ban)?;
            check_infallible(truthy, Resize::Ban)?;
            check_infallible(falsy, Resize::Ban)?;
        }
        Expr::Function { input, function } => check_infallible_function(function, input, resize)?,
        Expr::Explode { input: e, .. } => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "explode may cause data-dependent errors due to different lengths."
                );
            }
            check_infallible(e, resize)?;
        }
        Expr::Filter { input, by } => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "Filter may cause data-dependent errors due to different lengths."
                );
            }
            check_infallible(input.as_ref(), resize)?;
            check_infallible(by.as_ref(), resize)?;
        }
        Expr::Window { .. } => {
            return fallible!(
                MakeTransformation,
                "Window functions are not currently supported."
            );
        }
        Expr::Slice { .. } => {
            return fallible!(
                MakeTransformation,
                "Slice may cause data-dependent errors due to null offset."
            );
        }
        Expr::KeepName(e) => check_infallible(e.as_ref(), resize)?,
        Expr::Len => (),
        Expr::RenameAlias { expr, .. } => check_infallible(expr.as_ref(), resize)?,
        Expr::Field(_) => (),
        Expr::AnonymousFunction { .. } => {
            return fallible!(
                MakeTransformation,
                "Anonymous functions could raise data-dependent errors."
            );
        }
        Expr::SubPlan(_, _) => {
            return fallible!(MakeTransformation, "SubPlans are not currently supported.");
        }
        Expr::DataTypeFunction(_) => {
            return fallible!(
                MakeTransformation,
                "Data type function is not currently supported."
            );
        }
        Expr::Eval { .. } => {
            return fallible!(MakeTransformation, "Eval is not currently supported.");
        }
    })
}

/// # Proof Definition
/// Returns an error if the function expression may raise data-dependent errors,
/// or if `resize` is Ban and the expression resizes the data.
///
/// A resize is an expression that changes the number of rows in the data.
/// Scalar-valued expressions are not considered a resize,
/// because they can be broadcasted.
fn check_infallible_function(
    function: &FunctionExpr,
    inputs: &Vec<Expr>,
    resize: Resize,
) -> Fallible<()> {
    macro_rules! check_inputs {
        () => {
            check_inputs!(resize)
        };
        (resize=$name:literal) => {{
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "{} may cause data-dependent errors due to different lengths.",
                    $name
                );
            }
            check_inputs!(resize)
        }};
        (aggregate) => {
            check_inputs!(Resize::Allow)
        };
        (aligned_rows) => {
            check_inputs!(Resize::Ban)
        };
        ($resize:expr) => {
            inputs
                .iter()
                .try_for_each(|e| check_infallible(e, $resize))?
        };
    }
    Ok(match function {
        FunctionExpr::Boolean(bool_expr) => match bool_expr {
            BooleanFunction::Any { .. } => check_inputs!(aggregate),
            BooleanFunction::All { .. } => check_inputs!(aggregate),
            BooleanFunction::IsNull => check_inputs!(),
            BooleanFunction::IsNotNull => check_inputs!(),
            BooleanFunction::IsFinite => check_inputs!(),
            BooleanFunction::IsInfinite => check_inputs!(),
            BooleanFunction::IsFirstDistinct => check_inputs!(),
            BooleanFunction::IsLastDistinct => check_inputs!(),
            BooleanFunction::IsUnique => check_inputs!(),
            BooleanFunction::IsDuplicated => check_inputs!(),
            BooleanFunction::IsBetween { .. } => check_inputs!(),
            BooleanFunction::IsNan => check_inputs!(),
            BooleanFunction::IsNotNan => check_inputs!(),
            BooleanFunction::IsIn { .. } => {
                let [input, set] = <&[Expr; 2]>::try_from(inputs.as_slice())
                    .map_err(|_| err!(MakeTransformation, "IsIn must have two arguments"))?;
                check_infallible(input, resize)?;
                check_infallible(set, Resize::Allow)?;
            }
            BooleanFunction::AllHorizontal => check_inputs!(aligned_rows),
            BooleanFunction::AnyHorizontal => check_inputs!(aligned_rows),
            BooleanFunction::Not => check_inputs!(),
        },
        FunctionExpr::Abs => check_inputs!(),
        FunctionExpr::Negate => check_inputs!(),
        FunctionExpr::NullCount => check_inputs!(aggregate),
        FunctionExpr::Pow(_) => check_inputs!(aligned_rows),
        FunctionExpr::Range(_) => check_inputs!(),
        FunctionExpr::FillNull => check_inputs!(),
        FunctionExpr::FillNullWithStrategy(_) => check_inputs!(),
        FunctionExpr::ShiftAndFill => check_inputs!(),
        FunctionExpr::Shift => check_inputs!(),
        FunctionExpr::DropNans => check_inputs!(),
        FunctionExpr::DropNulls => check_inputs!(),
        FunctionExpr::Reshape(_) => {
            return fallible!(
                MakeTransformation,
                "reshape expression may cause data-dependent errors due to different lengths."
            );
        }
        FunctionExpr::RepeatBy => check_inputs!(aligned_rows),
        FunctionExpr::ArgUnique => check_inputs!(),
        FunctionExpr::Rank { .. } => check_inputs!(),
        FunctionExpr::Repeat => check_inputs!(),
        FunctionExpr::Clip { .. } => check_inputs!(aligned_rows),
        FunctionExpr::AsStruct => check_inputs!(aligned_rows),
        FunctionExpr::Reverse => check_inputs!(),
        FunctionExpr::ValueCounts { .. } => check_inputs!(resize = "value_counts"),
        FunctionExpr::Coalesce => check_inputs!(aligned_rows),
        FunctionExpr::ShrinkType => {
            return fallible!(MakeTransformation, "shrink_type has data-dependent dtype.");
        }
        FunctionExpr::Unique(_) => check_inputs!(resize = "unique"),
        FunctionExpr::Round { .. } => check_inputs!(),
        FunctionExpr::RoundSF { .. } => check_inputs!(),
        FunctionExpr::Floor => check_inputs!(),
        FunctionExpr::Ceil => check_inputs!(),
        FunctionExpr::UpperBound => check_inputs!(),
        FunctionExpr::LowerBound => check_inputs!(),
        FunctionExpr::ConcatExpr(_) => check_inputs!(resize = "concat_expr"),
        FunctionExpr::Cut { .. } => check_inputs!(),
        FunctionExpr::QCut { .. } => check_inputs!(),
        FunctionExpr::ToPhysical => check_inputs!(),
        FunctionExpr::Random { .. } => {
            // TODO: loosen this when Polars is updated past 0.44
            return fallible!(
                MakeTransformation,
                "Random may raise data-dependent errors due to sampling n without replacement from a set of length less than n."
            );
        }
        FunctionExpr::SetSortedFlag(_) => check_inputs!(),
        FunctionExpr::FfiPlugin { .. } => {
            return fallible!(
                MakeTransformation,
                "FfiPlugin may raise data-dependent errors."
            );
        }
        FunctionExpr::MaxHorizontal => check_inputs!(aligned_rows),
        FunctionExpr::MinHorizontal => check_inputs!(aligned_rows),
        FunctionExpr::SumHorizontal { ignore_nulls: _ } => check_inputs!(aligned_rows),
        FunctionExpr::MeanHorizontal { ignore_nulls: _ } => check_inputs!(aligned_rows),
        // in the future, other patterns may be added
        #[allow(unreachable_patterns)]
        _ => {
            return fallible!(
                MakeTransformation,
                "Function {function:?} is not currently supported."
            );
        }
    })
}

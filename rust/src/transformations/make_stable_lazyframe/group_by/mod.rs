use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{
    Context, DslPlanDomain, FrameDomain, SeriesDomain, WildExprDomain, option_min,
};
use crate::error::*;
use crate::metrics::{Bound, Bounds, FrameDistance, PartitionDistance};
use crate::traits::InfMul;
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::prelude::{FunctionFlags, GroupbyOptions};

use super::StableDslPlan;

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
) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, FrameDistance<M>, FrameDistance<M>>> {
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
            .make_stable(
                expr_domain.clone(),
                PartitionDistance(middle_metric.0.clone()),
            )
            .map(|_: Transformation<_, _, _, PartitionDistance<M>>| ())
    })?;

    // check that aggregations are infallible
    aggs.iter().try_for_each(|e| assert_infallible(e, true))?;

    // use Polars to compute the output dtype
    let series_domains = middle_domain
        .simulate_schema(|lf| lf.group_by(&keys).agg(&aggs))?
        .iter_fields()
        .map(SeriesDomain::new_from_field)
        .collect::<Fallible<_>>()?;

    let h_keys = keys.iter().cloned().collect();
    let output_domain = FrameDomain::new_with_margins(
        series_domains,
        middle_domain
            .margins
            .iter()
            .cloned()
            .filter(|m| m.by.is_subset(&h_keys))
            .map(|mut m| {
                if !m.by.is_subset(&h_keys) {
                    m.invariant = None;
                }
                m
            })
            .collect(),
    )?;

    let t_group_agg = Transformation::new(
        middle_domain,
        output_domain,
        Function::new(move |plan: &DslPlan| DslPlan::GroupBy {
            input: Arc::new(plan.clone()),
            keys: keys.clone(),
            aggs: aggs.clone(),
            apply: None,
            maintain_order,
            options: options.clone(),
        }),
        middle_metric.clone(),
        middle_metric.clone(),
        StabilityMap::new_fallible(move |d_in: &Bounds| {
            Ok(Bounds(vec![Bound {
                by: HashSet::new(),
                per_group: option_min(
                    d_in.get_bound(&HashSet::new()).per_group,
                    d_in.get_bound(&h_keys).num_groups,
                )
                .map(|v| v.inf_mul(&2))
                .transpose()?,
                num_groups: None,
            }]))
        }),
    )?;

    t_prior >> t_group_agg
}

pub(crate) fn assert_infallible(expr: &Expr, allow_resize: bool) -> Fallible<()> {
    Ok(match expr {
        Expr::Alias(e, _) => assert_infallible(e.as_ref(), allow_resize)?,
        Expr::Column(_) => (),
        Expr::Columns(_) => (),
        Expr::DtypeColumn(_) => (),
        Expr::IndexColumn(_) => (),
        Expr::Literal(_) => (),
        Expr::BinaryExpr { left, right, .. } => {
            assert_infallible(&left, false)?;
            assert_infallible(&right, false)?;
        }
        Expr::Cast { options, expr, .. } => {
            if matches!(options, CastOptions::Strict) {
                return fallible!(
                    MakeTransformation,
                    "Strict casting may cause data-dependent errors. Set strict to false."
                );
            }
            assert_infallible(expr, allow_resize)?;
        }
        Expr::Sort { expr, .. } => assert_infallible(expr.as_ref(), allow_resize)?,
        Expr::Gather { .. } => fallible!(
            MakeTransformation,
            "Gather may cause data-dependent errors due to OOB indexing."
        )?,
        Expr::SortBy { expr, by, .. } => {
            assert_infallible(expr, false)?;
            by.iter().try_for_each(|by| assert_infallible(by, false))?;
        }
        Expr::Agg(agg_expr) => match agg_expr {
            AggExpr::Sum(e) => assert_infallible(e, true)?,
            AggExpr::Mean(e) => assert_infallible(e, true)?,
            AggExpr::Median(e) => assert_infallible(e, true)?,
            AggExpr::NUnique(e) => assert_infallible(e, true)?,
            AggExpr::First(e) => assert_infallible(e, true)?,
            AggExpr::Last(e) => assert_infallible(e, true)?,
            AggExpr::Implode(e) => assert_infallible(e, true)?,
            AggExpr::Count(e, _) => assert_infallible(e, true)?,
            AggExpr::Quantile { expr: e, .. } => assert_infallible(e, true)?,
            AggExpr::Max { input: e, .. } => assert_infallible(e, true)?,
            AggExpr::Min { input: e, .. } => assert_infallible(e, true)?,
            AggExpr::Std(e, _) => assert_infallible(e, true)?,
            AggExpr::Var(e, _) => assert_infallible(e, true)?,
            AggExpr::AggGroups(e) => assert_infallible(e, true)?,
        },
        Expr::Ternary {
            predicate,
            truthy,
            falsy,
        } => {
            assert_infallible(predicate, false)?;
            assert_infallible(truthy, false)?;
            assert_infallible(falsy, false)?;
        }
        Expr::Function {
            input,
            function,
            options,
        } => {
            if !allow_resize && options.flags.contains(FunctionFlags::CHANGES_LENGTH) {
                return fallible!(
                    MakeTransformation,
                    "Function {function:?} may cause data-dependent errors due to different lengths."
                );
            }
            assert_infallible_function(function, input, allow_resize)?
        }
        Expr::Explode(e) => {
            if !allow_resize {
                return fallible!(
                    MakeTransformation,
                    "Explode may cause data-dependent errors due to different lengths."
                );
            }
            assert_infallible(e, allow_resize)?;
        }
        Expr::Filter { input, by } => {
            if !allow_resize {
                return fallible!(
                    MakeTransformation,
                    "Filter may cause data-dependent errors due to different lengths."
                );
            }
            assert_infallible(input.as_ref(), allow_resize)?;
            assert_infallible(by.as_ref(), allow_resize)?;
        }
        Expr::Window { .. } => {
            return fallible!(
                MakeTransformation,
                "Window functions are not currently supported."
            );
        }
        Expr::Wildcard => (),
        Expr::Slice { .. } => {
            return fallible!(
                MakeTransformation,
                "Slice may cause data-dependent errors due to null offset."
            );
        }
        Expr::Exclude(e, _) => assert_infallible(e.as_ref(), allow_resize)?,
        Expr::KeepName(e) => assert_infallible(e.as_ref(), allow_resize)?,
        Expr::Len => (),
        Expr::Nth(_) => (),
        Expr::RenameAlias { expr, .. } => assert_infallible(expr.as_ref(), allow_resize)?,
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
        Expr::Selector(_) => (),
    })
}

fn assert_infallible_function(
    function: &FunctionExpr,
    inputs: &Vec<Expr>,
    allow_resize: bool,
) -> Fallible<()> {
    macro_rules! check_inputs {
        () => {
            check_inputs!(allow_resize)
        };
        (aggregate) => {
            check_inputs!(true)
        };
        (aligned_rows) => {
            check_inputs!(false)
        };
        ($allow_resize:expr) => {
            inputs
                .iter()
                .try_for_each(|e| assert_infallible(e, $allow_resize))?
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
            BooleanFunction::IsNan => check_inputs!(),
            BooleanFunction::IsNotNan => check_inputs!(),
            BooleanFunction::IsIn => {
                let [input, set] = <&[Expr; 2]>::try_from(inputs.as_slice())
                    .map_err(|_| err!(MakeTransformation, "IsIn must have two arguments"))?;
                assert_infallible(input, allow_resize)?;
                assert_infallible(set, true)?;
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
        FunctionExpr::ValueCounts { .. } => check_inputs!(),
        FunctionExpr::Coalesce => check_inputs!(aligned_rows),
        FunctionExpr::ShrinkType => {
            return fallible!(MakeTransformation, "shrink_type has data-dependent dtype.");
        }
        FunctionExpr::Unique(_) => check_inputs!(),
        FunctionExpr::Round { .. } => check_inputs!(),
        FunctionExpr::RoundSF { .. } => check_inputs!(),
        FunctionExpr::Floor => check_inputs!(),
        FunctionExpr::Ceil => check_inputs!(),
        FunctionExpr::UpperBound => check_inputs!(),
        FunctionExpr::LowerBound => check_inputs!(),
        FunctionExpr::ConcatExpr(_) => check_inputs!(),
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
        FunctionExpr::BackwardFill { .. } => check_inputs!(),
        FunctionExpr::ForwardFill { .. } => check_inputs!(),
        FunctionExpr::MaxHorizontal => check_inputs!(aligned_rows),
        FunctionExpr::MinHorizontal => check_inputs!(aligned_rows),
        FunctionExpr::SumHorizontal => check_inputs!(aligned_rows),
        FunctionExpr::MeanHorizontal => check_inputs!(aligned_rows),
        FunctionExpr::Replace => {
            return fallible!(MakeTransformation, "replace is not currently supported.");
        }
        FunctionExpr::ReplaceStrict { .. } => {
            return fallible!(
                MakeTransformation,
                "replace_strict is not currently supported."
            );
        }
        FunctionExpr::GatherEvery { .. } => check_inputs!(),
        FunctionExpr::ExtendConstant => check_inputs!(),
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

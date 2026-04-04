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

/// Transformation for stable group-by and aggregate.
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
        predicates,
        aggs,
        apply,
        maintain_order,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected group-by in logical plan");
    };

    if apply.is_some() {
        return fallible!(
            MakeTransformation,
            "Apply is not currently supported in the logical plan. Please open an issue if this would be useful to you."
        );
    }

    if !predicates.is_empty() {
        // This is equivalent to running a filter after, as far as I can tell.
        // Possibly useful to support.
        return fallible!(
            MakeTransformation,
            "Having is not currently supported in logical plan. Please open an issue if this would be useful to you."
        );
    }

    if maintain_order {
        return fallible!(
            MakeTransformation,
            "maintain_order is wasted compute because row ordering is protected information"
        );
    }

    if options.as_ref() != &GroupbyOptions::default() {
        return fallible!(
            MakeTransformation,
            "Options is not currently supported. Please open an issue if this would be useful to you."
        );
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
            predicates: vec![],
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

const INVITE: &'static str = "Please open an issue if this would be useful to you.";

/// # Proof Definition
/// Returns an error if the expression may raise data-dependent errors,
/// or if `resize` is Ban and the expression resizes the data.
///
/// A resize is an expression that changes the number of rows in the data.
/// Scalar-valued expressions are not considered a resize,
/// because they can be broadcasted.
pub(crate) fn check_infallible(expr: &Expr, resize: Resize) -> Fallible<()> {
    Ok(match expr {
        Expr::Rolling {
            function,
            index_column,
            ..
        } => {
            check_infallible(&*function, resize)?;
            check_infallible(&*index_column, resize)?;
        }
        Expr::Element => (),
        Expr::Over {
            function,
            partition_by,
            order_by,
            ..
        } => {
            check_infallible(&*function, resize)?;
            partition_by
                .iter()
                .try_for_each(|by| check_infallible(&*by, Resize::Ban))?;
            if let Some((order, _)) = order_by {
                check_infallible(order, Resize::Ban)?;
            }
        }
        Expr::Eval {
            expr, evaluation, ..
        } => {
            check_infallible(expr, resize)?;
            check_infallible(evaluation, Resize::Allow)?;
        }
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
            AggExpr::Count { input: e, .. } => check_infallible(e, Resize::Allow)?,
            AggExpr::Quantile { expr: e, .. } => check_infallible(e, Resize::Allow)?,
            AggExpr::Max { input: e, .. } => check_infallible(e, Resize::Allow)?,
            AggExpr::Min { input: e, .. } => check_infallible(e, Resize::Allow)?,
            AggExpr::Std(e, _) => check_infallible(e, Resize::Allow)?,
            AggExpr::Var(e, _) => check_infallible(e, Resize::Allow)?,
            AggExpr::AggGroups(e) => check_infallible(e, Resize::Allow)?,
            AggExpr::FirstNonNull(e) | AggExpr::LastNonNull(e) => {
                check_infallible(e, Resize::Allow)?
            }
            AggExpr::Item { .. } => {
                return fallible!(
                    MakeTransformation,
                    "item raises an error if length is not one."
                );
            }
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
            return fallible!(
                MakeTransformation,
                "SubPlans are not currently supported. {INVITE}"
            );
        }
        Expr::DataTypeFunction(_) => (),
    })
}

/// # Proof Definition
/// Returns an error if the function expression may raise data-dependent errors,
/// or if `resize` is Ban and the expression resizes the data.
///
/// A resize is an expression that changes the number of rows in the data.
/// Scalar-valued expressions are not considered a resize,
/// because they can be broadcasted.
#[allow(unreachable_patterns)]
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
            BooleanFunction::IsClose { .. } => check_inputs!(),
            BooleanFunction::IsNull => check_inputs!(),
            BooleanFunction::IsNotNull => check_inputs!(),
            BooleanFunction::IsFinite => check_inputs!(),
            BooleanFunction::IsInfinite => check_inputs!(),
            BooleanFunction::IsFirstDistinct => check_inputs!(),
            BooleanFunction::IsLastDistinct => check_inputs!(),
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
            _ => match bool_expr.to_string().as_str() {
                "is_unique" | "is_duplicated" => check_inputs!(),
                _ => {
                    return fallible!(
                        MakeTransformation,
                        "Boolean function is not currently supported. {INVITE}"
                    );
                }
            },
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
            return fallible!(
                MakeTransformation,
                "Random may raise data-dependent errors due to sampling n without replacement from a set of length less than n."
            );
        }
        FunctionExpr::SetSortedFlag(_) => check_inputs!(),
        #[cfg(feature = "ffi")]
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
        FunctionExpr::ArrayExpr(array_function) => match array_function {
            // Unary on the array column
            ArrayFunction::Length
            | ArrayFunction::Min
            | ArrayFunction::Max
            | ArrayFunction::Sum
            | ArrayFunction::ToList
            | ArrayFunction::Unique(_)
            | ArrayFunction::NUnique
            | ArrayFunction::Std(_)
            | ArrayFunction::Var(_)
            | ArrayFunction::Mean
            | ArrayFunction::Median
            | ArrayFunction::Sort(_)
            | ArrayFunction::Reverse
            | ArrayFunction::ArgMin
            | ArrayFunction::ArgMax => check_inputs!(),

            // Still unary, but changes row count (Array width is schema-known, but it is a resize)
            ArrayFunction::Explode(_) => check_inputs!(resize = "array.explode"),

            // Multi-input: allow scalar broadcast, but ban mismatched lengths if Resize::Ban
            ArrayFunction::Get(_)
            | ArrayFunction::Join(_)
            | ArrayFunction::Contains { .. }
            | ArrayFunction::Shift
            | ArrayFunction::Concat
            | ArrayFunction::Slice(_, _) => check_inputs!(aligned_rows),
            _ => match array_function.to_string().as_str() {
                "arr.any" | "arr.all" | "arr.to_struct" => check_inputs!(),
                "arr.count_matches" => check_inputs!(aligned_rows),
                _ => {
                    return fallible!(
                        MakeTransformation,
                        "Array function is not currently supported. {INVITE}"
                    );
                }
            },
        },
        FunctionExpr::BinaryExpr(binary_function) => match binary_function {
            BinaryFunction::Contains
            | BinaryFunction::StartsWith
            | BinaryFunction::EndsWith
            | BinaryFunction::HexEncode
            | BinaryFunction::Size
            | BinaryFunction::Slice
            | BinaryFunction::Head
            | BinaryFunction::Tail
            | BinaryFunction::Reinterpret(_, _) => check_inputs!(aligned_rows),

            // Decode may raise if strict=true (invalid encoding depends on data)
            BinaryFunction::HexDecode(strict) => {
                if *strict {
                    return fallible!(
                        MakeTransformation,
                        "hex_decode(strict=true) may raise data-dependent errors."
                    );
                }
                check_inputs!(aligned_rows)
            }
            BinaryFunction::Base64Decode(strict) => {
                if *strict {
                    return fallible!(
                        MakeTransformation,
                        "base64_decode(strict=true) may raise data-dependent errors."
                    );
                }
                check_inputs!(aligned_rows)
            }
            BinaryFunction::Base64Encode => check_inputs!(),
        },
        FunctionExpr::Categorical(categorical_function) => match categorical_function {
            // These are schema/type dependent but not value-fallible in the “data-dependent error” sense.
            CategoricalFunction::GetCategories
            | CategoricalFunction::LenBytes
            | CategoricalFunction::LenChars => check_inputs!(),

            // Pattern inputs may be per-row, so require row alignment rules.
            CategoricalFunction::StartsWith(_)
            | CategoricalFunction::EndsWith(_)
            | CategoricalFunction::Slice(_, _) => check_inputs!(aligned_rows),
        },
        FunctionExpr::ListExpr(list_function) => match list_function {
            ListFunction::Concat => check_inputs!(aligned_rows),
            ListFunction::Contains { .. } => check_inputs!(aligned_rows),
            ListFunction::Slice => check_inputs!(aligned_rows),
            ListFunction::Shift => check_inputs!(aligned_rows),
            ListFunction::Sum
            | ListFunction::Length
            | ListFunction::Max
            | ListFunction::Min
            | ListFunction::Mean
            | ListFunction::Median
            | ListFunction::Std(_)
            | ListFunction::Var(_)
            | ListFunction::ArgMin
            | ListFunction::ArgMax
            | ListFunction::Diff { .. }
            | ListFunction::Sort(_)
            | ListFunction::Reverse
            | ListFunction::Unique(_)
            | ListFunction::NUnique => check_inputs!(),
            ListFunction::Get(null_on_oob) => {
                if !*null_on_oob {
                    return fallible!(
                        MakeTransformation,
                        "list.get/gather(null_on_oob=false) may raise data-dependent errors."
                    );
                }
                check_inputs!(aligned_rows)
            }
            ListFunction::Join(_) => check_inputs!(aligned_rows),
            ListFunction::ToArray(_) => {
                return fallible!(
                    MakeTransformation,
                    "list.to_array may raise data-dependent errors if list lengths don't match the target width."
                );
            }
            _ => match list_function.to_string().as_str() {
                "list.count_matches"
                | "list.union"
                | "list.difference"
                | "list.intersection"
                | "list.symmetric_difference" => check_inputs!(aligned_rows),
                "list.gather" | "list.gather_every" => check_inputs!(aligned_rows),
                "list.sample_n" | "list.sample_fraction" => {
                    return fallible!(
                        MakeTransformation,
                        "list.sample may raise data-dependent errors when sampling without replacement."
                    );
                }
                "list.drop_nulls" | "list.any" | "list.all" | "list.to_struct" => {
                    check_inputs!()
                }
                _ => {
                    return fallible!(
                        MakeTransformation,
                        "List function is not currently supported. {INVITE}"
                    );
                }
            },
        },
        FunctionExpr::StringExpr(string_function) => match string_function {
            // Most string transforms don't “value-fail” (they may be null-propagating).
            StringFunction::Format { .. }
            | StringFunction::ConcatHorizontal { .. }
            | StringFunction::ConcatVertical { .. }
            | StringFunction::CountMatches(_)
            | StringFunction::EndsWith
            | StringFunction::Extract(_)
            | StringFunction::ExtractAll
            | StringFunction::ExtractGroups { .. }
            | StringFunction::LenBytes
            | StringFunction::LenChars
            | StringFunction::Lowercase
            | StringFunction::Replace { .. }
            | StringFunction::Reverse
            | StringFunction::Slice
            | StringFunction::Head
            | StringFunction::Tail
            | StringFunction::HexEncode
            | StringFunction::Base64Encode
            | StringFunction::StartsWith
            | StringFunction::StripChars
            | StringFunction::StripCharsStart
            | StringFunction::StripCharsEnd
            | StringFunction::StripPrefix
            | StringFunction::StripSuffix
            | StringFunction::SplitExact { .. }
            | StringFunction::SplitN(_)
            | StringFunction::Split(_)
            | StringFunction::ToDecimal { .. }
            | StringFunction::Uppercase
            | StringFunction::EscapeRegex => check_inputs!(aligned_rows),

            // Regex/pattern operations: strict=true means “raise on bad pattern/match”.
            StringFunction::Contains { strict, .. } | StringFunction::Find { strict, .. } => {
                if *strict {
                    return fallible!(
                        MakeTransformation,
                        "string contains/find with strict=true may raise data-dependent errors."
                    );
                }
                check_inputs!(aligned_rows)
            }

            // Conversion: strict=true raises if any conversion fails.
            StringFunction::ToInteger { strict, .. } => {
                if *strict {
                    return fallible!(
                        MakeTransformation,
                        "str.to_integer(strict=true) may raise data-dependent errors."
                    );
                }
                check_inputs!(aligned_rows)
            }

            // Binary decode: strict=true raises on invalid encoding.
            StringFunction::HexDecode(strict) | StringFunction::Base64Decode(strict) => {
                if *strict {
                    return fallible!(
                        MakeTransformation,
                        "string decode with strict=true may raise data-dependent errors."
                    );
                }
                check_inputs!(aligned_rows)
            }

            StringFunction::Strptime(_, options) => {
                if options.strict {
                    return fallible!(
                        MakeTransformation,
                        "strptime strict may raise data-dependent errors."
                    );
                }
                check_inputs!(aligned_rows)
            }
            _ => match string_function.to_string().as_str() {
                "normalize" | "pad_start" | "pad_end" | "zfill" | "contains_any"
                | "replace_many" | "extract_many" | "find_many" => {
                    check_inputs!(aligned_rows)
                }
                _ => {
                    return fallible!(
                        MakeTransformation,
                        "String function is not currently supported. {INVITE}"
                    );
                }
            },
        },
        FunctionExpr::StructExpr(struct_function) => match struct_function {
            // Schema/metadata driven; not value-fallible.
            StructFunction::FieldByName(_) => check_inputs!(),
            StructFunction::RenameFields(_) => check_inputs!(),
            StructFunction::PrefixFields(_) => check_inputs!(),
            StructFunction::SuffixFields(_) => check_inputs!(),
            StructFunction::JsonEncode => check_inputs!(),
            StructFunction::WithFields => check_inputs!(aligned_rows),
            StructFunction::SelectFields(_) => check_inputs!(),
            // Callback is applied to field names (schema), not values.
            StructFunction::MapFieldNames(_) => check_inputs!(),
        },

        FunctionExpr::TemporalExpr(temporal_function) => match temporal_function {
            // Component extraction / conversions (value-safe; null-propagating).
            TemporalFunction::Millennium
            | TemporalFunction::Century
            | TemporalFunction::Year
            | TemporalFunction::IsLeapYear
            | TemporalFunction::IsoYear
            | TemporalFunction::Quarter
            | TemporalFunction::Month
            | TemporalFunction::DaysInMonth
            | TemporalFunction::Week
            | TemporalFunction::WeekDay
            | TemporalFunction::Day
            | TemporalFunction::OrdinalDay
            | TemporalFunction::Time
            | TemporalFunction::Date
            | TemporalFunction::Datetime
            | TemporalFunction::Hour
            | TemporalFunction::Minute
            | TemporalFunction::Second
            | TemporalFunction::Millisecond
            | TemporalFunction::Microsecond
            | TemporalFunction::Nanosecond
            | TemporalFunction::TotalDays { .. }
            | TemporalFunction::TotalHours { .. }
            | TemporalFunction::TotalMinutes { .. }
            | TemporalFunction::TotalSeconds { .. }
            | TemporalFunction::TotalMilliseconds { .. }
            | TemporalFunction::TotalMicroseconds { .. }
            | TemporalFunction::TotalNanoseconds { .. }
            | TemporalFunction::ToString(_)
            | TemporalFunction::Duration(_)
            | TemporalFunction::CastTimeUnit(_)
            | TemporalFunction::WithTimeUnit(_)
            | TemporalFunction::ConvertTimeZone(_)
            | TemporalFunction::TimeStamp(_)
            | TemporalFunction::BaseUtcOffset
            | TemporalFunction::DSTOffset => check_inputs!(),

            // These can raise depending on values around DST ambiguity/non-existent local times.
            TemporalFunction::Truncate | TemporalFunction::Round | TemporalFunction::Replace => {
                return fallible!(
                    MakeTransformation,
                    "Temporal operation may raise data-dependent errors due to ambiguous/non-existent datetimes (DST)."
                );
            }

            // ReplaceTimeZone has explicit ambiguous/non_existent handling and defaults to raising.
            TemporalFunction::ReplaceTimeZone(_, _) => {
                return fallible!(
                    MakeTransformation,
                    "replace_time_zone may raise data-dependent errors due to ambiguous/non-existent datetimes (DST)."
                );
            }

            // Combining date/time: aligns rows with the provided time expression/column.
            TemporalFunction::Combine(_) => check_inputs!(aligned_rows),

            // Datetime constructor/localization can raise on ambiguous datetimes depending on values.
            TemporalFunction::DatetimeFunction { .. } => {
                return fallible!(
                    MakeTransformation,
                    "datetime construction/localization may raise data-dependent errors due to ambiguous/non-existent datetimes (DST)."
                );
            }
            _ => match temporal_function.to_string().as_str() {
                "month_start" | "month_end" => check_inputs!(),
                "offset_by" => {
                    return fallible!(
                        MakeTransformation,
                        "Temporal operation may raise data-dependent errors due to ambiguous/non-existent datetimes (DST)."
                    );
                }
                _ => {
                    return fallible!(
                        MakeTransformation,
                        "Temporal function is not currently supported. {INVITE}"
                    );
                }
            },
        },

        FunctionExpr::Bitwise(bitwise_function) => match bitwise_function {
            BitwiseFunction::CountOnes
            | BitwiseFunction::CountZeros
            | BitwiseFunction::LeadingOnes
            | BitwiseFunction::LeadingZeros
            | BitwiseFunction::TrailingOnes
            | BitwiseFunction::TrailingZeros => check_inputs!(),

            BitwiseFunction::And | BitwiseFunction::Or | BitwiseFunction::Xor => {
                check_inputs!(aligned_rows)
            }
        },

        // Expr.hash(...) is value-safe; seeds affect determinism but not fallibility.
        FunctionExpr::Hash(_, _, _, _) => check_inputs!(),

        // Returns indices where condition is true => resizes based on values.
        FunctionExpr::ArgWhere => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "arg_where resizes the data (number of true values depends on input)."
                );
            }
            // Condition must be row-aligned; don't allow inner resizes.
            let [cond] = <&[Expr; 1]>::try_from(inputs.as_slice())
                .map_err(|_| err!(MakeTransformation, "arg_where must have one argument"))?;
            check_infallible(cond, Resize::Ban)?;
        }

        // Fixed-length rolling windows are value-safe (bad args are param-dependent, not data-dependent).
        FunctionExpr::RollingExpr { .. } => check_inputs!(),
        FunctionExpr::RollingExprBy { .. } => check_inputs!(aligned_rows),

        FunctionExpr::Rechunk => check_inputs!(),

        // Appends two expressions => resizes. Allow only if Resize::Allow.
        FunctionExpr::Append { .. } => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "append resizes the data (concatenates chunks/rows)."
                );
            }
            inputs
                .iter()
                .try_for_each(|e| check_infallible(e, Resize::Allow))?;
        }

        // Mode can return multiple values (ties) => resizes based on data.
        FunctionExpr::Mode { .. } => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "mode resizes the data (can return multiple values)."
                );
            }
            inputs
                .iter()
                .try_for_each(|e| check_infallible(e, Resize::Allow))?;
        }

        // Scalar aggregates (broadcastable)
        FunctionExpr::Skew(_) => check_inputs!(aggregate),
        FunctionExpr::Kurtosis(_, _) => check_inputs!(aggregate),
        FunctionExpr::ArgMin => check_inputs!(aggregate),
        FunctionExpr::ArgMax => check_inputs!(aggregate),
        FunctionExpr::Product => check_inputs!(aggregate),
        FunctionExpr::ApproxNUnique => check_inputs!(aggregate),

        // Argsort returns permutation indices of same length.
        FunctionExpr::ArgSort { .. } => check_inputs!(),

        // Cumulative ops keep length.
        FunctionExpr::CumCount { .. }
        | FunctionExpr::CumSum { .. }
        | FunctionExpr::CumProd { .. }
        | FunctionExpr::CumMin { .. }
        | FunctionExpr::CumMax { .. } => check_inputs!(),

        // unique_counts returns counts per unique value => resizes.
        FunctionExpr::UniqueCounts => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "unique_counts resizes the data (one row per unique value)."
                );
            }
            inputs
                .iter()
                .try_for_each(|e| check_infallible(e, Resize::Allow))?;
        }

        FunctionExpr::Diff(_) => check_inputs!(),
        FunctionExpr::Interpolate(_) => check_inputs!(),
        FunctionExpr::InterpolateBy => check_inputs!(aligned_rows),

        FunctionExpr::PeakMin => check_inputs!(),
        FunctionExpr::PeakMax => check_inputs!(),

        // rle returns runs => resizes based on values.
        FunctionExpr::RLE => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "rle resizes the data (one row per run)."
                );
            }
            inputs
                .iter()
                .try_for_each(|e| check_infallible(e, Resize::Allow))?;
        }
        // rle_id is per-row run labels => same length.
        FunctionExpr::RLEID => check_inputs!(),

        // These carry callbacks; treat as potentially value-fallible.
        FunctionExpr::FoldHorizontal { .. }
        | FunctionExpr::ReduceHorizontal { .. }
        | FunctionExpr::CumReduceHorizontal { .. }
        | FunctionExpr::CumFoldHorizontal { .. } => {
            return fallible!(
                MakeTransformation,
                "fold/reduce with callbacks may raise data-dependent errors."
            );
        }

        // Non-strict replace is intended to be non-fallible w.r.t. missing mapping keys.
        FunctionExpr::Replace => check_inputs!(aligned_rows),

        // Strict replace can error on incomplete mapping unless a default is present; reject conservatively.
        FunctionExpr::ReplaceStrict { .. } => {
            return fallible!(
                MakeTransformation,
                "replace_strict may raise data-dependent errors due to incomplete mapping."
            );
        }

        // Picks every n-th row => resizes.
        FunctionExpr::GatherEvery { .. } => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "gather_every resizes the data (subsamples rows)."
                );
            }
            inputs
                .iter()
                .try_for_each(|e| check_infallible(e, Resize::Allow))?;
        }
        // Extends series by constants => resizes.
        FunctionExpr::ExtendConstant => {
            if matches!(resize, Resize::Ban) {
                return fallible!(
                    MakeTransformation,
                    "extend_constant resizes the data (adds rows)."
                );
            }
            inputs
                .iter()
                .try_for_each(|e| check_infallible(e, Resize::Allow))?;
        }

        // Row encoding is length-preserving (encodes multiple cols into one binary col).
        FunctionExpr::RowEncode(_) => check_inputs!(aligned_rows),

        // Decoding can error if bytes/metadata are invalid; be conservative.
        FunctionExpr::RowDecode(_, _) => {
            return fallible!(
                MakeTransformation,
                "row_decode may raise data-dependent errors on invalid encoded data."
            );
        }
        _ => match function.to_string().as_str() {
            "hist" => check_inputs!(resize = "hist"),
            "index_of" => check_inputs!(aggregate),
            "search_sorted" => check_inputs!(aligned_rows),
            "top_k" | "top_k_by" => check_inputs!(resize = "top_k"),
            "pct_change" => check_inputs!(aligned_rows),
            "ewm_mean" | "ewm_std" | "ewm_var" | "reinterpret" => check_inputs!(),
            "cos" | "cot" | "sin" | "tan" | "arccos" | "arcsin" | "arctan" | "cosh" | "sinh"
            | "tanh" | "arccosh" | "arcsinh" | "arctanh" | "degrees" | "radians" | "sign"
            | "log" | "log1p" | "exp" => check_inputs!(),
            "atan2" => check_inputs!(aligned_rows),
            "ewm_mean_by" | "business_day_count" | "add_business_days" => {
                check_inputs!(aligned_rows)
            }
            "entropy" | "corr" | "spearman_rank_corr" => check_inputs!(aggregate),
            "is_business_day" | "ext.storage()" => check_inputs!(),
            name if name.starts_with("ext.to(") => check_inputs!(),
            _ => {
                return fallible!(
                    MakeTransformation,
                    "Function expression is not currently supported. {INVITE}"
                );
            }
        },
    })
}
